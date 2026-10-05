package tern

import (
	"bytes"
	"strconv"
	"strings"
	"unicode"
	"unicode/utf8"
)

// Key is one key press, named as in Tern's plugin API: a lowercase
// character ("a", "é", "+"), "space", a named key ("enter", "up", "f5", …)
// or "paste" with the pasted text.
type Key struct {
	// Name is the key's name.
	Name string `json:"name"`
	// Text is what the key types, when it types something.
	Text string `json:"text,omitempty"`
	// Ctrl, Alt, Shift and Meta are the modifiers held (Meta is Cmd/Super).
	Ctrl  bool `json:"ctrl,omitempty"`
	Alt   bool `json:"alt,omitempty"`
	Shift bool `json:"shift,omitempty"`
	Meta  bool `json:"meta,omitempty"`
}

func (Key) isInput() {}

// String is the key as a chord such as "ctrl+shift+a" or "enter".
func (k Key) String() string {
	var b strings.Builder
	for _, m := range []struct {
		on   bool
		name string
	}{{k.Ctrl, "ctrl"}, {k.Alt, "alt"}, {k.Shift, "shift"}, {k.Meta, "meta"}} {
		if m.on {
			b.WriteString(m.name)
			b.WriteByte('+')
		}
	}
	b.WriteString(k.Name)
	return b.String()
}

// Is reports whether k is the chord written as in String ("ctrl+c", "escape").
func (k Key) Is(chord string) bool { return k.String() == chord }

// KeyDecoder turns key bytes (what the input parser passes through) into
// keys: legacy xterm input, kitty CSI u keys and bracketed paste. Unknown
// sequences are dropped. It is not safe for concurrent use.
type KeyDecoder struct {
	buf   []byte
	paste bool
	keys  []Key
}

// Feed decodes data and returns the keys it completed.
func (d *KeyDecoder) Feed(data []byte) []Key {
	d.buf = append(d.buf, data...)
	d.run(false)
	return d.take()
}

// Flush decodes what is held: a lone ESC becomes Escape, a partial
// sequence is dropped. A paste in progress is kept.
func (d *KeyDecoder) Flush() []Key {
	d.run(true)
	return d.take()
}

// take returns and clears the decoded keys.
func (d *KeyDecoder) take() []Key {
	out := d.keys
	d.keys = nil
	return out
}

// run decodes as much of buf as it can.
func (d *KeyDecoder) run(flush bool) {
	for len(d.buf) > 0 {
		if d.paste {
			i := bytes.Index(d.buf, pasteOff)
			if i < 0 {
				return
			}
			d.keys = append(d.keys, Key{Name: "paste", Text: string(d.buf[:i])})
			d.buf = d.buf[i+len(pasteOff):]
			d.paste = false
			continue
		}
		key, n, ok := d.one(d.buf, flush)
		if n == 0 {
			if flush {
				d.buf = nil
			}
			return
		}
		d.buf = d.buf[n:]
		if ok {
			d.keys = append(d.keys, key)
		}
	}
	d.buf = nil
}

// one decodes the key at the start of b: n is the bytes it takes (0 when
// it needs more), ok false when they are dropped.
func (d *KeyDecoder) one(b []byte, flush bool) (Key, int, bool) {
	c := b[0]
	if c == 0x1b {
		return d.escape(b, flush)
	}
	if c < 0x20 || c == 0x7f {
		return control(c), 1, true
	}
	if !utf8.FullRune(b) {
		if flush {
			return Key{}, len(b), false
		}
		return Key{}, 0, false
	}
	r, n := utf8.DecodeRune(b)
	if r == utf8.RuneError && n <= 1 {
		return Key{}, 1, false
	}
	return char(r), n, true
}

// control is the key a C0 control byte or DEL types.
func control(c byte) Key {
	switch c {
	case '\r', '\n':
		return Key{Name: "enter"}
	case '\t':
		return Key{Name: "tab"}
	case 0x08:
		return Key{Name: "backspace", Ctrl: true}
	case 0x7f:
		return Key{Name: "backspace"}
	case 0x1b:
		return Key{Name: "escape"}
	case 0x00:
		return Key{Name: "space", Ctrl: true}
	}
	if c >= 0x01 && c <= 0x1a {
		return Key{Name: string(rune('a' + c - 1)), Ctrl: true}
	}
	return Key{Name: string("\\]^_"[c-0x1c]), Ctrl: true}
}

// char is the key that types rune r.
func char(r rune) Key {
	if r == ' ' {
		return Key{Name: "space", Text: " "}
	}
	lower := unicode.ToLower(r)
	return Key{Name: string(lower), Text: string(r), Shift: lower != r}
}

// escape decodes a sequence starting with ESC.
func (d *KeyDecoder) escape(b []byte, flush bool) (Key, int, bool) {
	if len(b) == 1 {
		if flush {
			return Key{Name: "escape"}, 1, true
		}
		return Key{}, 0, false
	}
	switch b[1] {
	case 0x1b:
		return alt(Key{Name: "escape"}), 2, true
	case '_', ']', 'P', '^', 'X':
		for i := 2; i < len(b); i++ {
			if b[i] == 0x07 || (b[i] == '\\' && b[i-1] == 0x1b) {
				return Key{}, i + 1, false
			}
		}
		return Key{}, 0, false
	case '[':
		return d.csi(b, flush)
	case 'O':
		return d.csi(b, flush)
	}
	key, n, ok := d.one(b[1:], flush)
	if n == 0 {
		return Key{}, 0, false
	}
	return alt(key), n + 1, ok
}

// alt is k with Alt held, which types nothing.
func alt(k Key) Key {
	k.Alt = true
	k.Text = ""
	return k
}

// ss3Keys names the keys sent as ESC O <final>.
var ss3Keys = map[byte]string{
	'A': "up", 'B': "down", 'C': "right", 'D': "left", 'H': "home", 'F': "end", 'E': "begin",
	'P': "f1", 'Q': "f2", 'R': "f3", 'S': "f4", 'M': "enter",
}

// csiFinalKeys names the keys sent as CSI [1;mods] <final>.
var csiFinalKeys = map[byte]string{
	'A': "up", 'B': "down", 'C': "right", 'D': "left", 'H': "home", 'F': "end", 'E': "begin",
	'P': "f1", 'Q': "f2", 'S': "f4",
}

// tildeKeys names the keys sent as CSI <n>[;mods] ~.
var tildeKeys = map[int]string{
	1: "home", 2: "insert", 3: "delete", 4: "end", 5: "page_up", 6: "page_down", 7: "home", 8: "end",
	11: "f1", 12: "f2", 13: "f3", 14: "f4", 15: "f5", 17: "f6", 18: "f7", 19: "f8", 20: "f9",
	21: "f10", 23: "f11", 24: "f12", 29: "menu",
}

// kittyKeys names the kitty functional key codes that are not characters.
var kittyKeys = map[int]string{
	27: "escape", 13: "enter", 9: "tab", 127: "backspace",
	57358: "caps_lock", 57359: "scroll_lock", 57360: "num_lock", 57361: "print_screen",
	57362: "pause", 57363: "menu", 57414: "enter", 57417: "left", 57418: "right", 57419: "up",
	57420: "down", 57421: "page_up", 57422: "page_down", 57423: "home", 57424: "end",
	57425: "insert", 57426: "delete", 57427: "begin",
}

// kittyKeypad maps kitty keypad codes to the characters they stand for.
var kittyKeypad = map[int]rune{
	57409: '.', 57410: '/', 57411: '*', 57412: '-', 57413: '+', 57415: '=', 57416: ',',
}

// csi decodes a CSI key sequence.
func (d *KeyDecoder) csi(b []byte, flush bool) (Key, int, bool) {
	i := 2
	for i < len(b) && b[i] >= 0x30 && b[i] <= 0x3f {
		i++
	}
	for i < len(b) && b[i] >= 0x20 && b[i] <= 0x2f {
		i++
	}
	if i >= len(b) {
		if flush {
			return Key{}, len(b), false
		}
		return Key{}, 0, false
	}
	final := b[i]
	n := i + 1
	if final < 0x40 || final > 0x7e {
		return Key{}, i, false
	}
	params := string(b[2:i])
	if params != "" && (params[0] < '0' || params[0] > ';') {
		return Key{}, n, false
	}
	fields := strings.Split(params, ";")
	num := func(i int) int {
		if i >= len(fields) {
			return 0
		}
		v, _ := strconv.Atoi(strings.SplitN(fields[i], ":", 2)[0])
		return v
	}
	if released(fields) {
		return Key{}, n, false
	}
	if b[1] == 'O' {
		name, ok := ss3Keys[final]
		return mods(Key{Name: name}, num(1)), n, ok
	}
	switch {
	case final == '~':
		code := num(0)
		if code == 200 {
			d.paste = true
			return Key{}, n, false
		}
		name, ok := tildeKeys[code]
		return mods(Key{Name: name}, num(1)), n, ok
	case final == 'Z':
		return mods(Key{Name: "tab", Shift: true}, num(1)), n, true
	case final == 'u':
		key, ok := kitty(fields, num(1))
		return key, n, ok
	}
	name, ok := csiFinalKeys[final]
	return mods(Key{Name: name}, num(1)), n, ok
}

// released reports whether a kitty key's event type is a release.
func released(fields []string) bool {
	if len(fields) < 2 {
		return false
	}
	sub := strings.Split(fields[1], ":")
	return len(sub) > 1 && sub[1] == "3"
}

// kitty decodes a CSI <code>[;mods] u key.
func kitty(fields []string, m int) (Key, bool) {
	codes := strings.Split(fields[0], ":")
	code, err := strconv.Atoi(codes[0])
	if err != nil || code < 0 || code > utf8.MaxRune || !utf8.ValidRune(rune(code)) {
		return Key{}, false
	}
	var text strings.Builder
	if len(fields) > 2 {
		for _, value := range strings.Split(fields[2], ":") {
			n, err := strconv.Atoi(value)
			if err == nil && n >= 0 && n <= utf8.MaxRune && utf8.ValidRune(rune(n)) {
				text.WriteRune(rune(n))
			}
		}
	}
	if name, ok := kittyKeys[code]; ok {
		return mods(Key{Name: name, Text: text.String()}, m), true
	}
	r := rune(code)
	switch {
	case code >= 57376 && code <= 57398:
		return mods(Key{Name: "f" + strconv.Itoa(13+code-57376)}, m), true
	case code >= 57399 && code <= 57408:
		r = rune('0' + code - 57399)
	case kittyKeypad[code] != 0:
		r = kittyKeypad[code]
	case code >= 57344:
		return Key{}, false
	}
	bits := max(m-1, 0)
	if text.Len() == 0 && bits&62 == 0 {
		typed := r
		if bits&1 != 0 {
			typed = usShift(r)
			if len(codes) > 1 {
				n, err := strconv.Atoi(codes[1])
				if err == nil && n >= 0 && n <= utf8.MaxRune && utf8.ValidRune(rune(n)) {
					typed = rune(n)
				}
			}
		}
		text.WriteRune(typed)
	}
	k := char(r)
	k.Text = text.String()
	if utf8.RuneCountInString(k.Text) == 1 {
		typed, _ := utf8.DecodeRuneInString(k.Text)
		k = char(typed)
	}
	return mods(k, m), true
}

// usShift is the plugin decoder's fallback when kitty omits a shifted code.
func usShift(r rune) rune {
	if r >= 'a' && r <= 'z' {
		return r - 'a' + 'A'
	}
	const base = "1234567890-=[]\\;',./`"
	const shifted = "!@#$%^&*()_+{}|:\"<>?~"
	if i := strings.IndexRune(base, r); i >= 0 {
		return rune(shifted[i])
	}
	return r
}

// mods applies an xterm/kitty modifier parameter (1 + bits) to k.
func mods(k Key, m int) Key {
	if m <= 1 {
		return k
	}
	bits := m - 1
	k.Shift = k.Shift || bits&1 != 0
	k.Alt = k.Alt || bits&2 != 0
	k.Ctrl = k.Ctrl || bits&4 != 0
	k.Meta = k.Meta || bits&8 != 0 || bits&32 != 0
	return k
}
