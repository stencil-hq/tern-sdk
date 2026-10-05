package tern

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"io"
	"strings"
	"sync"
	"sync/atomic"
	"testing"
	"time"
)

// sent is one message the program wrote.
type sent struct {
	verb string
	body string
}

// fakeTerm is a scripted terminal on the other end of a session's pipes.
type fakeTerm struct {
	t      *testing.T
	out    *io.PipeReader
	msgs   chan sent
	input  chan string
	mu     sync.Mutex
	raw    bytes.Buffer
	rawOn  atomic.Int32
	order  []string
	orderM sync.Mutex
}

// newTerm starts a terminal and returns it with Options wired to it.
func newTerm(t *testing.T) (*fakeTerm, Options) {
	t.Helper()
	for _, k := range []string{"TERN_TSP", "TMUX", "STY", "ZELLIJ", "TERN_TSP_RECORD"} {
		t.Setenv(k, "")
	}
	inR, inW := io.Pipe()
	outR, outW := io.Pipe()
	ft := &fakeTerm{t: t, out: outR, msgs: make(chan sent, 256), input: make(chan string, 64)}
	go ft.read()
	go func() {
		for s := range ft.input {
			if _, err := inW.Write([]byte(s)); err != nil {
				return
			}
		}
	}()
	t.Cleanup(func() {
		inW.Close()
		outR.Close()
	})
	opts := Options{
		App: "test", In: inR, Out: outW, Timeout: 500 * time.Millisecond,
		MakeRaw: func() (func() error, error) {
			ft.rawOn.Add(1)
			ft.note("raw")
			return func() error {
				ft.rawOn.Add(-1)
				ft.note("restore")
				return nil
			}, nil
		},
	}
	return ft, opts
}

// note records an event in order.
func (ft *fakeTerm) note(s string) {
	ft.orderM.Lock()
	ft.order = append(ft.order, s)
	ft.orderM.Unlock()
}

// read parses the program's output into messages and raw bytes.
func (ft *fakeTerm) read() {
	var buf []byte
	chunk := make([]byte, 4096)
	for {
		n, err := ft.out.Read(chunk)
		buf = append(buf, chunk[:n]...)
		for {
			i := bytes.Index(buf, []byte("\x1b_tsp;"))
			if i < 0 {
				ft.rawBytes(buf)
				buf = nil
				break
			}
			ft.rawBytes(buf[:i])
			end := bytes.Index(buf[i:], []byte("\x1b\\"))
			if end < 0 {
				buf = buf[i:]
				break
			}
			m, _ := SplitMessage(buf[i+6 : i+end])
			ft.note("msg " + m.Verb)
			ft.msgs <- sent{m.Verb, string(m.Body)}
			buf = buf[i+end+2:]
		}
		if err != nil {
			close(ft.msgs)
			return
		}
	}
}

// rawBytes keeps output that is not TSP.
func (ft *fakeTerm) rawBytes(b []byte) {
	if len(b) == 0 {
		return
	}
	ft.mu.Lock()
	ft.raw.Write(b)
	ft.mu.Unlock()
	ft.note("raw " + string(b))
}

// expect waits for the next message, which must be verb.
func (ft *fakeTerm) expect(verb string) map[string]any {
	ft.t.Helper()
	select {
	case m, ok := <-ft.msgs:
		if !ok {
			ft.t.Fatalf("output closed, want %s", verb)
		}
		if m.verb != verb {
			ft.t.Fatalf("got %s %s, want %s", m.verb, m.body, verb)
		}
		var v map[string]any
		if err := json.Unmarshal([]byte(m.body), &v); err != nil {
			ft.t.Fatalf("%s body %q: %v", verb, m.body, err)
		}
		return v
	case <-time.After(2 * time.Second):
		ft.t.Fatalf("timed out waiting for %s", verb)
	}
	return nil
}

// quiet asserts that no message arrives for a while.
func (ft *fakeTerm) quiet() {
	ft.t.Helper()
	select {
	case m := <-ft.msgs:
		ft.t.Fatalf("unexpected %s %s", m.verb, m.body)
	case <-time.After(80 * time.Millisecond):
	}
}

// send writes input to the program, in order, without waiting for it to be read.
func (ft *fakeTerm) send(s string) { ft.input <- s }

// event writes an e message.
func (ft *fakeTerm) event(body string) { ft.send("\x1b_tsp;e;" + body + "\x1b\\") }

// rawOutput is the non-TSP output so far.
func (ft *fakeTerm) rawOutput() string {
	ft.mu.Lock()
	defer ft.mu.Unlock()
	return ft.raw.String()
}

const helloReply = "\x1b_tsp;r;{\"r\":\"hello\",\"v\":1,\"term\":\"tern\",\"ver\":\"0.4.3\",\"kinds\":[\"col\",\"text\",\"el\"],\"features\":[\"flow\",\"styles\"],\"apc\":65536,\"credits\":2,\"cols\":120,\"cell\":{\"w\":8,\"h\":17},\"dark\":true,\"reduceMotion\":false}\x1b\\\x1b[?62;52;c"

// connect connects a session to ft, answering hello.
func connect(t *testing.T, ft *fakeTerm, opts Options) *Session {
	t.Helper()
	type result struct {
		s   *Session
		err error
	}
	done := make(chan result, 1)
	go func() {
		s, err := Connect(context.Background(), opts)
		done <- result{s, err}
	}()
	hello := ft.expect(VerbQuery)
	if hello["q"] != "hello" || hello["app"] != "test" {
		t.Fatalf("hello %v", hello)
	}
	ft.send(helloReply)
	r := <-done
	if r.err != nil {
		t.Fatal(r.err)
	}
	t.Cleanup(func() { _ = r.s.Close() })
	return r.s
}

func TestHelloHour12(t *testing.T) {
	var r HelloReply
	if err := json.Unmarshal([]byte(`{"r":"hello","v":1,"hour12":true}`), &r); err != nil {
		t.Fatal(err)
	}
	if !capsOf(&r).Hour12 {
		t.Fatal("hour12 dropped")
	}
	var older HelloReply
	if err := json.Unmarshal([]byte(`{"r":"hello","v":1}`), &older); err != nil {
		t.Fatal(err)
	}
	if capsOf(&older).Hour12 {
		t.Fatal("older terminals read as 12-hour")
	}
}

func TestConnectHandshake(t *testing.T) {
	ft, opts := newTerm(t)
	opts.Features = []string{FeatureEdit, FeatureSend}
	done := make(chan *Session, 1)
	go func() {
		s, err := Connect(context.Background(), opts)
		if err != nil {
			t.Error(err)
		}
		done <- s
	}()
	hello := ft.expect(VerbQuery)
	if ft.rawOn.Load() != 1 {
		t.Fatal("hello written before raw mode")
	}
	want := map[string]any{"q": "hello", "v": []any{1.0}, "app": "test", "features": []any{"edit", "send"}}
	if !jsonEqual(hello, want) {
		t.Fatalf("hello %v", hello)
	}
	ft.send(helloReply)
	s := <-done
	caps := s.Caps()
	if caps.Cols != 120 || !caps.Dark || caps.Credits != 2 || !caps.HasFeature("flow") || !caps.HasKind("el") || caps.Hour12 {
		t.Fatalf("caps %+v", caps)
	}
	if err := s.Close(); err != nil {
		t.Fatal(err)
	}
	if ft.rawOn.Load() != 0 {
		t.Fatal("tty not restored")
	}
	out := ft.rawOutput()
	if !strings.Contains(out, da1Request+pasteModeOn+kittyPush) || !strings.HasSuffix(out, kittyPop+pasteModeOff) {
		t.Fatalf("modes %q", out)
	}
}

// jsonEqual compares values after a JSON round trip.
func jsonEqual(a, b any) bool {
	ja, _ := json.Marshal(a)
	jb, _ := json.Marshal(b)
	var va, vb any
	_ = json.Unmarshal(ja, &va)
	_ = json.Unmarshal(jb, &vb)
	ja, _ = json.Marshal(va)
	jb, _ = json.Marshal(vb)
	return string(ja) == string(jb)
}

func TestConnectDA1FirstIsUnsupported(t *testing.T) {
	ft, opts := newTerm(t)
	errs := make(chan error, 1)
	go func() {
		_, err := Connect(context.Background(), opts)
		errs <- err
	}()
	ft.expect(VerbQuery)
	ft.send("\x1b[?62;22c")
	if err := <-errs; !errors.Is(err, ErrUnsupported) {
		t.Fatalf("err %v", err)
	}
	if ft.rawOn.Load() != 0 {
		t.Fatal("tty not restored")
	}
}

func TestConnectTimeoutIsUnsupported(t *testing.T) {
	ft, opts := newTerm(t)
	opts.Timeout = 60 * time.Millisecond
	start := time.Now()
	_, err := Connect(context.Background(), opts)
	if !errors.Is(err, ErrUnsupported) {
		t.Fatalf("err %v", err)
	}
	if time.Since(start) < 60*time.Millisecond {
		t.Fatal("gave up before the timeout")
	}
	if ft.rawOn.Load() != 0 {
		t.Fatal("tty not restored")
	}
}

func TestConnectOptOuts(t *testing.T) {
	for _, env := range []string{"TERN_TSP=0", "TMUX=/tmp/tmux", "STY=1.pts", "ZELLIJ=0"} {
		t.Run(env, func(t *testing.T) {
			ft, opts := newTerm(t)
			k, v, _ := strings.Cut(env, "=")
			t.Setenv(k, v)
			if _, err := Connect(context.Background(), opts); !errors.Is(err, ErrUnsupported) {
				t.Fatalf("err %v", err)
			}
			ft.orderM.Lock()
			defer ft.orderM.Unlock()
			if len(ft.order) != 0 {
				t.Fatalf("touched the terminal: %v", ft.order)
			}
		})
	}
}

// text is a text node.
func text(key, s string) Node {
	return Node{Kind: KindText, Props: map[string]any{"key": key, "text": s}}
}

func TestCreditsCoalesceOnAck(t *testing.T) {
	ft, opts := newTerm(t)
	s := connect(t, ft, opts)
	sf, err := s.Open(SurfaceOptions{Mode: Flow})
	if err != nil {
		t.Fatal(err)
	}
	open := ft.expect(VerbOpen)
	if open["id"] != "s1" || open["mode"] != "flow" || open["listen"] != nil {
		t.Fatalf("open %v", open)
	}
	view := func(texts ...string) View {
		var ns Nodes
		for i, s := range texts {
			ns = append(ns, text(string(rune('a'+i)), s))
		}
		return View{Main: ns}
	}
	for _, v := range []View{view("1"), view("1", "2"), view("1", "2", "3"), view("x", "2", "3", "4")} {
		if err := sf.Render(v); err != nil {
			t.Fatal(err)
		}
	}
	if f := ft.expect(VerbFrame); f["s"] != 1.0 {
		t.Fatalf("frame %v", f)
	}
	if f := ft.expect(VerbFrame); f["s"] != 2.0 {
		t.Fatalf("frame %v", f)
	}
	ft.quiet()
	ft.event(`{"ev":"ack","sf":"s1","s":1}`)
	f := ft.expect(VerbFrame)
	want := map[string]any{"sf": "s1", "s": 3, "ops": []any{
		[]any{"add", "main.d", "main", nil, map[string]any{"id": "main.d", "k": "text", "p": map[string]any{"key": "d", "text": "4"}}},
		[]any{"add", "main.c", "main", "main.d", map[string]any{"id": "main.c", "k": "text", "p": map[string]any{"key": "c", "text": "3"}}},
		[]any{"text", "main.a", "replace", "x"},
	}}
	if !jsonEqual(f, want) {
		t.Fatalf("coalesced frame %v", f)
	}
	ft.quiet()
	ft.event(`{"ev":"ack","sf":"s1","s":3}`)
	if err := sf.Render(view("x", "2", "3", "4")); err != nil {
		t.Fatal(err)
	}
	if err := sf.Focus("main.a"); err != nil {
		t.Fatal(err)
	}
	if f := ft.expect(VerbFrame); !jsonEqual(f["ops"], []any{[]any{"focus", "main.a"}}) || f["s"] != 4.0 {
		t.Fatalf("an unchanged view with a queued op %v", f)
	}
}

func TestListenFalseSendsAtOnce(t *testing.T) {
	ft, opts := newTerm(t)
	s := connect(t, ft, opts)
	sf, err := s.Open(SurfaceOptions{ID: "ls", Mode: Flow, NoListen: true})
	if err != nil {
		t.Fatal(err)
	}
	if open := ft.expect(VerbOpen); open["listen"] != false {
		t.Fatalf("open %v", open)
	}
	for i, s := range []string{"a", "ab", "abc", "abcd"} {
		if err := sf.Render(View{Main: Nodes{Node{Kind: KindMd, Props: map[string]any{"text": s}}}}); err != nil {
			t.Fatal(err)
		}
		if f := ft.expect(VerbFrame); f["s"] != float64(i+1) {
			t.Fatalf("frame %v", f)
		}
	}
	if err := sf.Close(); err != nil {
		t.Fatal(err)
	}
	if x := ft.expect(VerbClose); x["id"] != "ls" || x["keep"] != true {
		t.Fatalf("close %v", x)
	}
}

func TestHandlersRouteEvents(t *testing.T) {
	ft, opts := newTerm(t)
	s := connect(t, ft, opts)
	sf, _ := s.Open(SurfaceOptions{})
	ft.expect(VerbOpen)
	var clicks, menus, changes int
	button := Node{Kind: KindEl, Props: map[string]any{"tag": "button", "text": "Go"}, Handlers: Handlers{
		OnClick: func(a *Action) { clicks++ },
		OnMenu:  map[string]func(*Action){"sort=name": func(a *Action) { menus++ }},
	}}
	box := Node{Kind: KindEl, Props: map[string]any{"key": "box", "tag": "input", "type": "checkbox"}, Handlers: Handlers{
		OnChange: func(c *Change) { changes++ },
	}}
	if err := sf.Render(View{Main: Nodes{button, box}}); err != nil {
		t.Fatal(err)
	}
	f := ft.expect(VerbFrame)
	add := f["ops"].([]any)[0].([]any)[4].(map[string]any)
	gotActions := add["c"].([]any)[0].(map[string]any)["p"].(map[string]any)["actions"]
	if !jsonEqual(gotActions, map[string]any{"click": "click", "menu": []any{"sort=name"}}) {
		t.Fatalf("actions %v", gotActions)
	}
	ft.event(`{"ev":"action","sf":"s1","id":"main.0","act":"click"}`)
	ft.event(`{"ev":"action","sf":"s1","id":"main.0","act":"sort","value":"name"}`)
	ft.event(`{"ev":"change","sf":"s1","id":"main.box","value":"on","checked":true}`)
	ft.event(`{"ev":"action","sf":"s1","id":"main.0","act":"other"}`)
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()
	in, err := s.Next(ctx)
	if err != nil {
		t.Fatal(err)
	}
	if a, ok := in.(*Action); !ok || a.Act != "other" {
		t.Fatalf("unhandled input %#v", in)
	}
	if clicks != 1 || menus != 1 || changes != 1 {
		t.Fatalf("clicks %d menus %d changes %d", clicks, menus, changes)
	}
	ft.send("q\x1b[A")
	in, err = s.Next(ctx)
	if k, ok := in.(Key); err != nil || !ok || k.Name != "q" {
		t.Fatalf("key %#v %v", in, err)
	}
	in, _ = s.Next(ctx)
	if k, ok := in.(Key); !ok || k.Name != "up" {
		t.Fatalf("key %#v", in)
	}
}

func TestResizeAndGone(t *testing.T) {
	ft, opts := newTerm(t)
	s := connect(t, ft, opts)
	sf, _ := s.Open(SurfaceOptions{})
	ft.expect(VerbOpen)
	ft.event(`{"ev":"resize","sf":"s1","cols":80,"cell":{"w":9,"h":18},"visible":true}`)
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()
	if in, err := s.Next(ctx); err != nil || in.(*Resize).Cols != 80 {
		t.Fatalf("resize %v %v", in, err)
	}
	if s.Caps().Cols != 80 {
		t.Fatal("cols not updated")
	}
	ft.event(`{"ev":"gone","sf":"s1","ids":["s1"]}`)
	if _, err := s.Next(ctx); err != nil {
		t.Fatal(err)
	}
	if err := sf.Render(View{Main: Nodes{}}); !errors.Is(err, ErrClosed) {
		t.Fatalf("render after gone: %v", err)
	}
}

func TestCloseSequence(t *testing.T) {
	ft, opts := newTerm(t)
	s := connect(t, ft, opts)
	sf, _ := s.Open(SurfaceOptions{})
	ft.expect(VerbOpen)
	gone, _ := s.Open(SurfaceOptions{Discard: true})
	ft.expect(VerbOpen)
	for i := range 3 {
		_ = sf.Render(View{Main: Nodes{text("t", strings.Repeat("x", i+1))}})
	}
	ft.expect(VerbFrame)
	ft.expect(VerbFrame)
	ft.quiet()
	if err := s.Close(); err != nil {
		t.Fatal(err)
	}
	if f := ft.expect(VerbFrame); f["s"] != 3.0 {
		t.Fatalf("pending frame %v", f)
	}
	if x := ft.expect(VerbClose); x["id"] != "s1" || x["keep"] != true {
		t.Fatalf("close %v", x)
	}
	if x := ft.expect(VerbClose); x["id"] != "s2" || x["keep"] != false {
		t.Fatalf("close %v", x)
	}
	ft.orderM.Lock()
	order := strings.Join(ft.order, "|")
	ft.orderM.Unlock()
	if !strings.HasSuffix(order, "msg x|msg x|raw "+kittyPop+pasteModeOff+"|restore") {
		t.Fatalf("order %s", order)
	}
	if _, err := s.Next(context.Background()); !errors.Is(err, ErrClosed) {
		t.Fatalf("next after close: %v", err)
	}
	if err := gone.Render(View{}); !errors.Is(err, ErrClosed) {
		t.Fatalf("render after close: %v", err)
	}
}

func TestBlobs(t *testing.T) {
	ft, opts := newTerm(t)
	s := connect(t, ft, opts)
	id, err := s.Blob([]byte("hello"), "text/plain")
	if err != nil || id != BlobID([]byte("hello")) {
		t.Fatal(id, err)
	}
	select {
	case m := <-ft.msgs:
		if m.verb != VerbBlob || m.body != "aGVsbG8=" {
			t.Fatalf("blob %v", m)
		}
	case <-time.After(time.Second):
		t.Fatal("no blob")
	}
	if _, err := s.Blob([]byte("hello"), "text/plain"); err != nil {
		t.Fatal(err)
	}
	ft.quiet()
	got := make(chan []string, 1)
	go func() {
		have, err := s.HaveBlobs(context.Background(), []string{id, "ff"})
		if err != nil {
			t.Error(err)
		}
		got <- have
	}()
	if q := ft.expect(VerbQuery); q["q"] != "blobs" {
		t.Fatalf("query %v", q)
	}
	ft.send("\x1b_tsp;r;{\"r\":\"blobs\",\"have\":[\"" + id + "\"]}\x1b\\")
	if have := <-got; len(have) != 1 || have[0] != id {
		t.Fatalf("have %v", have)
	}
}

func TestAsk(t *testing.T) {
	type result struct {
		a   *Answer
		err error
	}
	run := func(script func(*fakeTerm)) result {
		ft, opts := newTerm(t)
		done := make(chan result, 1)
		go func() {
			a, err := Ask(context.Background(), View{Main: Nodes{Node{Kind: KindEl, Props: map[string]any{"tag": "form"}}}},
				AskOptions{CSS: ".a{}", Options: opts})
			done <- result{a, err}
		}()
		ft.expect(VerbQuery)
		ft.send(helloReply)
		if o := ft.expect(VerbOpen); o["mode"] != "flow" {
			t.Fatalf("open %v", o)
		}
		ft.expect(VerbSheet)
		ft.expect(VerbFrame)
		script(ft)
		r := <-done
		if x := ft.expect(VerbClose); x["keep"] != true {
			t.Fatalf("close %v", x)
		}
		return r
	}
	r := run(func(ft *fakeTerm) {
		ft.event(`{"ev":"action","sf":"s1","id":"main.0","act":"cancel"}`)
		ft.event(`{"ev":"action","sf":"s1","id":"go","act":"submit","values":{"size":"l","notify":true,"tags":["a"]}}`)
	})
	if r.err != nil || r.a == nil || r.a.ID != "go" || r.a.Values.String("size") != "l" || !r.a.Values.Bool("notify") || r.a.Values.Strings("tags")[0] != "a" {
		t.Fatalf("answer %+v %v", r.a, r.err)
	}
	r = run(func(ft *fakeTerm) { ft.send("\x1b[27u") })
	if r.err != nil || r.a != nil {
		t.Fatalf("escape gave %+v %v", r.a, r.err)
	}
}

func TestPrint(t *testing.T) {
	ft, opts := newTerm(t)
	done := make(chan error, 1)
	go func() {
		done <- Print(context.Background(), View{Main: Nodes{text("a", "hi")}}, PrintOptions{CSS: ".x{}", Options: opts})
	}()
	ft.expect(VerbQuery)
	ft.send(helloReply)
	if o := ft.expect(VerbOpen); o["listen"] != false || o["mode"] != "flow" {
		t.Fatalf("open %v", o)
	}
	if sh := ft.expect(VerbSheet); sh["name"] != "main" || sh["css"] != ".x{}" {
		t.Fatalf("sheet %v", sh)
	}
	ft.expect(VerbFrame)
	if x := ft.expect(VerbClose); x["keep"] != true {
		t.Fatalf("close %v", x)
	}
	if err := <-done; err != nil {
		t.Fatal(err)
	}
}

func TestPrintFallback(t *testing.T) {
	t.Setenv("TERN_TSP", "0")
	var out bytes.Buffer
	err := Print(context.Background(), View{Main: Nodes{text("a", "hello")}}, PrintOptions{Options: Options{Out: &out}})
	if err != nil || out.String() != "hello\n" {
		t.Fatalf("%q %v", out.String(), err)
	}
	out.Reset()
	_ = Print(context.Background(), View{Main: Nodes{text("a", "hello")}}, PrintOptions{Fallback: "plain", Options: Options{Out: &out}})
	if out.String() != "plain\n" {
		t.Fatalf("fallback %q", out.String())
	}
	if _, err := Ask(context.Background(), View{}, AskOptions{}); !errors.Is(err, ErrUnsupported) {
		t.Fatalf("ask without TSP: %v", err)
	}
}
