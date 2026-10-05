package tern

import (
	"bytes"
	"context"
	"errors"
	"io"
	"strings"
	"testing"
	"time"
)

var errWrite = errors.New("scripted write failure")

type faultWriter struct {
	bytes.Buffer
	fail  bool
	short bool
}

func (w *faultWriter) Write(p []byte) (int, error) {
	if w.fail {
		return 0, errWrite
	}
	if w.short {
		return len(p) - 1, nil
	}
	return w.Buffer.Write(p)
}

func (w *faultWriter) WriteString(s string) (int, error) { return w.Write([]byte(s)) }

func memorySession(w io.Writer) *Session {
	done := make(chan struct{})
	close(done)
	return &Session{out: w, caps: Caps{APC: DefaultAPC, Credits: 2}, surfaces: map[string]*Surface{}, sent: map[string]bool{}, notify: make(chan struct{}, 1), handshake: make(chan *HelloReply, 1), done: make(chan struct{}), readerDone: done}
}

func TestFrameWriteFailureRetainsState(t *testing.T) {
	w := new(faultWriter)
	s := memorySession(w)
	sf, _ := s.Open(SurfaceOptions{})
	w.fail = true
	view := Main(text("a", "hello"))
	if err := sf.Render(view); !errors.Is(err, errWrite) {
		t.Fatal(err)
	}
	if sf.seq != 0 || sf.sent != nil || !sf.dirty {
		t.Fatal("failed frame committed")
	}
	if err := sf.Focus("main.a"); !errors.Is(err, errWrite) {
		t.Fatal(err)
	}
	w.fail = false
	w.Reset()
	if err := sf.Render(view); err != nil {
		t.Fatal(err)
	}
	if sf.seq != 1 || !strings.Contains(w.String(), `["add","main"`) || !strings.Contains(w.String(), `["focus","main.a"]`) {
		t.Fatal(w.String())
	}
	w.short = true
	if err := sf.Render(Main(text("a", "changed"))); !errors.Is(err, io.ErrShortWrite) {
		t.Fatal(err)
	}
	if sf.seq != 1 {
		t.Fatal("short frame committed")
	}
}

func TestAckWriteFailureReachesNext(t *testing.T) {
	w := new(faultWriter)
	s := memorySession(w)
	sf, _ := s.Open(SurfaceOptions{})
	for _, v := range []string{"one", "two", "three"} {
		if err := sf.Render(Main(text("a", v))); err != nil {
			t.Fatal(err)
		}
	}
	w.fail = true
	ev, _ := DecodeEvent([]byte(`{"ev":"ack","sf":"s1","s":1}`))
	s.event(ev)
	if _, err := s.Next(context.Background()); !errors.Is(err, errWrite) {
		t.Fatal(err)
	}
	if sf.seq != 2 || !sf.dirty {
		t.Fatal("ack flush committed a failed frame")
	}
}

func TestAdoptPreservesMainAndSequence(t *testing.T) {
	w := new(faultWriter)
	s := memorySession(w)
	sf, _ := s.Open(SurfaceOptions{ID: "kept"})
	if err := sf.Render(View{Main: Nodes{text("a", "old")}, Dock: Nodes{text("ed", "draft")}, Layer: Nodes{text("popup", "open")}}); err != nil {
		t.Fatal(err)
	}
	if err := sf.Close(); err != nil {
		t.Fatal(err)
	}
	adopted, _ := s.Open(SurfaceOptions{ID: "kept", Adopt: true})
	w.Reset()
	if err := adopted.Render(View{Main: Nodes{text("a", "new")}, Dock: Nodes{text("ed", "new draft")}}); err != nil {
		t.Fatal(err)
	}
	got := w.String()
	if adopted.seq != 2 || strings.Contains(got, `["add","main"`) || !strings.Contains(got, `["text","main.a","replace","new"]`) || !strings.Contains(got, `["add","dock"`) || strings.Contains(got, `["del","layer"]`) {
		t.Fatal(got)
	}
	unknown, _ := s.Open(SurfaceOptions{ID: "unknown", Adopt: true})
	w.Reset()
	if err := unknown.Render(Main(text("a", "fresh"))); err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(w.String(), `"ops":[["del","main"],["add","main"`) {
		t.Fatal(w.String())
	}
}

func TestCloseRestoresAndClosesAfterFrameFailure(t *testing.T) {
	w := new(faultWriter)
	s := memorySession(w)
	restored := false
	s.restore = func() error { restored = true; return nil }
	sf, _ := s.Open(SurfaceOptions{})
	for _, v := range []string{"one", "two", "three"} {
		_ = sf.Render(Main(text("a", v)))
	}
	s.out = &failFrameWriter{out: w}
	w.Reset()
	if err := s.Close(); !errors.Is(err, errWrite) {
		t.Fatal(err)
	}
	if !restored || !strings.Contains(w.String(), "\x1b_tsp;x;") {
		t.Fatalf("restored=%v output=%q", restored, w.String())
	}
}
func TestCloseRestoresAfterModeResetFailure(t *testing.T) {
	w := new(faultWriter)
	s := memorySession(w)
	restored := false
	s.restore = func() error { restored = true; return nil }
	s.modesArmed = true
	w.fail = true
	if err := s.Close(); !errors.Is(err, errWrite) {
		t.Fatal(err)
	}
	if !restored {
		t.Fatal("failed mode reset skipped restoration")
	}
}

type failFrameWriter struct{ out io.Writer }

func (w *failFrameWriter) Write(p []byte) (int, error) {
	if bytes.Contains(p, []byte("\x1b_tsp;f;")) {
		return 0, errWrite
	}
	return w.out.Write(p)
}

func TestHandshakeArrivalAndTrailingInput(t *testing.T) {
	for _, tc := range []struct {
		name, stream string
		supported    bool
	}{
		{"DA1First", "\x1b[?62c" + helloReply, false},
		{"HelloFirst", helloReply + "z\x1b_tsp;e;{\"ev\":\"resize\",\"cols\":77}\x1b\\", true},
	} {
		t.Run(tc.name, func(t *testing.T) {
			ft, opts := newTerm(t)
			type result struct {
				s   *Session
				err error
			}
			ch := make(chan result, 1)
			go func() { s, err := Connect(context.Background(), opts); ch <- result{s, err} }()
			ft.expect(VerbQuery)
			ft.send(tc.stream)
			r := <-ch
			if !tc.supported {
				if !errors.Is(r.err, ErrUnsupported) {
					t.Fatal(r.err)
				}
				return
			}
			if r.err != nil {
				t.Fatal(r.err)
			}
			defer r.s.Close()
			ctx, cancel := context.WithTimeout(context.Background(), time.Second)
			defer cancel()
			in, err := r.s.Next(ctx)
			if err != nil || in.(Key).Text != "z" {
				t.Fatal(in, err)
			}
			in, err = r.s.Next(ctx)
			if err != nil || in.(*Resize).Cols != 77 || r.s.Caps().Cols != 77 {
				t.Fatal(in, err)
			}
		})
	}
}

func TestInputBytesPrecedeEOF(t *testing.T) {
	s := memorySession(io.Discard)
	chunks := make(chan readResult, 2)
	chunks <- readResult{data: []byte("abc")}
	chunks <- readResult{err: io.EOF}
	go s.process(chunks)
	for _, want := range []string{"a", "b", "c"} {
		in, err := s.Next(context.Background())
		if err != nil || in.(Key).Text != want {
			t.Fatal(in, err)
		}
	}
	if _, err := s.Next(context.Background()); !errors.Is(err, io.EOF) {
		t.Fatal(err)
	}
}

func TestCancelledBlobReplyKeepsOwnership(t *testing.T) {
	ft, opts := newTerm(t)
	s := connect(t, ft, opts)
	ctx, cancel := context.WithCancel(context.Background())
	first := make(chan error, 1)
	go func() { _, err := s.HaveBlobs(ctx, []string{"a"}); first <- err }()
	ft.expect(VerbQuery)
	cancel()
	if err := <-first; !errors.Is(err, context.Canceled) {
		t.Fatal(err)
	}
	second := make(chan []string, 1)
	go func() {
		have, err := s.HaveBlobs(context.Background(), []string{"b"})
		if err != nil {
			t.Error(err)
		}
		second <- have
	}()
	ft.expect(VerbQuery)
	ft.send("\x1b_tsp;r;{\"r\":\"blobs\",\"have\":[\"a\"]}\x1b\\\x1b_tsp;r;{\"r\":\"blobs\",\"have\":[\"b\"]}\x1b\\")
	if have := <-second; len(have) != 1 || have[0] != "b" {
		t.Fatal(have)
	}
}

func TestAskReturnsHandledSubmit(t *testing.T) {
	ft, opts := newTerm(t)
	s := connect(t, ft, opts)
	called := false
	button := Node{Kind: KindEl, Props: map[string]any{"tag": "button", "actions": map[string]any{"click": "submit"}}, Handlers: Handlers{OnClick: func(*Action) { called = true }}}
	done := make(chan *Answer, 1)
	go func() {
		ans, err := askOn(context.Background(), s, Main(button), AskOptions{Submit: "submit"})
		if err != nil {
			t.Error(err)
		}
		done <- ans
	}()
	ft.expect(VerbOpen)
	ft.expect(VerbFrame)
	ft.event(`{"ev":"action","sf":"s1","id":"main.0","act":"submit"}`)
	if ans := <-done; ans == nil || !called {
		t.Fatal(ans, called)
	}
}

func TestSplitClickAndDoubleClickActions(t *testing.T) {
	for _, double := range []bool{false, true} {
		called := false
		b := bound{click: "sort=name", dblclick: "sort=date"}
		if double {
			b.OnDblClick = func(*Action) { called = true }
		} else {
			b.OnClick = func(*Action) { called = true }
		}
		value := "name"
		if double {
			value = "date"
		}
		ev, _ := DecodeEvent([]byte(`{"ev":"action","act":"sort","value":"` + value + `"}`))
		if !b.route(ev) || !called {
			t.Fatal("split action not routed")
		}
	}
}

func TestIdleFlushIndependentOfNextDeadline(t *testing.T) {
	s := memorySession(io.Discard)
	chunks := make(chan readResult, 1)
	go s.process(chunks)
	defer close(s.done)
	chunks <- readResult{data: []byte("\x1b")}
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Millisecond)
	defer cancel()
	if _, err := s.Next(ctx); !errors.Is(err, context.DeadlineExceeded) {
		t.Fatal(err)
	}
	ctx2, cancel2 := context.WithTimeout(context.Background(), time.Second)
	defer cancel2()
	in, err := s.Next(ctx2)
	if err != nil || in.(Key).Name != "escape" {
		t.Fatal(in, err)
	}
	s.push(Key{Name: "q"})
	expired, cancel3 := context.WithCancel(context.Background())
	cancel3()
	if in, err := s.Next(expired); err != nil || in.(Key).Name != "q" {
		t.Fatal(in, err)
	}
}

func TestRawSetupFailureRollsBack(t *testing.T) {
	_, opts := newTerm(t)
	restored := false
	opts.MakeRaw = func() (func() error, error) { return func() error { restored = true; return nil }, errWrite }
	if _, err := Connect(context.Background(), opts); !errors.Is(err, ErrUnsupported) {
		t.Fatal(err)
	}
	if !restored {
		t.Fatal("partial raw setup not restored")
	}
}

func TestCloseJoinsCustomReader(t *testing.T) {
	ft, opts := newTerm(t)
	s := connect(t, ft, opts)
	if err := s.Close(); err != nil {
		t.Fatal(err)
	}
	select {
	case <-s.readerDone:
	default:
		t.Fatal("reader still running")
	}
	if _, err := opts.In.Read(make([]byte, 1)); !errors.Is(err, io.ErrClosedPipe) {
		t.Fatal(err)
	}
}
