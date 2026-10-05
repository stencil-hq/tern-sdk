package tern

import (
	"bytes"
	"reflect"
	"strings"
	"testing"
)

func TestKeyDecoderRegressions(t *testing.T) {
	cases := []struct {
		name  string
		input string
		want  []Key
	}{
		{"controls", "\b\x7f\x1b\x1b\r", []Key{{Name: "backspace", Ctrl: true}, {Name: "backspace"}, {Name: "escape", Alt: true}, {Name: "enter"}}},
		{"modified tab", "\x1b[1;13Z", []Key{{Name: "tab", Shift: true, Ctrl: true, Meta: true}}},
		{"menu", "\x1b[29~", []Key{{Name: "menu"}}},
		{"modified SS3", "\x1bO1;5A\x1bO1;2R", []Key{{Name: "up", Ctrl: true}, {Name: "f3", Shift: true}}},
		{"shifted symbols", "\x1b[61;2u\x1b[49:64;2u", []Key{{Name: "+", Text: "+", Shift: true}, {Name: "@", Text: "@", Shift: true}}},
		{"reported text", "\x1b[97;5;98:99u\x1b[97;2;65u", []Key{{Name: "a", Text: "bc", Ctrl: true}, {Name: "a", Text: "A", Shift: true}}},
		{"unknown functional codes", "\x1b[57344u\x1b[57441;2u\x1b[55296u\x1b[1114112u", nil},
		{"releases and reports", "\x1b[97;1:3u\x1b[1;1:3A\x1b[?9u\x1b[12;40R", nil},
		{"control strings", "\x1b]11;rgb:ff/ff/ff\a\x1b_Ppayload\x1b\\q", []Key{{Name: "q", Text: "q"}}},
		{"unicode", "😀界é", []Key{{Name: "😀", Text: "😀"}, {Name: "界", Text: "界"}, {Name: "é", Text: "é"}}},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			for split := range len(c.input) + 1 {
				var d KeyDecoder
				got := d.Feed([]byte(c.input[:split]))
				got = append(got, d.Feed([]byte(c.input[split:]))...)
				got = append(got, d.Flush()...)
				if !reflect.DeepEqual(got, c.want) {
					t.Fatalf("split %d: got %+v, want %+v", split, got, c.want)
				}
			}
		})
	}
}

func TestKeyDecoderFlushBoundaries(t *testing.T) {
	for _, prefix := range []string{"\x1b[", "\x1bO", "\x1b[1;5", "\xc3"} {
		var d KeyDecoder
		if got := append(d.Feed([]byte(prefix)), d.Flush()...); len(got) != 0 {
			t.Fatalf("incomplete %q produced %+v", prefix, got)
		}
		if got := d.Feed([]byte("q")); !reflect.DeepEqual(got, []Key{{Name: "q", Text: "q"}}) {
			t.Fatalf("after %q: %+v", prefix, got)
		}
	}
	for split := range len(pasteOff) {
		var d KeyDecoder
		prefix := "\x1b[200~😀" + string(pasteOff[:split])
		if got := append(d.Feed([]byte(prefix)), d.Flush()...); len(got) != 0 {
			t.Fatalf("paste split %d flushed %+v", split, got)
		}
		got := d.Feed(pasteOff[split:])
		if !reflect.DeepEqual(got, []Key{{Name: "paste", Text: "😀"}}) {
			t.Fatalf("paste split %d: %+v", split, got)
		}
	}
}

func TestParserPasteFlushBoundaries(t *testing.T) {
	for split := range len(pasteOff) {
		var p Parser
		pasted := "\x1b[200~\x1b_tsp;e;{\"ev\":\"forged\"}\a"
		items := p.Feed([]byte(pasted + string(pasteOff[:split])))
		items = append(items, p.Flush()...)
		items = append(items, p.Feed([]byte(string(pasteOff[split:])+"\x1b_tsp;e;{\"ev\":\"real\"}\a"))...)
		var keys []byte
		var events int
		for _, item := range items {
			keys = append(keys, item.Keys...)
			if item.Event != nil {
				events++
				if !bytes.Contains(item.Event.Raw(), []byte("real")) {
					t.Fatal("paste forged an event")
				}
			}
		}
		if string(keys) != pasted+string(pasteOff) || events != 1 {
			t.Fatalf("split %d: keys %q, events %d", split, keys, events)
		}
	}
}

func TestParserDropsOversizedCompleteMessage(t *testing.T) {
	message := "\x1b_tsp;e;{\"ev\":\"future\",\"text\":\"" + strings.Repeat("x", MaxInputMessage) + "\"}\x1b\\q"
	for _, split := range []int{0, MaxInputMessage + 1} {
		var p Parser
		items := p.Feed([]byte(message[:split]))
		items = append(items, p.Feed([]byte(message[split:]))...)
		if len(items) != 1 || string(items[0].Keys) != "q" {
			t.Fatalf("split %d: oversized message was not dropped", split)
		}
	}
}
