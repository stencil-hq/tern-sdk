package tern

import (
	"strconv"
	"strings"
)

// Mode is how a surface sits in the pane.
type Mode string

// Surface modes.
const (
	// Inline sits at the cursor row and owns the pane while live (a session program).
	Inline Mode = "inline"
	// Screen covers the whole pane like the alternate screen.
	Screen Mode = "screen"
	// Flow sits among the output around it and stays in the scrollback (a CLI).
	Flow Mode = "flow"
)

// SurfaceOptions configure Session.Open.
type SurfaceOptions struct {
	// ID names the surface; it defaults to s1, s2, … per session.
	ID string
	// Mode is Inline (the default), Screen or Flow.
	Mode Mode
	// Title names the pane until the program sets its own title.
	Title string
	// Role names the surface for stylesheets (data-surface).
	Role string
	// NoListen opens with listen:false: the program never reads input for
	// it, so Tern sends no acks or events and frames go out at once.
	NoListen bool
	// Adopt reopens the newest closed inline surface with ID.
	Adopt bool
	// Discard makes Close remove the surface (keep:false) instead of
	// leaving it in the scrollback.
	Discard bool
}

// Surface is one open document in the pane. Render diffs views into frames
// under credit-based flow control. Its methods are safe for concurrent use.
type Surface struct {
	s       *Session
	id      string
	listen  bool
	discard bool
	closed  bool

	seq, acked int64
	sent       *viewTree
	want       *viewTree
	dirty      bool
	queue      []Op
	resetMain  bool
}

// Open opens a surface.
func (s *Session) Open(opts SurfaceOptions) (*Surface, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	if opts.ID == "" {
		for {
			s.nextID++
			opts.ID = "s" + strconv.Itoa(s.nextID)
			if s.surfaces[opts.ID] == nil {
				break
			}
		}
	}
	if opts.Mode == "" {
		opts.Mode = Inline
	}
	msg := OpenMsg{ID: opts.ID, Mode: opts.Mode, Title: opts.Title, Role: opts.Role, Adopt: opts.Adopt}
	if opts.NoListen {
		msg.Listen = new(false)
	}
	if err := s.sendJSON(VerbOpen, msg); err != nil {
		return nil, err
	}
	sf := &Surface{s: s, id: opts.ID, listen: !opts.NoListen, discard: opts.Discard}
	if old := s.surfaces[opts.ID]; old != nil {
		if opts.Adopt && old.closed {
			sf.seq, sf.acked = old.seq, old.seq
			if old.sent != nil {
				handlers := make(map[string]*bound)
				for id, handler := range old.sent.handlers {
					if id == "main" || strings.HasPrefix(id, "main.") {
						handlers[id] = handler
					}
				}
				sf.sent = &viewTree{regions: [3]*tree{old.sent.regions[0]}, handlers: handlers}
				sf.want = sf.sent
			}
		}
		old.closed = true
	} else {
		sf.resetMain = opts.Adopt
	}
	s.surfaces[opts.ID] = sf
	s.order = append(s.order, sf)
	return sf, nil
}

// ID is the surface's id, the sf of its frames and events.
func (sf *Surface) ID() string { return sf.id }

// Closed reports whether the surface was closed, by the program or by a
// gone event.
func (sf *Surface) Closed() bool {
	sf.s.mu.Lock()
	defer sf.s.mu.Unlock()
	return sf.closed
}

// Render shows v: the difference from the last view sent goes out in one
// frame, or, while every credit is in use, v is kept and its difference
// sent when an ack returns a credit. A view with two siblings of one id is
// rejected and nothing is sent.
func (sf *Surface) Render(v View) error {
	vt, err := compile(v)
	if err != nil {
		return err
	}
	sf.s.mu.Lock()
	defer sf.s.mu.Unlock()
	if sf.closed {
		return ErrClosed
	}
	sf.want = vt
	sf.dirty = true
	return sf.flushLocked(false)
}

// Send queues raw frame ops, sent after the reconciled difference of the
// next frame.
func (sf *Surface) Send(ops ...Op) error {
	sf.s.mu.Lock()
	defer sf.s.mu.Unlock()
	if sf.closed {
		return ErrClosed
	}
	sf.queue = append(sf.queue, ops...)
	return sf.flushLocked(false)
}

// Focus gives the caret to editor or input id ("" for none).
func (sf *Surface) Focus(id string) error { return sf.Send(OpFocus(id)) }

// Reveal scrolls node id into view at "start", "end" or "nearest".
func (sf *Surface) Reveal(id, at string) error { return sf.Send(OpReveal(id, at)) }

// Scroll scrolls the container of id by "line-up", "line-down", "page-up",
// "page-down", "start" or "end".
func (sf *Surface) Scroll(id, by string) error { return sf.Send(OpScroll(id, by)) }

// Settle hints that subtree id is unlikely to change soon.
func (sf *Surface) Settle(id string) error { return sf.Send(OpSettle(id)) }

// Suspend hands the pane back to the grid (an external command).
func (sf *Surface) Suspend() error { return sf.Send(OpSuspend()) }

// Resume takes the pane again after Suspend.
func (sf *Surface) Resume() error { return sf.Send(OpResume()) }

// Stylesheet installs or replaces stylesheet name; an empty css removes it.
func (sf *Surface) Stylesheet(name, css string) error {
	sf.s.mu.Lock()
	defer sf.s.mu.Unlock()
	if sf.closed {
		return ErrClosed
	}
	msg := SheetMsg{SF: sf.id, Name: name}
	if css != "" {
		msg.CSS = &css
	}
	return sf.s.sendJSON(VerbSheet, msg)
}

// Palette is a program palette: token → #rrggbb per appearance.
type Palette struct {
	Dark  map[string]string
	Light map[string]string
	Name  *PaletteNames
}

// Palette sends the surface's program palette, replacing the last one.
func (sf *Surface) Palette(p Palette) error {
	sf.s.mu.Lock()
	defer sf.s.mu.Unlock()
	if sf.closed {
		return ErrClosed
	}
	return sf.s.sendJSON(VerbPalette, PaletteMsg{SF: sf.id, Dark: p.Dark, Light: p.Light, Name: p.Name})
}

// Close sends what is still pending and closes the surface, keeping it in
// the scrollback unless it was opened with Discard.
func (sf *Surface) Close() error {
	sf.s.mu.Lock()
	defer sf.s.mu.Unlock()
	if sf.closed {
		return nil
	}
	return sf.closeLocked(!sf.discard)
}

// Remove sends what is still pending and closes the surface with keep:false.
func (sf *Surface) Remove() error {
	sf.s.mu.Lock()
	defer sf.s.mu.Unlock()
	if sf.closed {
		return nil
	}
	return sf.closeLocked(false)
}

// closeLocked flushes pending changes past the credit limit, so the last
// view is the one left on screen, then sends x. The caller holds mu.
func (sf *Surface) closeLocked(keep bool) error {
	err := sf.flushLocked(true)
	if e := sf.s.sendJSON(VerbClose, CloseMsg{ID: sf.id, Keep: keep}); err == nil {
		err = e
	}
	sf.closed = true
	return err
}

// ackLocked takes an ack for every frame up to s and sends what waits.
// The caller holds mu.
func (sf *Surface) ackLocked(s int64) error {
	if s > sf.acked {
		sf.acked = min(s, sf.seq)
	}
	if !sf.closed {
		return sf.flushLocked(false)
	}
	return nil
}

// flushLocked sends one frame with the reconciled difference and the
// queued ops when a credit is free (or force). The caller holds mu.
func (sf *Surface) flushLocked(force bool) error {
	if !sf.dirty && len(sf.queue) == 0 {
		return nil
	}
	if !force && sf.listen && sf.seq-sf.acked >= int64(sf.s.caps.Credits) {
		return nil
	}
	var ops []Op
	if sf.resetMain {
		ops = append(ops, OpDel("main"))
	}
	if sf.dirty {
		ops = append(ops, diffViews(sf.sent, sf.want, sf.id)...)
	}
	ops = append(ops, sf.queue...)
	if len(ops) > 0 {
		if err := sf.s.sendJSON(VerbFrame, FrameMsg{SF: sf.id, S: sf.seq + 1, Ops: ops}); err != nil {
			return err
		}
		sf.seq++
	}
	if sf.dirty {
		sf.sent = sf.want
		sf.dirty = false
	}
	sf.queue = nil
	sf.resetMain = false
	return nil
}
