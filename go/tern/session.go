package tern

import (
	"context"
	"errors"
	"io"
	"os"
	"path/filepath"
	"slices"
	"sync"
	"sync/atomic"
	"time"

	"golang.org/x/term"
)

// Errors a session reports.
var (
	// ErrUnsupported means TSP is not available: TERN_TSP=0, no tty, a
	// multiplexer, or the terminal did not answer hello. Fall back to plain
	// output.
	ErrUnsupported = errors.New("tern: TSP is not available")
	// ErrClosed is returned by a closed session or surface.
	ErrClosed = errors.New("tern: closed")
	// ErrStop ends Session.Run without an error when a callback returns it.
	ErrStop = errors.New("tern: stop")
)

// Timing of the input loop.
const (
	// DefaultTimeout is how long Connect waits for the hello reply.
	DefaultTimeout = time.Second
	// FlushDelay is the pause in input after which undecided bytes are keys.
	FlushDelay = 30 * time.Millisecond
	// DrainDelay is how long Close keeps reading input so late replies
	// never reach the shell.
	DrainDelay = 50 * time.Millisecond
)

// Terminal mode sequences a session sets and undoes.
const (
	pasteModeOn  = "\x1b[?2004h"
	pasteModeOff = "\x1b[?2004l"
	kittyPush    = "\x1b[>1u"
	kittyPop     = "\x1b[<u"
	da1Request   = "\x1b[c"
)

// Only one session may own the process terminal's modes.
var ttyOwned atomic.Bool

type readResult struct {
	data []byte
	err  error
}

// Options configure Connect.
type Options struct {
	// App is the program's name in hello; it defaults to the executable's name.
	App string
	// Version is the program's version in hello.
	Version string
	// Features are the program features it handles (FeatureEdit, …).
	Features []string
	// Timeout is how long to wait for the hello reply (default 1 s).
	Timeout time.Duration
	// In is the pty's input (default os.Stdin).
	In io.Reader
	// Out is the pty's output (default os.Stdout).
	Out io.Writer
	// MakeRaw switches the terminal to raw mode and returns how to restore
	// it. When nil, In and Out must be terminals and In's is made raw; when
	// set, the caller vouches for the terminal (tests drive a session over
	// pipes this way).
	MakeRaw func() (restore func() error, err error)
	// CancelRead interrupts a custom In's blocked Read on close. If omitted,
	// a non-pollable In must implement io.ReadCloser and is closed instead.
	CancelRead func() error
	// NoPaste leaves bracketed paste mode alone.
	NoPaste bool
	// NoKitty leaves the kitty keyboard protocol alone.
	NoKitty bool
}

// Caps are what the terminal said it supports, kept current by resize,
// theme and motion events.
type Caps struct {
	Term         string
	Ver          string
	Kinds        []string
	Features     []string
	APC          int
	Credits      int
	Cols         int
	Cell         Cell
	Dark         bool
	ReduceMotion bool
	Hour12       bool // the user's system reads a 12-hour clock (3:05 PM); false (24-hour) when unsaid
}

// HasKind reports whether the terminal draws kind.
func (c Caps) HasKind(kind string) bool { return slices.Contains(c.Kinds, kind) }

// HasFeature reports whether the terminal lists feature.
func (c Caps) HasFeature(feature string) bool { return slices.Contains(c.Features, feature) }

// Session is a connection to Tern over the pane's pty: raw mode, surfaces,
// flow control and the input loop. Its methods are safe for concurrent use,
// except that one goroutine at a time reads input (Next or Run). Always
// Close it, typically with defer.
type Session struct {
	opts    Options
	out     io.Writer
	restore func() error
	rec     *recorder

	// mu guards everything below it up to inMu, and every write to out.
	mu       sync.Mutex
	caps     Caps
	surfaces map[string]*Surface
	order    []*Surface
	nextID   int
	chunks   uint64
	sent     map[string]bool
	closed   bool

	inMu   sync.Mutex
	queue  []Input
	inErr  error
	notify chan struct{}

	handshake  chan *HelloReply
	blobs      []chan *BlobsReply
	cancelRead func() error
	ownsTTY    bool
	modesArmed bool

	done       chan struct{}
	readerDone chan struct{}
	closeOnce  sync.Once
	stopSignal func()
}

// Connect detects Tern and says hello. It returns ErrUnsupported (and
// leaves the terminal as it was) when TSP is not available.
func Connect(ctx context.Context, opts Options) (*Session, error) {
	if os.Getenv("TERN_TSP") == "0" || os.Getenv("TMUX") != "" || os.Getenv("STY") != "" || os.Getenv("ZELLIJ") != "" {
		return nil, ErrUnsupported
	}
	if opts.In == nil {
		opts.In = os.Stdin
	}
	if opts.Out == nil {
		opts.Out = os.Stdout
	}
	if opts.Timeout <= 0 {
		opts.Timeout = DefaultTimeout
	}
	if opts.App == "" {
		opts.App = filepath.Base(os.Args[0])
	}
	var cancelRead func() error
	if f, ok := opts.In.(*os.File); !ok || !pollable(f) {
		cancelRead = opts.CancelRead
		if cancelRead == nil {
			if r, ok := opts.In.(io.ReadCloser); ok {
				cancelRead = r.Close
			} else {
				return nil, errors.New("tern: custom input requires CancelRead or io.ReadCloser")
			}
		}
	}
	makeRaw := opts.MakeRaw
	if makeRaw == nil {
		in, ok := opts.In.(*os.File)
		out, ok2 := opts.Out.(*os.File)
		if !ok || !ok2 || !term.IsTerminal(int(in.Fd())) || !term.IsTerminal(int(out.Fd())) {
			return nil, ErrUnsupported
		}
		makeRaw = func() (func() error, error) {
			fd := int(in.Fd())
			state, err := term.GetState(fd)
			if err != nil {
				return nil, err
			}
			restore := func() error { return term.Restore(fd, state) }
			_, err = term.MakeRaw(fd)
			return restore, err
		}
	}
	s := &Session{
		opts:       opts,
		out:        opts.Out,
		cancelRead: cancelRead,
		caps:       Caps{APC: DefaultAPC, Credits: DefaultCredits},
		surfaces:   map[string]*Surface{},
		sent:       map[string]bool{},
		notify:     make(chan struct{}, 1),
		handshake:  make(chan *HelloReply, 1),
		done:       make(chan struct{}),
		readerDone: make(chan struct{}),
	}
	s.mu.Lock()
	if opts.MakeRaw == nil {
		if !ttyOwned.CompareAndSwap(false, true) {
			s.mu.Unlock()
			return nil, errors.New("tern: terminal already owned by a session")
		}
		s.ownsTTY = true
		s.stopSignal = onSignal(func() { _ = s.Close() })
	}
	restore, err := makeRaw()
	s.restore = restore
	if err != nil {
		close(s.readerDone)
		s.mu.Unlock()
		s.abort()
		return nil, ErrUnsupported
	}
	s.rec = openRecorder(os.Getenv("TERN_TSP_RECORD"))
	chunks := make(chan readResult, 16)
	go s.read(chunks)
	go s.process(chunks)
	s.mu.Unlock()

	hello, _ := Marshal(HelloQuery{Q: "hello", V: []int{ProtocolVersion}, App: opts.App, Ver: opts.Version, Features: opts.Features})
	s.mu.Lock()
	err = s.sendLocked(VerbQuery, nil, hello, da1Request)
	s.mu.Unlock()
	if err != nil {
		s.abort()
		return nil, err
	}
	timer := time.NewTimer(opts.Timeout)
	defer timer.Stop()
	var reply *HelloReply
	select {
	case reply = <-s.handshake:
	case <-timer.C:
	case <-ctx.Done():
		s.abort()
		return nil, ctx.Err()
	}
	if reply == nil {
		s.abort()
		return nil, ErrUnsupported
	}
	s.mu.Lock()
	var modes string
	if !opts.NoPaste {
		modes += pasteModeOn
	}
	if !opts.NoKitty {
		modes += kittyPush
	}
	if modes != "" {
		s.modesArmed = true
		err = s.writeModes(modes)
	}
	s.mu.Unlock()
	if err != nil {
		s.abort()
		return nil, err
	}
	return s, nil
}

// capsOf is the capabilities a hello reply gives, with defaults for what it
// leaves out.
func capsOf(r *HelloReply) Caps {
	c := Caps{Term: r.Term, Ver: r.Ver, Kinds: r.Kinds, Features: r.Features, APC: r.APC,
		Credits: r.Credits, Cols: r.Cols, Cell: r.Cell, Dark: r.Dark, ReduceMotion: r.ReduceMotion,
		Hour12: r.Hour12}
	if c.APC <= 0 {
		c.APC = DefaultAPC
	}
	if c.Credits <= 0 {
		c.Credits = DefaultCredits
	}
	return c
}

// abort stops the input loop and restores the terminal after a failed
// Connect.
func (s *Session) abort() { _ = s.Close() }

// waitReader joins the actual reader before handing input back.
func (s *Session) waitReader() {
	if s.cancelRead != nil {
		_ = s.cancelRead()
	}
	<-s.readerDone
}

// read sends bytes and errors through one ordered stream.
func (s *Session) read(chunks chan<- readResult) {
	defer close(s.readerDone)
	if f, ok := s.opts.In.(*os.File); ok && pollable(f) {
		pollRead(f, chunks, s.done)
		return
	}
	buf := make([]byte, 4096)
	for {
		select {
		case <-s.done:
			return
		default:
		}
		n, err := s.opts.In.Read(buf)
		result := readResult{data: append([]byte(nil), buf[:n]...), err: err}
		select {
		case chunks <- result:
		case <-s.done:
			return
		}
		if err != nil {
			return
		}
	}
}

// process parses input, keeps flow control and capabilities current and
// queues keys and events for Next.
func (s *Session) process(chunks <-chan readResult) {
	var parser Parser
	var keys KeyDecoder
	// The first hello or DA1 decides the handshake in arrival order.
	handshakeDone := false
	flush := time.NewTimer(time.Hour)
	flush.Stop()
	defer flush.Stop()
	handle := func(items []Item, flushKeys bool) {
		for _, it := range items {
			switch {
			case it.Keys != nil:
				for _, k := range keys.Feed(it.Keys) {
					s.push(k)
				}
			case it.Reply != nil:
				s.rec.record("in", VerbReply, nil, it.Reply.Raw)
				if it.Reply.Hello != nil && !handshakeDone {
					handshakeDone = true
					s.mu.Lock()
					s.caps = capsOf(it.Reply.Hello)
					s.mu.Unlock()
					s.handshake <- it.Reply.Hello
				}
				s.reply(it.Reply)
			case it.Event != nil:
				s.rec.record("in", VerbEvent, nil, it.Event.Raw())
				s.event(it.Event)
			case it.DA1 && !handshakeDone:
				handshakeDone = true
				s.handshake <- nil
			}
		}
		if flushKeys {
			for _, k := range keys.Flush() {
				s.push(k)
			}
		}
	}
	for {
		select {
		case <-s.done:
			return
		case result := <-chunks:
			handle(parser.Feed(result.data), false)
			if result.err != nil {
				handle(parser.Flush(), true)
				s.failInput(result.err)
				return
			}
			flush.Reset(FlushDelay)
		case <-flush.C:
			handle(parser.Flush(), true)
		}
	}
}

// reply hands a reply to whoever waits for it.
func (s *Session) reply(r *Reply) {
	if r.Blobs != nil {
		s.mu.Lock()
		if len(s.blobs) > 0 {
			ch := s.blobs[0]
			s.blobs[0] = nil
			s.blobs = s.blobs[1:]
			ch <- r.Blobs
		}
		s.mu.Unlock()
	}
}

// event applies what an event says to the session and queues it for the
// program; acks are consumed here.
func (s *Session) event(ev Event) {
	s.mu.Lock()
	sf := s.surfaces[ev.Surface()]
	switch e := ev.(type) {
	case *Ack:
		if sf != nil {
			if err := sf.ackLocked(e.S); err != nil {
				s.failInput(err)
			}
		}
		s.mu.Unlock()
		return
	case *Resize:
		s.caps.Cols, s.caps.Cell = e.Cols, e.Cell
	case *Theme:
		s.caps.Dark = e.Dark
	case *Motion:
		s.caps.ReduceMotion = e.Reduce
	case *Gone:
		for _, id := range e.IDs {
			if g := s.surfaces[id]; g != nil && (e.SF == "" || e.SF == id) {
				g.closed = true
			}
		}
	}
	s.mu.Unlock()
	s.push(ev)
}

// failInput makes asynchronous read/write failures visible to Next and Run.
func (s *Session) failInput(err error) {
	s.inMu.Lock()
	if s.inErr == nil {
		s.inErr = err
	}
	s.inMu.Unlock()
	s.wake()
}

// push queues an input for Next.
func (s *Session) push(in Input) {
	s.inMu.Lock()
	s.queue = append(s.queue, in)
	s.inMu.Unlock()
	s.wake()
}

// wake signals a waiting Next.
func (s *Session) wake() {
	select {
	case s.notify <- struct{}{}:
	default:
	}
}

// pop waits for the next queued input.
func (s *Session) pop(ctx context.Context) (Input, error) {
	for {
		s.inMu.Lock()
		if len(s.queue) > 0 {
			in := s.queue[0]
			s.queue[0] = nil
			s.queue = s.queue[1:]
			s.inMu.Unlock()
			return in, nil
		}
		err := s.inErr
		s.inMu.Unlock()
		if err != nil {
			return nil, err
		}
		select {
		case <-s.notify:
		case <-ctx.Done():
			return nil, ctx.Err()
		case <-s.done:
			return nil, ErrClosed
		}
	}
}

// dispatch runs the handler of the node ev names, reporting whether one ran.
func (s *Session) dispatch(ev Event) bool {
	id := ev.Target()
	if id == "" {
		return false
	}
	s.mu.Lock()
	var b *bound
	if sf := s.surfaces[ev.Surface()]; sf != nil && sf.want != nil {
		b = sf.want.handlers[id]
	}
	s.mu.Unlock()
	return b != nil && b.route(ev)
}

// Next returns the next key or unhandled event, running node handlers for
// the events they handle on the calling goroutine. It returns io.EOF when
// input ends and ErrClosed after Close.
func (s *Session) Next(ctx context.Context) (Input, error) {
	for {
		in, err := s.pop(ctx)
		if err != nil {
			return nil, err
		}
		if ev, ok := in.(Event); ok && s.dispatch(ev) {
			continue
		}
		return in, nil
	}
}

// Run calls fn with each key and unhandled event until fn returns an error
// (ErrStop ends it with nil), ctx is done or input ends. Handlers run on
// Run's goroutine.
func (s *Session) Run(ctx context.Context, fn func(Input) error) error {
	for {
		in, err := s.Next(ctx)
		if err != nil {
			return err
		}
		if err := fn(in); err != nil {
			if errors.Is(err, ErrStop) {
				return nil
			}
			return err
		}
	}
}

// Caps returns the terminal's current capabilities.
func (s *Session) Caps() Caps {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.caps
}

// sendLocked writes one message (chunked as the terminal asks) followed by
// trailer, and records it. The caller holds mu.
func (s *Session) sendLocked(verb string, params []Param, body []byte, trailer string) error {
	if s.closed {
		return ErrClosed
	}
	if len(body) > s.caps.APC {
		s.chunks++
	}
	out := Encode(verb, params, body, s.caps.APC, chunkID(s.chunks))
	out = append(out, trailer...)
	if verb == VerbBlob {
		s.rec.recordString("out", verb, params, string(body))
	} else {
		s.rec.record("out", verb, params, body)
	}
	n, err := s.out.Write(out)
	if err == nil && n != len(out) {
		err = io.ErrShortWrite
	}
	return err
}

func (s *Session) writeModes(modes string) error {
	n, err := io.WriteString(s.out, modes)
	if err == nil && n != len(modes) {
		return io.ErrShortWrite
	}
	return err
}

// sendJSON marshals v and sends it as verb. The caller holds mu.
func (s *Session) sendJSON(verb string, v any) error {
	body, err := Marshal(v)
	if err != nil {
		return err
	}
	return s.sendLocked(verb, nil, body, "")
}

// Blob sends data once per session and returns its id for image nodes.
func (s *Session) Blob(data []byte, mime string) (string, error) {
	if len(data) > MaxBlob {
		return "", ErrBlobTooLarge
	}
	id := BlobID(data)
	s.mu.Lock()
	defer s.mu.Unlock()
	if s.sent[id] {
		return id, nil
	}
	var params []Param
	if mime != "" {
		params = append(params, Param{"mime", mime})
	}
	body := []byte(encodeBase64(data))
	if err := s.sendLocked(VerbBlob, params, body, ""); err != nil {
		return "", err
	}
	s.sent[id] = true
	return id, nil
}

// HaveBlobs asks which of ids Tern still holds; a reply is read by the
// input loop, so it needs no Next call.
func (s *Session) HaveBlobs(ctx context.Context, ids []string) ([]string, error) {
	ch := make(chan *BlobsReply, 1)
	s.mu.Lock()
	err := s.sendJSON(VerbQuery, BlobsQuery{Q: "blobs", IDs: ids})
	if err == nil {
		// A cancelled caller leaves its slot until its reply arrives.
		s.blobs = append(s.blobs, ch)
	}
	s.mu.Unlock()
	if err != nil {
		return nil, err
	}
	select {
	case r := <-ch:
		return r.Have, nil
	case <-ctx.Done():
		return nil, ctx.Err()
	case <-s.done:
		return nil, ErrClosed
	}
}

// Close closes the open surfaces (each as it says, kept by default), undoes
// the terminal modes, drains input for DrainDelay so late replies never
// reach the shell, and restores the tty. It is safe to call more than once.
func (s *Session) Close() error {
	var err error
	s.closeOnce.Do(func() {
		s.mu.Lock()
		for _, sf := range s.order {
			if !sf.closed {
				if e := sf.closeLocked(!sf.discard); e != nil && err == nil {
					err = e
				}
			}
		}
		var modes string
		if s.modesArmed && !s.opts.NoKitty {
			modes += kittyPop
		}
		if s.modesArmed && !s.opts.NoPaste {
			modes += pasteModeOff
		}
		if modes != "" {
			if e := s.writeModes(modes); e != nil && err == nil {
				err = e
			}
		}
		s.closed = true
		s.mu.Unlock()
		time.Sleep(DrainDelay)
		close(s.done)
		s.waitReader()
		if s.stopSignal != nil {
			s.stopSignal()
		}
		if s.restore != nil {
			if e := s.restore(); e != nil && err == nil {
				err = e
			}
		}
		if s.ownsTTY {
			ttyOwned.Store(false)
		}
		s.rec.close()
	})
	return err
}
