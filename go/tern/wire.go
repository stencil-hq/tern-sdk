package tern

import (
	"bytes"
	"crypto/sha256"
	"encoding/base64"
	"encoding/hex"
	"encoding/json"
	"errors"
	"strconv"
)

// Protocol constants from the hello handshake.
const (
	// ProtocolVersion is the TSP version this SDK speaks.
	ProtocolVersion = 1
	// DefaultAPC is the largest body sent in one message before chunking,
	// until the hello reply says otherwise.
	DefaultAPC = 65536
	// DefaultCredits is how many frames may be unacknowledged, until the
	// hello reply says otherwise.
	DefaultCredits = 2
	// MaxBlob is the largest blob, decoded, that Tern accepts.
	MaxBlob = 16 << 20
)

// Node kinds of the TSP vocabulary.
const (
	KindCol       = "col"
	KindRow       = "row"
	KindCard      = "card"
	KindSection   = "section"
	KindRule      = "rule"
	KindSpacer    = "spacer"
	KindText      = "text"
	KindMd        = "md"
	KindCode      = "code"
	KindDiff      = "diff"
	KindAnsi      = "ansi"
	KindRows      = "rows"
	KindMath      = "math"
	KindKV        = "kv"
	KindTable     = "table"
	KindTree      = "tree"
	KindBadge     = "badge"
	KindKbd       = "kbd"
	KindIcon      = "icon"
	KindImage     = "image"
	KindList      = "list"
	KindItem      = "item"
	KindTabs      = "tabs"
	KindPicker    = "picker"
	KindSpinner   = "spinner"
	KindShimmer   = "shimmer"
	KindElapsed   = "elapsed"
	KindRate      = "rate"
	KindProgress  = "progress"
	KindMeter     = "meter"
	KindChart     = "chart"
	KindEffort    = "effort"
	KindEditor    = "editor"
	KindInput     = "input"
	KindStatus    = "status"
	KindSeg       = "seg"
	KindToast     = "toast"
	KindOverlay   = "overlay"
	KindTool      = "tool"
	KindAgent     = "agent"
	KindChecklist = "checklist"
	KindBlock     = "block"
	KindPrefs     = "prefs"
	KindEl        = "el"
)

// Kinds lists every node kind of the TSP vocabulary.
var Kinds = []string{
	KindCol, KindRow, KindCard, KindSection, KindRule, KindSpacer, KindText, KindMd, KindCode,
	KindDiff, KindAnsi, KindRows, KindMath, KindKV, KindTable, KindTree, KindBadge, KindKbd,
	KindIcon, KindImage, KindList, KindItem, KindTabs, KindPicker, KindSpinner, KindShimmer,
	KindElapsed, KindRate, KindProgress, KindMeter, KindChart, KindEffort, KindEditor, KindInput,
	KindStatus, KindSeg, KindToast, KindOverlay, KindTool, KindAgent, KindChecklist, KindBlock,
	KindPrefs, KindEl,
}

// TextKinds lists the kinds whose primary text lives in the text prop and
// is updated with the text and splice ops.
var TextKinds = []string{KindText, KindMd, KindCode, KindAnsi, KindMath, KindEditor, KindInput, KindShimmer, KindEl}

// IsTextKind reports whether kind keeps its primary text in the text prop.
func IsTextKind(kind string) bool {
	switch kind {
	case KindText, KindMd, KindCode, KindAnsi, KindMath, KindEditor, KindInput, KindShimmer, KindEl:
		return true
	}
	return false
}

// Program features announced in the hello query.
const (
	// FeatureEdit says the program applies edit events (native editing).
	FeatureEdit = "edit"
	// FeatureUndo says the program applies undo events.
	FeatureUndo = "undo"
	// FeatureSend says the program submits text from send events.
	FeatureSend = "send"
)

// Terminal features listed in the hello reply.
const (
	TermFeatureBlobs          = "blobs"
	TermFeatureSettle         = "settle"
	TermFeatureAdopt          = "adopt"
	TermFeatureDock           = "dock"
	TermFeatureProgramPalette = "program-palette"
	TermFeatureReduceMotion   = "reduce-motion"
	TermFeatureAside          = "aside"
	TermFeatureScroll         = "scroll"
	TermFeatureStyles         = "styles"
	TermFeatureFlow           = "flow"
)

// Message verbs, program to terminal and terminal to program.
const (
	VerbQuery   = "q"
	VerbOpen    = "o"
	VerbFrame   = "f"
	VerbBlob    = "b"
	VerbPalette = "t"
	VerbSheet   = "s"
	VerbClose   = "x"
	VerbReply   = "r"
	VerbEvent   = "e"
)

// Param is one key=value parameter of a TSP message.
type Param struct {
	Key, Value string
}

// HelloQuery is the body of the hello query (q).
type HelloQuery struct {
	Q        string   `json:"q"`
	V        []int    `json:"v"`
	App      string   `json:"app,omitempty"`
	Ver      string   `json:"ver,omitempty"`
	Features []string `json:"features,omitempty"`
}

// BlobsQuery is the body of the blobs query (q), asking which blobs Tern holds.
type BlobsQuery struct {
	Q   string   `json:"q"`
	IDs []string `json:"ids"`
}

// OpenMsg is the body of o, which opens a surface.
type OpenMsg struct {
	ID     string `json:"id"`
	Mode   Mode   `json:"mode,omitempty"`
	Title  string `json:"title,omitempty"`
	Role   string `json:"role,omitempty"`
	Adopt  bool   `json:"adopt,omitempty"`
	Listen *bool  `json:"listen,omitempty"`
}

// FrameMsg is the body of f: an atomic batch of ops for one surface.
type FrameMsg struct {
	SF  string `json:"sf"`
	S   int64  `json:"s"`
	Ops []Op   `json:"ops"`
}

// PaletteMsg is the body of t: the program palette of a surface.
type PaletteMsg struct {
	SF    string            `json:"sf,omitempty"`
	Dark  map[string]string `json:"dark,omitempty"`
	Light map[string]string `json:"light,omitempty"`
	Name  *PaletteNames     `json:"name,omitempty"`
}

// PaletteNames names the two variants of a palette.
type PaletteNames struct {
	Dark  string `json:"dark,omitempty"`
	Light string `json:"light,omitempty"`
}

// SheetMsg is the body of s: a stylesheet installed or removed (nil CSS).
type SheetMsg struct {
	SF   string  `json:"sf,omitempty"`
	Name string  `json:"name"`
	CSS  *string `json:"css"`
}

// CloseMsg is the body of x, which closes a surface.
type CloseMsg struct {
	ID   string `json:"id"`
	Keep bool   `json:"keep"`
}

// Op is one frame op, a JSON array such as ["del", id].
type Op []any

// WireNode is a node as it travels in an add op: id, kind, props, children.
type WireNode struct {
	ID string         `json:"id"`
	K  string         `json:"k"`
	P  map[string]any `json:"p,omitempty"`
	C  []WireNode     `json:"c,omitempty"`
}

// nullable is id as a JSON value, null when empty.
func nullable(id string) any {
	if id == "" {
		return nil
	}
	return id
}

// OpAdd inserts node under parent before sibling before ("" for last).
func OpAdd(parent, before string, node WireNode) Op {
	return Op{"add", node.ID, parent, nullable(before), node}
}

// OpSet merges props into node id; a nil value deletes that prop.
func OpSet(id string, props map[string]any) Op { return Op{"set", id, props} }

// OpText appends to ("append") or replaces ("replace") the primary text of id.
func OpText(id, mode, text string) Op { return Op{"text", id, mode, text} }

// OpSplice replaces del UTF-16 units at at in the primary text of id.
func OpSplice(id string, at, del int, text string) Op { return Op{"splice", id, at, del, text} }

// OpMove moves id under parent before sibling before ("" for last).
func OpMove(id, parent, before string) Op { return Op{"move", id, parent, nullable(before)} }

// OpDel removes the subtree id.
func OpDel(id string) Op { return Op{"del", id} }

// OpSettle hints that the subtree id is unlikely to change soon.
func OpSettle(id string) Op { return Op{"settle", id} }

// OpFocus gives the caret to editor or input id ("" for none).
func OpFocus(id string) Op { return Op{"focus", nullable(id)} }

// OpReveal scrolls id into view at "start", "end" or "nearest".
func OpReveal(id, at string) Op { return Op{"reveal", id, at} }

// OpScroll scrolls the container of id: "line-up", "line-down", "page-up",
// "page-down", "start" or "end".
func OpScroll(id, by string) Op { return Op{"scroll", id, by} }

// OpSuspend hands the pane back to the grid.
func OpSuspend() Op { return Op{"suspend"} }

// OpResume takes the pane again after OpSuspend.
func OpResume() Op { return Op{"resume"} }

// Marshal writes v as compact JSON without HTML escaping.
func Marshal(v any) ([]byte, error) {
	var buf bytes.Buffer
	enc := json.NewEncoder(&buf)
	enc.SetEscapeHTML(false)
	if err := enc.Encode(v); err != nil {
		return nil, err
	}
	return bytes.TrimSuffix(buf.Bytes(), []byte{'\n'}), nil
}

// keyByte reports whether b may appear in a parameter key.
func keyByte(b byte) bool {
	return b >= 'A' && b <= 'Z' || b >= 'a' && b <= 'z' || b >= '0' && b <= '9' || b == '_' || b == '-'
}

// valueByte reports whether b may appear in a parameter value.
func valueByte(b byte) bool { return b >= 0x21 && b <= 0x7e && b != ';' }

// leadsWithParam reports whether rest starts with a key=value segment
// followed by ';', which a parser would read as one more parameter.
func leadsWithParam(rest []byte) bool {
	k := 0
	for k < len(rest) && keyByte(rest[k]) {
		k++
	}
	if k == 0 || k >= len(rest) || rest[k] != '=' {
		return false
	}
	for _, b := range rest[k+1:] {
		if !valueByte(b) {
			return b == ';'
		}
	}
	return false
}

// chunkEnd is where the chunk of body starting at start ends.
func chunkEnd(body []byte, start, limit int) int {
	target := start + max(limit, 1)
	if target >= len(body) {
		return len(body)
	}
	safe := func(at int) bool { return body[at]&0xc0 != 0x80 && !leadsWithParam(body[at:]) }
	for at := target; at > start; at-- {
		if safe(at) {
			return at
		}
	}
	for at := target + 1; at < len(body); at++ {
		if safe(at) {
			return at
		}
	}
	return len(body)
}

// Chunks splits body into pieces of at most limit bytes that Tern's parser
// reads back whole: cut on UTF-8 boundaries, never before a
// parameter-shaped segment. A body within limit is one piece.
func Chunks(body []byte, limit int) [][]byte {
	if len(body) <= limit {
		return [][]byte{body}
	}
	var out [][]byte
	for start := 0; start < len(body); {
		end := chunkEnd(body, start, limit)
		out = append(out, body[start:end])
		start = end
	}
	return out
}

// Encode frames body as one TSP message of verb with params, chunked with
// chunk id chunk when body is over limit.
func Encode(verb string, params []Param, body []byte, limit int, chunk string) []byte {
	var out bytes.Buffer
	head := func(params []Param) {
		out.WriteString("\x1b_tsp;")
		out.WriteString(verb)
		out.WriteByte(';')
		for _, p := range params {
			out.WriteString(p.Key)
			out.WriteByte('=')
			out.WriteString(p.Value)
			out.WriteByte(';')
		}
	}
	if len(body) <= limit {
		head(params)
		out.Write(body)
		out.WriteString("\x1b\\")
		return out.Bytes()
	}
	pieces := Chunks(body, limit)
	for i, piece := range pieces {
		var ps []Param
		if i == 0 {
			ps = append(ps, params...)
		}
		ps = append(ps, Param{"c", chunk})
		if i < len(pieces)-1 {
			ps = append(ps, Param{"m", "1"})
		}
		head(ps)
		out.Write(piece)
		out.WriteString("\x1b\\")
	}
	return out.Bytes()
}

// ErrBlobTooLarge is returned for a blob over MaxBlob bytes.
var ErrBlobTooLarge = errors.New("tern: blob over 16 MiB")

// BlobID is the id of data: the lowercase hex SHA-256 of its bytes.
func BlobID(data []byte) string {
	sum := sha256.Sum256(data)
	return hex.EncodeToString(sum[:])
}

// EncodeBlob frames data as a b message with its id and optional mime.
func EncodeBlob(data []byte, mime string, limit int, chunk string) ([]byte, string, error) {
	if len(data) > MaxBlob {
		return nil, "", ErrBlobTooLarge
	}
	id := BlobID(data)
	params := []Param{{"id", id}}
	if mime != "" {
		params = append(params, Param{"mime", mime})
	}
	body := []byte(encodeBase64(data))
	return Encode(VerbBlob, params, body, limit, chunk), id, nil
}

// encodeBase64 is data in standard base64.
func encodeBase64(data []byte) string { return base64.StdEncoding.EncodeToString(data) }

// Message is one TSP message split into verb, parameters and body.
type Message struct {
	Verb   string
	Params []Param
	Body   []byte
}

// SplitMessage splits the data after "tsp;" into a message; ok is false
// when it has no verb.
func SplitMessage(inner []byte) (Message, bool) {
	semi := bytes.IndexByte(inner, ';')
	if semi < 0 {
		return Message{Verb: string(inner)}, len(inner) > 0
	}
	if semi == 0 {
		return Message{}, false
	}
	m := Message{Verb: string(inner[:semi])}
	pos := semi + 1
	for {
		n := bytes.IndexByte(inner[pos:], ';')
		if n < 0 {
			break
		}
		seg := inner[pos : pos+n]
		eq := bytes.IndexByte(seg, '=')
		if eq <= 0 || !validParam(seg[:eq], seg[eq+1:]) {
			break
		}
		m.Params = append(m.Params, Param{string(seg[:eq]), string(seg[eq+1:])})
		pos += n + 1
	}
	m.Body = inner[pos:]
	return m, true
}

// validParam reports whether key and value form a parameter.
func validParam(key, value []byte) bool {
	for _, b := range key {
		if !keyByte(b) {
			return false
		}
	}
	for _, b := range value {
		if !valueByte(b) {
			return false
		}
	}
	return true
}

// chunkID is the base-36 chunk id of counter n.
func chunkID(n uint64) string { return strconv.FormatUint(n, 36) }

// paramsObject is params as a JSON object, for recordings.
func paramsObject(params []Param) map[string]string {
	out := make(map[string]string, len(params))
	for _, p := range params {
		out[p.Key] = p.Value
	}
	return out
}
