package tern

import "bytes"

// MaxInputMessage is the size past which a TSP message on input is dropped.
const MaxInputMessage = 32 << 20

// Item is one thing the input parser found, in input order. Exactly one of
// Keys (pass-through bytes), Reply, Event or DA1 is set.
type Item struct {
	// Keys are bytes that are not TSP, passed through exactly.
	Keys []byte
	// Reply is a decoded r message.
	Reply *Reply
	// Event is a decoded e message.
	Event Event
	// DA1 marks a primary device attributes answer (ESC [ ? … c).
	DA1 bool
}

// parserState is what the parser is in the middle of.
type parserState int

const (
	// stateGround: looking for the next ESC.
	stateGround parserState = iota
	// stateTSP: inside a recognized TSP message, waiting for its terminator.
	stateTSP
	// stateDrop: inside an oversized TSP message, discarding to its terminator.
	stateDrop
	// statePaste: inside a bracketed paste, passing everything through.
	statePaste
)

// Parser splits TSP replies and events and DA1 answers out of a pty's input
// and passes every other byte through as keys. Feed it bytes as they
// arrive; call Flush after a pause to release undecided prefixes. It is not
// safe for concurrent use.
type Parser struct {
	buf   []byte
	state parserState
	// open is the length of the TSP opener at the start of buf in stateTSP.
	open int
	// scanned is how far into buf a terminator has been looked for.
	scanned int
	items   []Item
}

var (
	apcTSP   = []byte("\x1b_tsp;")
	oscTSP   = []byte("\x1b]877;tsp;")
	pasteOn  = []byte("\x1b[200~")
	pasteOff = []byte("\x1b[201~")
)

// Feed parses p and returns the items it completed.
func (p *Parser) Feed(data []byte) []Item {
	p.buf = append(p.buf, data...)
	p.run(false)
	return p.take()
}

// Flush releases an undecided prefix (a lone ESC, a partial CSI) as keys.
// A TSP message already recognized and a partial paste end marker are kept.
func (p *Parser) Flush() []Item {
	p.run(true)
	return p.take()
}

// take returns and clears the completed items.
func (p *Parser) take() []Item {
	out := p.items
	p.items = nil
	return out
}

// keys appends pass-through bytes, joining them with a previous keys item.
func (p *Parser) keys(b []byte) {
	if len(b) == 0 {
		return
	}
	if n := len(p.items); n > 0 && p.items[n-1].Keys != nil {
		p.items[n-1].Keys = append(p.items[n-1].Keys, b...)
		return
	}
	p.items = append(p.items, Item{Keys: bytes.Clone(b)})
}

// consume drops the first n bytes of buf.
func (p *Parser) consume(n int) {
	p.buf = p.buf[n:]
	if len(p.buf) == 0 {
		p.buf = nil
	}
}

// run parses as much of buf as can be decided; flush releases what can't.
func (p *Parser) run(flush bool) {
	for len(p.buf) > 0 {
		switch p.state {
		case statePaste:
			if !p.paste() {
				return
			}
		case stateTSP, stateDrop:
			if !p.message() {
				return
			}
		default:
			if !p.ground(flush) {
				return
			}
		}
	}
}

// paste passes bytes through until the end of the paste; false when it
// needs more input.
func (p *Parser) paste() bool {
	if i := bytes.Index(p.buf, pasteOff); i >= 0 {
		p.keys(p.buf[:i+len(pasteOff)])
		p.consume(i + len(pasteOff))
		p.state = stateGround
		return true
	}
	hold := partialSuffix(p.buf, pasteOff)
	p.keys(p.buf[:len(p.buf)-hold])
	p.consume(len(p.buf) - hold)
	return false
}

// partialSuffix is the length of the longest suffix of b that is a proper
// prefix of marker.
func partialSuffix(b, marker []byte) int {
	for n := min(len(marker)-1, len(b)); n > 0; n-- {
		if bytes.HasPrefix(marker, b[len(b)-n:]) {
			return n
		}
	}
	return 0
}

// message looks for the end of the TSP message at the start of buf; false
// when it needs more input.
func (p *Parser) message() bool {
	end, term := -1, 0
	for i := max(p.scanned, p.open); i < len(p.buf); i++ {
		if p.buf[i] == 0x07 {
			end, term = i, 1
			break
		}
		if p.buf[i] == 0x1b && i+1 < len(p.buf) && p.buf[i+1] == '\\' {
			end, term = i, 2
			break
		}
	}
	if end < 0 {
		p.scanned = max(len(p.buf)-1, p.open)
		if p.state == stateTSP && len(p.buf) > MaxInputMessage {
			p.state = stateDrop
		}
		if p.state == stateDrop {
			keep := 0
			if p.buf[len(p.buf)-1] == 0x1b {
				keep = 1
			}
			p.buf = append(p.buf[:0], p.buf[len(p.buf)-keep:]...)
			p.open, p.scanned = 0, 0
		}
		return false
	}
	if p.state == stateTSP && end+term <= MaxInputMessage {
		p.deliver(p.buf[p.open:end])
	}
	p.consume(end + term)
	p.state, p.open, p.scanned = stateGround, 0, 0
	return true
}

// deliver decodes one message's data after "tsp;" and keeps r and e bodies
// that are JSON objects.
func (p *Parser) deliver(inner []byte) {
	m, ok := SplitMessage(inner)
	if !ok {
		return
	}
	switch m.Verb {
	case VerbReply:
		if r, ok := decodeReply(m.Body); ok {
			p.items = append(p.items, Item{Reply: r})
		}
	case VerbEvent:
		if ev, ok := DecodeEvent(m.Body); ok {
			p.items = append(p.items, Item{Event: ev})
		}
	}
}

// ground passes bytes through up to the next escape sequence and decides
// it; false when it needs more input.
func (p *Parser) ground(flush bool) bool {
	esc := bytes.IndexByte(p.buf, 0x1b)
	if esc < 0 {
		p.keys(p.buf)
		p.consume(len(p.buf))
		return false
	}
	if esc > 0 {
		p.keys(p.buf[:esc])
		p.consume(esc)
	}
	n, kind := p.sequence()
	switch kind {
	case seqUndecided:
		if flush {
			p.keys(p.buf)
			p.consume(len(p.buf))
		}
		return false
	case seqTSP:
		p.state, p.open, p.scanned = stateTSP, n, n
	case seqDA1:
		p.items = append(p.items, Item{DA1: true})
		p.consume(n)
	case seqPaste:
		p.keys(p.buf[:n])
		p.consume(n)
		p.state = statePaste
	default:
		p.keys(p.buf[:n])
		p.consume(n)
	}
	return true
}

// seqKind classifies the escape sequence at the start of buf.
type seqKind int

const (
	seqUndecided seqKind = iota
	seqKeys
	seqTSP
	seqDA1
	seqPaste
)

// sequence classifies the escape sequence at the start of buf and gives the
// bytes it decides (the opener for TSP).
func (p *Parser) sequence() (int, seqKind) {
	b := p.buf
	if len(b) < 2 {
		return 0, seqUndecided
	}
	switch b[1] {
	case '_':
		return opener(b, apcTSP)
	case ']':
		return opener(b, oscTSP)
	case '[':
		return csi(b)
	}
	return 1, seqKeys
}

// opener matches a TSP opener at the start of b: decided TSP, undecided
// while b is a prefix of it, else the two-byte introducer as keys.
func opener(b, open []byte) (int, seqKind) {
	if bytes.HasPrefix(b, open) {
		return len(open), seqTSP
	}
	if bytes.HasPrefix(open, b) {
		return 0, seqUndecided
	}
	return 2, seqKeys
}

// csi decides the CSI sequence at the start of b.
func csi(b []byte) (int, seqKind) {
	i := 2
	for i < len(b) && b[i] >= 0x30 && b[i] <= 0x3f {
		i++
	}
	for i < len(b) && b[i] >= 0x20 && b[i] <= 0x2f {
		i++
	}
	if i >= len(b) {
		return 0, seqUndecided
	}
	if b[i] < 0x40 || b[i] > 0x7e {
		return i, seqKeys
	}
	n := i + 1
	seq := b[:n]
	if bytes.Equal(seq, pasteOn) {
		return n, seqPaste
	}
	if b[i] == 'c' && i > 3 && b[2] == '?' && da1Params(seq[3:i]) {
		return n, seqDA1
	}
	return n, seqKeys
}

// da1Params reports whether p holds only digits and ';'.
func da1Params(p []byte) bool {
	for _, c := range p {
		if c != ';' && (c < '0' || c > '9') {
			return false
		}
	}
	return true
}
