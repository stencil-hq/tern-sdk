package tern

import (
	"encoding/json"
	"fmt"
	"math"
	"regexp"
	"strconv"
	"strings"
	"unicode"
	"unicode/utf8"

	"golang.org/x/text/width"
)

// DefaultCols is the width Plain assumes when it is given none.
const DefaultCols = 80

// barWidth is the cells of a plain progress bar between its brackets.
const barWidth = 20

// Plain renders view as readable plain text without escapes, for output
// outside Tern: text kinds as their text, rows on one line, cards and
// sections as a head line over an indented body, tables and kv as aligned
// columns, lists as bullets, progress and meters as bars, motion kinds as
// their label and el as its text with block tags on lines of their own.
// cols <= 0 means DefaultCols.
func Plain(view View, cols int) string {
	if cols <= 0 {
		cols = DefaultCols
	}
	p := plainer{cols: cols}
	var lines []string
	for _, el := range []Element{view.Main, view.Dock, view.Layer} {
		if el != nil {
			lines = append(lines, p.node(plainTree(el.Node()))...)
		}
	}
	if len(lines) == 0 {
		return ""
	}
	return stripANSI(strings.Join(lines, "\n") + "\n")
}

// plainTree is n with props normalized and children resolved; ids are not
// needed, so duplicate keys are not an error here.
func plainTree(n Node) *tree {
	t := &tree{kind: n.Kind, props: normalizeProps(n.Props)}
	for _, c := range n.Children {
		if c != nil {
			t.children = append(t.children, plainTree(c.Node()))
		}
	}
	for name, c := range n.Slots {
		if c != nil {
			if t.slots == nil {
				t.slots = map[string]*tree{}
			}
			t.slots[name] = plainTree(c.Node())
		}
	}
	return t
}

// slotLine is a slot node (head or end) on one line, else the prop's spans.
func (p plainer) slotLine(t *tree, name string) string {
	if s, ok := t.slots[name]; ok {
		return oneLine(p.node(s))
	}
	if name == "head" {
		return spansText(t.props["head"])
	}
	return ""
}

// panel is a card, section or overlay: head and end over the indented
// body, then the foot.
func (p plainer) panel(t *tree, extra string) []string {
	head := join("  ", join(" ", p.slotLine(t, "head"), extra), spansText(t.props["meta"]),
		p.slotLine(t, "end"))
	lines := p.headed(head, p.children(t))
	if s, ok := t.slots["foot"]; ok {
		lines = append(lines, indent(p.node(s))...)
	}
	return lines
}

// plainer renders trees to lines.
type plainer struct {
	cols int
}

// ansiEscape matches terminal escape sequences, including their C1 forms.
var ansiEscape = regexp.MustCompile(`(?:\x1b\[|\x{009b})[0-?]*[ -/]*[@-~]|(?:\x1b\]|\x{009d})[^\x07\x1b\x{009c}]*(?:\x07|\x1b\\|\x{009c})|(?:\x1b[PX^_]|\x{0090}|\x{0098}|\x{009e}|\x{009f})[^\x1b\x{009c}]*(?:\x1b\\|\x{009c})|\x1b[ -/]*[@-~]`)

// stripANSI removes escape sequences and stray control characters but tabs
// and newlines.
func stripANSI(s string) string {
	s = ansiEscape.ReplaceAllString(s, "")
	return strings.Map(func(r rune) rune {
		if r < 0x20 && r != '\n' && r != '\t' || r >= 0x7f && r <= 0x9f {
			return -1
		}
		return r
	}, s)
}

// str is a string prop ("" when absent or not a string).
func str(props map[string]any, key string) string {
	s, _ := props[key].(string)
	return s
}

// num is a number prop and whether it is one.
func num(v any) (float64, bool) {
	n, ok := v.(json.Number)
	if !ok {
		return 0, false
	}
	f, err := n.Float64()
	return f, err == nil
}

// spansText is the text of a string or a span list.
func spansText(v any) string {
	switch x := v.(type) {
	case string:
		return x
	case []any:
		var b strings.Builder
		for _, s := range x {
			b.WriteString(spansText(s))
		}
		return b.String()
	case map[string]any:
		t, _ := x["t"].(string)
		return t
	case json.Number:
		return x.String()
	}
	return ""
}

// richText is a node's text: spans when they are a list, else text.
func richText(props map[string]any) string {
	if spans, ok := props["spans"].([]any); ok {
		return spansText(spans)
	}
	return spansText(props["text"])
}

// splitLines splits s on newlines, nil for an empty s.
func splitLines(s string) []string {
	s = strings.TrimRight(s, "\n")
	if s == "" {
		return nil
	}
	return strings.Split(s, "\n")
}

// indent prefixes every line with two spaces.
func indent(lines []string) []string {
	out := make([]string, len(lines))
	for i, l := range lines {
		if l == "" {
			out[i] = l
		} else {
			out[i] = "  " + l
		}
	}
	return out
}

// oneLine joins lines with spaces.
func oneLine(lines []string) string {
	var parts []string
	for _, l := range lines {
		if l = strings.TrimSpace(l); l != "" {
			parts = append(parts, l)
		}
	}
	return strings.Join(parts, " ")
}

// join joins the non-empty parts with sep.
func join(sep string, parts ...string) string {
	var out []string
	for _, p := range parts {
		if p != "" {
			out = append(out, p)
		}
	}
	return strings.Join(out, sep)
}

// children renders every child, one after another.
func (p plainer) children(t *tree) []string {
	var lines []string
	for _, c := range t.children {
		lines = append(lines, p.node(c)...)
	}
	return lines
}

// headed is a head line over the indented body.
func (p plainer) headed(head string, body []string) []string {
	var lines []string
	if head != "" {
		lines = append(lines, head)
		return append(lines, indent(body)...)
	}
	return body
}

// status is a run status as a bracketed chip.
func status(props map[string]any) string {
	if s := str(props, "status"); s != "" {
		return "[" + s + "]"
	}
	return ""
}

// node renders one node.
func (p plainer) node(t *tree) []string {
	pr := t.props
	if hidden, _ := pr["hidden"].(bool); hidden {
		return nil
	}
	switch t.kind {
	case KindRow:
		var parts []string
		for _, c := range t.children {
			parts = append(parts, oneLine(p.node(c)))
		}
		return nonEmpty(join(" ", parts...))
	case KindCard:
		state := str(pr, "status")
		if state != "" {
			state = "(" + state + ")"
		}
		return p.panel(t, state)
	case KindSection, KindOverlay:
		return p.panel(t, "")
	case KindButton:
		if text := str(pr, "text"); text != "" {
			return []string{"[" + text + "]"}
		}
		return nil
	case KindStats:
		return statsLine(pr)
	case KindTool:
		head := join(" ", spansText(pr["title"]), spansText(pr["target"]), metaText(pr["meta"]), spansText(pr["note"]), status(pr))
		return p.headed(head, p.children(t))
	case KindAgent:
		head := join(" ", str(pr, "name"), str(pr, "agent"), spansText(pr["task"]), status(pr))
		return p.headed(head, p.children(t))
	case KindRule:
		label := stripANSI(join(" · ", spansText(pr["label"]), str(pr, "time")))
		if label == "" {
			return []string{strings.Repeat("─", p.cols)}
		}
		rest := max(p.cols-displayWidth(label)-4, 2)
		return []string{"── " + label + " " + strings.Repeat("─", rest)}
	case KindSpacer:
		return []string{""}
	case KindText:
		if spans, ok := pr["spans"].([]any); ok {
			return splitLines(citedText(spans))
		}
		return splitLines(richText(pr))
	case KindShimmer, KindSeg:
		return splitLines(richText(pr))
	case KindMd, KindCode, KindMath:
		return splitLines(str(pr, "text"))
	case KindEditor, KindInput:
		text := str(pr, "text")
		if secret, _ := pr["secret"].(bool); secret {
			text = strings.Repeat("•", utf8.RuneCountInString(text))
		}
		if text == "" {
			text = spansText(pr["placeholder"])
		}
		return splitLines(spansText(pr["prompt"]) + text)
	case KindSelect:
		return nonEmpty(join(": ", str(pr, "label"), selectShown(pr)))
	case KindSwitch:
		mark := "[ ]"
		if on, _ := pr["on"].(bool); on {
			mark = "[x]"
		}
		return []string{join(" ", mark, str(pr, "label"))}
	case KindSlider:
		return nonEmpty(join(": ", str(pr, "label"), sliderValue(pr)))
	case KindField:
		return p.fieldLines(t)
	case KindOtp:
		return []string{otpText(pr)}
	case KindButtons:
		var parts []string
		for _, c := range t.children {
			parts = append(parts, oneLine(p.node(c)))
		}
		return nonEmpty(join(" ", parts...))
	case KindAvatar:
		return nonEmpty(str(pr, "text"))
	case KindAvatars:
		var names []string
		items, _ := pr["items"].([]any)
		for _, it := range items {
			m, _ := it.(map[string]any)
			if s := str(m, "text"); s != "" {
				names = append(names, s)
			}
		}
		return nonEmpty(join(" ", strings.Join(names, ", "), str(pr, "label")))
	case KindBreadcrumb:
		sep := " › "
		if str(pr, "variant") == "path" {
			sep = " / "
		}
		var parts []string
		items, _ := pr["items"].([]any)
		for _, it := range items {
			m, _ := it.(map[string]any)
			parts = append(parts, labelOrID(m))
		}
		return nonEmpty(join(sep, parts...))
	case KindPager:
		return []string{pagerText(pr)}
	case KindEmpty:
		lines := nonEmpty(spansText(pr["title"]))
		lines = append(lines, nonEmpty(spansText(pr["text"]))...)
		return append(lines, p.children(t)...)
	case KindSkeleton:
		return nil
	case KindCalendar:
		return nonEmpty(datesText(pr["value"]))
	case KindDate:
		if s := datesText(pr["value"]); s != "" {
			return []string{s}
		}
		return nonEmpty(str(pr, "placeholder"))
	case KindMenu:
		if s := str(pr, "text"); s != "" {
			return []string{"[" + s + "]"}
		}
		if s := str(pr, "title"); s != "" {
			return []string{"[" + s + "]"}
		}
		return nil
	case KindMenubar:
		var parts []string
		menus, _ := pr["menus"].([]any)
		for _, it := range menus {
			m, _ := it.(map[string]any)
			parts = append(parts, labelOrID(m))
		}
		return nonEmpty(join("  ", parts...))
	case KindNav:
		return navLine(pr)
	case KindCarousel:
		if len(t.children) == 0 {
			return nil
		}
		i, _ := num(pr["index"])
		return p.node(t.children[min(max(int(i), 0), len(t.children)-1)])
	case KindMessage:
		return p.messageLines(t)
	case KindAttachment:
		return []string{"[attachment: " + str(pr, "name") + "]"}
	case KindTrace:
		return p.traceLines(t)
	case KindStacktrace:
		return stackLines(pr)
	case KindSuggestions:
		var lines []string
		items, _ := pr["items"].([]any)
		for _, it := range items {
			m, _ := it.(map[string]any)
			if s := str(m, "text"); s != "" {
				lines = append(lines, "- "+s)
			}
		}
		return lines
	case KindQuestions:
		return questionLines(pr)
	case KindAnsi:
		return splitLines(stripANSI(str(pr, "text")))
	case KindRows:
		var lines []string
		list, _ := pr["lines"].([]any)
		for _, l := range list {
			if s, ok := l.(string); ok {
				lines = append(lines, stripANSI(s))
			}
		}
		return lines
	case KindDiff:
		return diffLines(pr)
	case KindKV:
		return kvLines(pr)
	case KindTable:
		lines := tableLines(pr)
		if rows, _ := pr["rows"].([]any); len(rows) == 0 {
			lines = append(lines, splitLines(spansText(pr["empty"]))...)
		}
		return lines
	case KindTree:
		list, _ := pr["nodes"].([]any)
		return treeLines(list)
	case KindJSON:
		var b strings.Builder
		enc := json.NewEncoder(&b)
		enc.SetEscapeHTML(false)
		enc.SetIndent("", "  ")
		if err := enc.Encode(pr["value"]); err != nil {
			return nil
		}
		return strings.Split(strings.TrimSuffix(b.String(), "\n"), "\n")
	case KindBadge:
		if s := str(pr, "text"); s != "" {
			return []string{"[" + s + "]"}
		}
		return nil
	case KindKbd:
		var keys []string
		list, _ := pr["keys"].([]any)
		for _, k := range list {
			if s, ok := k.(string); ok {
				keys = append(keys, s)
			}
		}
		return nonEmpty(strings.Join(keys, "+"))
	case KindIcon:
		return nonEmpty(str(pr, "aria"))
	case KindImage:
		alt := str(pr, "alt")
		if alt == "" {
			alt = "image"
		}
		return []string{"[" + alt + "]"}
	case KindMedia:
		path := str(pr, "path")
		if path == "" {
			path = "media"
		}
		if rev := str(pr, "rev"); rev != "" {
			path += " @ " + rev
		}
		return []string{"[" + path + "]"}
	case KindList:
		var lines []string
		for _, c := range t.children {
			lines = append(lines, p.node(c)...)
		}
		if len(lines) == 0 {
			return splitLines(spansText(pr["empty"]))
		}
		return lines
	case KindItem:
		return []string{"- " + join("  ", spansText(pr["label"]), spansText(pr["detail"]), spansText(pr["value"]))}
	case KindTabs:
		return tabsLine(pr)
	case KindPicker:
		return pickerLines(pr)
	case KindSpinner:
		return nonEmpty(spansText(pr["label"]))
	case KindElapsed:
		return []string{elapsedText(pr)}
	case KindRate:
		v, _ := num(pr["value"])
		return []string{join(" ", strconv.FormatFloat(v, 'f', -1, 64), str(pr, "unit"))}
	case KindProgress:
		v, ok := num(pr["value"])
		return []string{join(" ", bar(v, ok), spansText(pr["label"]))}
	case KindMeter:
		v, ok := meterValue(pr)
		return []string{join(" ", bar(v, ok), spansText(pr["label"]), spansText(pr["total"]))}
	case KindChart:
		return chartLines(pr)
	case KindEffort:
		return nonEmpty(str(pr, "level"))
	case KindStatus:
		var left, right []string
		for _, c := range t.children {
			s := oneLine(p.node(c))
			if str(c.props, "side") == "right" {
				right = append(right, s)
			} else {
				left = append(left, s)
			}
		}
		return nonEmpty(join("  ", join("  ", left...), join("  ", right...)))
	case KindToast:
		return nonEmpty(join("  ", str(pr, "text"), str(pr, "sub")))
	case KindChecklist:
		return checklistLines(pr)
	case KindPrefs:
		return prefsLines(pr)
	case KindEl:
		var f flow
		p.el(t, &f)
		return f.done()
	}
	return p.children(t)
}

// nonEmpty is s as one line, or nothing when it is empty.
func nonEmpty(s string) []string {
	if s == "" {
		return nil
	}
	return []string{s}
}

// metaText joins a tool's meta facts.
func metaText(v any) string {
	list, _ := v.([]any)
	var parts []string
	for _, m := range list {
		parts = append(parts, spansText(m))
	}
	return join(" ", parts...)
}

// bar draws a fraction as [####----] 50%, or an unknown one as dashes.
func bar(v float64, known bool) string {
	if !known {
		return "[" + strings.Repeat("-", barWidth) + "]"
	}
	v = math.Max(0, math.Min(1, v))
	fill := int(math.Round(v * barWidth))
	return fmt.Sprintf("[%s%s] %d%%", strings.Repeat("#", fill), strings.Repeat("-", barWidth-fill), int(math.Round(v*100)))
}

// elapsedText formats an elapsed node's time.
func elapsedText(pr map[string]any) string {
	ms, _ := num(pr["age"])
	if stopped, ok := num(pr["stopped"]); ok {
		ms = stopped
	}
	ms = math.Max(0, ms)
	sec := int(ms / 1000)
	if str(pr, "format") == "clock" {
		if sec >= 3600 {
			return fmt.Sprintf("%d:%02d:%02d", sec/3600, sec/60%60, sec%60)
		}
		return fmt.Sprintf("%d:%02d", sec/60, sec%60)
	}
	switch {
	case sec >= 3600:
		return fmt.Sprintf("%dh %02dm", sec/3600, sec/60%60)
	case sec >= 60:
		return fmt.Sprintf("%dm %02ds", sec/60, sec%60)
	}
	return fmt.Sprintf("%d.%ds", sec, int(ms/100)%10)
}

// displayWidth counts terminal cells: wide/fullwidth runes occupy two cells,
// combining marks and formatting controls none, and other runes one.
func displayWidth(s string) int {
	cells := 0
	for _, r := range s {
		if unicode.Is(unicode.Mn, r) || unicode.Is(unicode.Me, r) || unicode.Is(unicode.Cf, r) {
			continue
		}
		switch width.LookupRune(r).Kind() {
		case width.EastAsianWide, width.EastAsianFullwidth:
			cells += 2
		default:
			cells++
		}
	}
	return cells
}

// columns aligns rows of cells; right marks right-aligned columns.
func columns(rows [][]string, right []bool) []string {
	var widths []int
	for _, r := range rows {
		for i, c := range r {
			if i >= len(widths) {
				widths = append(widths, 0)
			}
			c = stripANSI(c)
			r[i] = c
			widths[i] = max(widths[i], displayWidth(c))
		}
	}
	lines := make([]string, 0, len(rows))
	for _, r := range rows {
		var b strings.Builder
		for i, c := range r {
			if i > 0 {
				b.WriteString("  ")
			}
			pad := strings.Repeat(" ", widths[i]-displayWidth(c))
			if i < len(right) && right[i] {
				b.WriteString(pad + c)
			} else if i < len(r)-1 {
				b.WriteString(c + pad)
			} else {
				b.WriteString(c)
			}
		}
		lines = append(lines, strings.TrimRight(b.String(), " "))
	}
	return lines
}

// kvLines is a kv node as aligned key and value columns.
func kvLines(pr map[string]any) []string {
	items, _ := pr["items"].([]any)
	var rows [][]string
	for _, it := range items {
		m, _ := it.(map[string]any)
		rows = append(rows, []string{spansText(m["k"]), spansText(m["v"])})
	}
	if str(pr, "layout") == "inline" {
		var parts []string
		for _, r := range rows {
			parts = append(parts, r[0]+" "+r[1])
		}
		return nonEmpty(strings.Join(parts, "  "))
	}
	return columns(rows, nil)
}

// tableLines is a table node as aligned columns with its header.
func tableLines(pr map[string]any) []string {
	cols, _ := pr["cols"].([]any)
	rowsV, _ := pr["rows"].([]any)
	var ids []string
	var heads []string
	var right []bool
	hasHead := false
	for _, c := range cols {
		m, _ := c.(map[string]any)
		id := str(m, "id")
		if id == "" {
			continue
		}
		ids = append(ids, id)
		h := spansText(m["head"])
		hasHead = hasHead || h != ""
		heads = append(heads, h)
		right = append(right, str(m, "align") == "end")
	}
	var rows [][]string
	if hasHead {
		rows = append(rows, heads)
	}
	for _, r := range rowsV {
		m, _ := r.(map[string]any)
		cells, _ := m["cells"].(map[string]any)
		row := make([]string, len(ids))
		for i, id := range ids {
			row[i] = cellText(cells[id])
		}
		rows = append(rows, row)
	}
	return columns(rows, right)
}

// cellText is a table cell's text: its spans, a meter cell's bar, a spark
// cell's sparkline, a badge or button cell's bracketed label, or a text
// cell's text.
func cellText(v any) string {
	if cell, ok := v.(map[string]any); ok {
		if meter, ok := cell["meter"].(map[string]any); ok {
			val, known := meterValue(meter)
			return bar(val, known)
		}
		if spark, ok := cell["spark"].(map[string]any); ok {
			return sparkline(spark)
		}
		for _, k := range []string{"badge", "button"} {
			if m, ok := cell[k].(map[string]any); ok {
				if text := str(m, "text"); text != "" {
					return "[" + text + "]"
				}
				return ""
			}
		}
		if _, ok := cell["text"]; ok {
			return spansText(cell["text"])
		}
		return ""
	}
	return spansText(v)
}

// sparkBlocks are a sparkline's levels, lowest first.
var sparkBlocks = []rune("▁▂▃▄▅▆▇█")

// sparkline draws a spark cell's values as block levels between its min
// and max (default the values' own).
func sparkline(pr map[string]any) string {
	list, _ := pr["values"].([]any)
	vals := make([]float64, 0, len(list))
	for _, v := range list {
		if f, ok := num(v); ok && !math.IsNaN(f) && !math.IsInf(f, 0) {
			vals = append(vals, f)
		}
	}
	if len(vals) == 0 {
		return ""
	}
	lo, hi := vals[0], vals[0]
	for _, v := range vals {
		lo, hi = min(lo, v), max(hi, v)
	}
	if f, ok := num(pr["min"]); ok {
		lo = f
	}
	if f, ok := num(pr["max"]); ok {
		hi = f
	}
	var b strings.Builder
	top := len(sparkBlocks) - 1
	for _, v := range vals {
		i := 0
		if hi > lo {
			i = int(math.Round((v - lo) / (hi - lo) * float64(top)))
		}
		b.WriteRune(sparkBlocks[min(max(i, 0), top)])
	}
	return b.String()
}

// statsLine is a stats well as "label: value (sub)" per item.
func statsLine(pr map[string]any) []string {
	items, _ := pr["items"].([]any)
	var parts []string
	for _, it := range items {
		m, _ := it.(map[string]any)
		part := join(": ", spansText(m["label"]), spansText(m["value"]))
		if sub := spansText(m["sub"]); sub != "" {
			part = join(" ", part, "("+sub+")")
		}
		if part != "" {
			parts = append(parts, part)
		}
	}
	return nonEmpty(strings.Join(parts, "  ·  "))
}

// meterValue uses the explicit value, or the sum of nonnegative parts.
func meterValue(pr map[string]any) (float64, bool) {
	if value, ok := num(pr["value"]); ok {
		return value, true
	}
	parts, ok := pr["parts"].([]any)
	var value float64
	for _, part := range parts {
		if m, ok := part.(map[string]any); ok {
			v, _ := num(m["value"])
			value += math.Max(0, v)
		}
	}
	return value, ok
}

// treeLines is a tree's items as nested bullets.
func treeLines(nodes []any) []string {
	var lines []string
	for _, n := range nodes {
		m, _ := n.(map[string]any)
		lines = append(lines, "- "+spansText(m["label"]))
		kids, _ := m["children"].([]any)
		lines = append(lines, indent(treeLines(kids))...)
	}
	return lines
}

// tabsLine is a tab strip with the active tab bracketed.
func tabsLine(pr map[string]any) []string {
	items, _ := pr["items"].([]any)
	active := str(pr, "active")
	var parts []string
	for _, it := range items {
		m, _ := it.(map[string]any)
		label := spansText(m["label"])
		if str(m, "id") == active {
			label = "[" + label + "]"
		}
		parts = append(parts, label)
	}
	return nonEmpty(strings.Join(parts, " | "))
}

// pickerLines is a picker's title and its rows as bullets.
func pickerLines(pr map[string]any) []string {
	lines := nonEmpty(join(" ", spansText(pr["title"]), spansText(pr["subtitle"])))
	items, _ := pr["items"].([]any)
	byID := map[string]map[string]any{}
	var order []any
	for _, it := range items {
		m, _ := it.(map[string]any)
		if id := str(m, "id"); id != "" {
			byID[id] = m
			order = append(order, id)
		}
	}
	if o, ok := pr["order"].([]any); ok {
		order = o
	}
	for _, o := range order {
		switch x := o.(type) {
		case string:
			if m := byID[x]; m != nil {
				lines = append(lines, "- "+join("  ", spansText(m["label"]), spansText(m["detail"])))
			}
		case map[string]any:
			lines = append(lines, spansText(x["label"]))
		}
	}
	return lines
}

// diffLines is a diff's lines, from its text or its hunks.
func diffLines(pr map[string]any) []string {
	hunks, ok := pr["hunks"].([]any)
	if !ok {
		return splitLines(str(pr, "text"))
	}
	var lines []string
	for _, h := range hunks {
		m, _ := h.(map[string]any)
		list, _ := m["lines"].([]any)
		for _, l := range list {
			if s, ok := l.(string); ok {
				lines = append(lines, s)
			}
		}
	}
	return lines
}

// chartLines is a chart's summary, or its series: values, sparklines of
// lines and areas, or a donut's segments.
func chartLines(pr map[string]any) []string {
	if s := spansText(pr["summary"]); s != "" {
		return []string{s}
	}
	series, _ := pr["series"].([]any)
	switch str(pr, "kind") {
	case "line", "area":
		var lines []string
		for _, s := range series {
			if m, ok := s.(map[string]any); ok {
				if values, ok := m["values"].([]any); ok {
					lines = append(lines, join(" ", str(m, "label"), pointSpark(values)))
				}
			}
		}
		if lines != nil {
			return lines
		}
		return nonEmpty(pointSpark(series))
	case "donut":
		var parts []string
		for _, s := range series {
			m, isMap := s.(map[string]any)
			v := s
			if isMap {
				v = m["value"]
			}
			if f, ok := num(v); ok {
				parts = append(parts, join(" ", str(m, "label"), chartUnit(fmtNum(f), str(pr, "unit"))))
			}
		}
		return nonEmpty(strings.Join(parts, "  "))
	}
	var parts []string
	for _, pt := range series {
		m, _ := pt.(map[string]any)
		v, _ := num(m["value"])
		parts = append(parts, join(" ", str(m, "label"), strconv.FormatFloat(v, 'f', -1, 64)))
	}
	return nonEmpty(strings.Join(parts, "  "))
}

// checkMarks draws checklist item states.
var checkMarks = map[string]string{"done": "[x]", "active": "[>]", "dropped": "[-]", "blocked": "[!]"}

// checklistLines is a checklist's phases and items.
func checklistLines(pr map[string]any) []string {
	phases, _ := pr["phases"].([]any)
	var lines []string
	for _, ph := range phases {
		m, _ := ph.(map[string]any)
		var items []string
		list, _ := m["items"].([]any)
		for _, it := range list {
			im, _ := it.(map[string]any)
			mark := checkMarks[str(im, "status")]
			if mark == "" {
				mark = "[ ]"
			}
			items = append(items, mark+" "+spansText(im["text"]))
			if note := spansText(im["note"]); note != "" {
				items = append(items, "    "+note)
			}
		}
		if title := spansText(m["title"]); title != "" {
			lines = append(lines, title)
			lines = append(lines, indent(items)...)
		} else {
			lines = append(lines, items...)
		}
	}
	return lines
}

// prefsLines is a prefs page's sections and rows.
func prefsLines(pr map[string]any) []string {
	lines := nonEmpty(str(pr, "title"))
	sections, _ := pr["sections"].([]any)
	for _, sec := range sections {
		m, _ := sec.(map[string]any)
		var rows [][]string
		list, _ := m["rows"].([]any)
		for _, r := range list {
			rm, _ := r.(map[string]any)
			label := str(rm, "label")
			if label == "" {
				label = str(rm, "id")
			}
			ctrl, _ := rm["control"].(map[string]any)
			rows = append(rows, []string{label, controlText(ctrl)})
		}
		if title := str(m, "title"); title != "" {
			lines = append(lines, title)
			lines = append(lines, indent(columns(rows, nil))...)
		} else {
			lines = append(lines, columns(rows, nil)...)
		}
	}
	return lines
}

// controlText is a prefs control's current value.
func controlText(c map[string]any) string {
	switch str(c, "k") {
	case "switch":
		if on, _ := c["on"].(bool); on {
			return "on"
		}
		return "off"
	case "multi":
		vals, _ := c["values"].([]any)
		var parts []string
		for _, v := range vals {
			parts = append(parts, spansText(v))
		}
		return strings.Join(parts, ", ")
	case "action":
		if l := str(c, "label"); l != "" {
			return l
		}
		return "Edit…"
	case "keys":
		return ""
	}
	return spansText(c["value"])
}

// blockTags are the el tags that sit on lines of their own.
var blockTags = map[string]bool{
	"div": true, "p": true, "section": true, "header": true, "footer": true, "nav": true,
	"aside": true, "main": true, "article": true, "figure": true, "blockquote": true, "ul": true,
	"ol": true, "li": true, "dl": true, "dt": true, "dd": true, "h1": true, "h2": true, "h3": true,
	"h4": true, "pre": true, "hr": true, "table": true, "thead": true, "tbody": true, "tr": true,
	"form": true, "": true,
}

// flow gathers inline text into lines between block elements.
type flow struct {
	lines      []string
	cur        strings.Builder
	listNumber int
}

// inline adds text to the current line.
func (f *flow) inline(s string) { f.cur.WriteString(s) }

// block ends the current line and adds lines of their own.
func (f *flow) block(lines []string) {
	f.end()
	f.lines = append(f.lines, lines...)
}

// end ends the current line.
func (f *flow) end() {
	if s := strings.TrimRight(f.cur.String(), " "); s != "" {
		f.lines = append(f.lines, s)
	}
	f.cur.Reset()
}

// done is every line gathered.
func (f *flow) done() []string {
	f.end()
	return f.lines
}

// el renders an el node into f.
func (p plainer) el(t *tree, f *flow) {
	pr := t.props
	if hidden, _ := pr["hidden"].(bool); hidden {
		return
	}
	tag := str(pr, "tag")
	switch tag {
	case "input":
		checked, _ := pr["checked"].(bool)
		mark := map[[2]bool]string{{false, false}: "[ ] ", {false, true}: "[x] ", {true, false}: "( ) ", {true, true}: "(*) "}
		f.inline(mark[[2]bool{str(pr, "type") == "radio", checked}])
		return
	case "hr":
		f.block([]string{strings.Repeat("─", min(p.cols, 40))})
		return
	case "table":
		f.block(elTable(p, t))
		return
	case "button":
		var inner flow
		p.content(t, &inner)
		f.inline("[" + oneLine(inner.done()) + "] ")
		return
	}
	if !blockTags[tag] {
		p.content(t, f)
		return
	}
	var inner flow
	if tag == "ol" {
		inner.listNumber = 1
	}
	p.content(t, &inner)
	lines := inner.done()
	switch tag {
	case "li":
		marker := "- "
		if f.listNumber > 0 {
			marker = strconv.Itoa(f.listNumber) + ". "
			f.listNumber++
		}
		for i := range lines {
			if i == 0 {
				lines[i] = marker + lines[i]
			} else {
				lines[i] = strings.Repeat(" ", len(marker)) + lines[i]
			}
		}
	case "dd", "blockquote", "ul", "ol":
		lines = indent(lines)
	}
	f.block(lines)
}

// content renders an el's text and children into f.
func (p plainer) content(t *tree, f *flow) {
	if s := str(t.props, "text"); s != "" {
		lines := strings.Split(s, "\n")
		for i, l := range lines {
			if i > 0 {
				f.end()
			}
			f.inline(l)
		}
	}
	for _, c := range t.children {
		if c.kind == KindEl {
			p.el(c, f)
		} else {
			f.block(p.node(c))
		}
	}
}

// numeric matches cell text that reads as a number or a size.
var numeric = regexp.MustCompile(`^[-+]?[0-9][0-9.,]*\s?[A-Za-z%]{0,3}$`)

// elTable is an el table as aligned columns; columns whose td cells are
// all numbers align right.
func elTable(p plainer, t *tree) []string {
	var rows [][]string
	var head []bool
	var collect func(t *tree)
	collect = func(t *tree) {
		for _, c := range t.children {
			switch str(c.props, "tag") {
			case "thead", "tbody":
				collect(c)
			case "tr":
				var row []string
				isHead := false
				for _, cell := range c.children {
					var f flow
					p.content(cell, &f)
					row = append(row, oneLine(f.done()))
					isHead = isHead || str(cell.props, "tag") == "th"
				}
				rows = append(rows, row)
				head = append(head, isHead)
			}
		}
	}
	collect(t)
	var right []bool
	for ri, r := range rows {
		for i, c := range r {
			for len(right) <= i {
				right = append(right, true)
			}
			if !head[ri] && c != "" && !numeric.MatchString(c) {
				right[i] = false
			}
		}
	}
	return columns(rows, right)
}

// fmtNum is f in decimal, without a trailing .0.
func fmtNum(f float64) string { return strconv.FormatFloat(f, 'f', -1, 64) }

// citedText is a text's spans with [n] after each cited run. A source keeps
// the number its first cite gives it, else takes the next free one in order
// of first appearance; cites without an href are ignored.
func citedText(spans []any) string {
	used := map[int]bool{}
	for _, s := range spans {
		for _, c := range cites(s) {
			if n, ok := num(c["n"]); ok && n >= 1 {
				used[int(n)] = true
			}
		}
	}
	numbers := map[string]int{}
	next := 1
	var b strings.Builder
	for _, s := range spans {
		b.WriteString(spansText(s))
		for _, c := range cites(s) {
			href := str(c, "href")
			if href == "" {
				continue
			}
			n, seen := numbers[href]
			if !seen {
				if f, ok := num(c["n"]); ok && f >= 1 {
					n = int(f)
				} else {
					for used[next] {
						next++
					}
					n = next
					used[n] = true
				}
				numbers[href] = n
			}
			b.WriteString("[" + strconv.Itoa(n) + "]")
		}
	}
	return b.String()
}

// cites are a span's cites, given as one or as a list.
func cites(span any) []map[string]any {
	m, _ := span.(map[string]any)
	switch c := m["cite"].(type) {
	case map[string]any:
		return []map[string]any{c}
	case []any:
		out := make([]map[string]any, 0, len(c))
		for _, x := range c {
			if cm, ok := x.(map[string]any); ok {
				out = append(out, cm)
			}
		}
		return out
	}
	return nil
}

// pointSpark is the sparkline of a line's numbers and points' values;
// gaps are skipped.
func pointSpark(points []any) string {
	values := make([]any, 0, len(points))
	for _, pt := range points {
		if m, ok := pt.(map[string]any); ok {
			pt = m["value"]
		}
		if _, ok := num(pt); ok {
			values = append(values, pt)
		}
	}
	return sparkline(map[string]any{"values": values})
}

// chartUnit writes unit with a chart's value: a currency symbol before it
// (after any minus sign), against it when it starts with %, × or °, else
// after a space.
func chartUnit(v, unit string) string {
	switch unit {
	case "$", "€", "£", "¥":
		if rest, ok := strings.CutPrefix(v, "-"); ok {
			return "−" + unit + rest
		}
		if rest, ok := strings.CutPrefix(v, "−"); ok {
			return "−" + unit + rest
		}
		return unit + v
	}
	if strings.HasPrefix(unit, "%") || strings.HasPrefix(unit, "×") || strings.HasPrefix(unit, "°") {
		return v + unit
	}
	return join(" ", v, unit)
}

// sliderUnits are the units a slider writes against its number.
var sliderUnits = map[string]bool{"%": true, "‰": true, "°": true, "×": true, "x": true, "′": true, "″": true, "'": true, `"`: true}

// sliderValue is a slider's value, or its range lo–hi, with its unit.
func sliderValue(pr map[string]any) string {
	var v string
	switch x := pr["value"].(type) {
	case json.Number:
		f, _ := num(x)
		v = fmtNum(f)
	case []any:
		if len(x) == 2 {
			lo, okLo := num(x[0])
			hi, okHi := num(x[1])
			if okLo && okHi {
				v = fmtNum(lo) + "–" + fmtNum(hi)
			}
		}
	}
	if v == "" {
		return ""
	}
	if unit := str(pr, "unit"); sliderUnits[unit] {
		return v + unit
	}
	return join(" ", v, str(pr, "unit"))
}

// selectShown is what a select's trigger shows: the current option's label,
// else the value, else the placeholder.
func selectShown(pr map[string]any) string {
	value := str(pr, "value")
	if value == "" {
		return str(pr, "placeholder")
	}
	options, _ := pr["options"].([]any)
	for _, o := range options {
		if m, ok := o.(map[string]any); ok && str(m, "value") == value {
			if label := str(m, "label"); label != "" {
				return label
			}
			break
		}
	}
	return value
}

// problem reports whether a field's error or warning is set.
func problem(v any) bool {
	if b, ok := v.(bool); ok {
		return b
	}
	return spansText(v) != ""
}

// fieldLines is a field's label (with * when required), its controls, then
// its error, warning or hint.
func (p plainer) fieldLines(t *tree) []string {
	pr := t.props
	label := spansText(pr["label"])
	if required, _ := pr["required"].(bool); required && label != "" {
		label += " *"
	}
	lines := append(nonEmpty(label), p.children(t)...)
	note := spansText(pr["hint"])
	switch {
	case problem(pr["error"]):
		note = spansText(pr["error"])
	case problem(pr["warning"]):
		note = spansText(pr["warning"])
	}
	return append(lines, nonEmpty(note)...)
}

// otpText is an otp's code padded with _ to its length, masked when asked.
func otpText(pr map[string]any) string {
	n := 6
	if f, ok := num(pr["length"]); ok {
		n = min(max(int(f), 1), 12)
	}
	cells := []rune(str(pr, "text"))
	if len(cells) > n {
		cells = cells[:n]
	}
	mask := rune(0)
	switch m := pr["mask"].(type) {
	case bool:
		if m {
			mask = '•'
		}
	case string:
		if r, _ := utf8.DecodeRuneInString(m); m != "" {
			mask = r
		}
	}
	if mask != 0 {
		for i := range cells {
			cells[i] = mask
		}
	}
	return string(cells) + strings.Repeat("_", n-len(cells))
}

// labelOrID is an entry's label, else its id.
func labelOrID(m map[string]any) string {
	if s := spansText(m["label"]); s != "" {
		return s
	}
	return spansText(m["id"])
}

// pagerText is "Page N", with " of M" when the page count is known.
func pagerText(pr map[string]any) string {
	page := 1
	if f, ok := num(pr["page"]); ok && f >= 1 {
		page = int(f)
	}
	pages := 0
	if f, ok := num(pr["pages"]); ok && f > 0 {
		pages = int(f)
	} else if total, ok := num(pr["total"]); ok {
		if size, ok := num(pr["size"]); ok && size > 0 {
			pages = int(math.Ceil(total / size))
		}
	}
	if pages > 0 {
		return fmt.Sprintf("Page %d of %d", page, pages)
	}
	return fmt.Sprintf("Page %d", page)
}

// datesText is a calendar's or date's value: the day, or from – to.
func datesText(v any) string {
	switch x := v.(type) {
	case string:
		return x
	case []any:
		if len(x) == 0 {
			return ""
		}
		from, _ := x[0].(string)
		to := ""
		if len(x) > 1 {
			to, _ = x[1].(string)
		}
		if to == "" {
			return from
		}
		return from + " – " + to
	}
	return ""
}

// navLine is a nav's items with the active one bracketed, as tabs.
func navLine(pr map[string]any) []string {
	items, _ := pr["items"].([]any)
	active := str(pr, "active")
	var parts []string
	for _, it := range items {
		m, _ := it.(map[string]any)
		on := active != "" && spansText(m["id"]) == active
		subs, _ := m["items"].([]any)
		for _, s := range subs {
			if sm, _ := s.(map[string]any); active != "" && spansText(sm["id"]) == active {
				on = true
			}
		}
		label := labelOrID(m)
		if on {
			label = "[" + label + "]"
		}
		parts = append(parts, label)
	}
	return nonEmpty(strings.Join(parts, " | "))
}

// messageLines is a message's author and time over its indented body,
// reactions and status.
func (p plainer) messageLines(t *tree) []string {
	pr := t.props
	body := p.children(t)
	var chips []string
	reactions, _ := pr["reactions"].([]any)
	for _, r := range reactions {
		m, _ := r.(map[string]any)
		text := str(m, "text")
		if text == "" {
			continue
		}
		if c, ok := num(m["count"]); ok && c > 0 {
			text += " " + fmtNum(c)
		}
		chips = append(chips, text)
	}
	body = append(body, nonEmpty(strings.Join(chips, "  "))...)
	body = append(body, nonEmpty(str(pr, "status"))...)
	return append(nonEmpty(join(" ", str(pr, "author"), str(pr, "time"))), indent(body)...)
}

// traceLines is a trace's head, its lines as bullets with each line's
// content indented under it, then the rest of its children.
func (p plainer) traceLines(t *tree) []string {
	pr := t.props
	text := spansText(pr["title"])
	if text == "" {
		text = spansText(pr["summary"])
	}
	lines := nonEmpty(join(" ", text, parenStatus(pr)))
	byKey := map[string]*tree{}
	for _, c := range t.children {
		if k := str(c.props, "key"); k != "" {
			byKey[k] = c
		}
	}
	used := map[*tree]bool{}
	list, _ := pr["lines"].([]any)
	for i, l := range list {
		m, _ := l.(map[string]any)
		lines = append(lines, "- "+join(" ", spansText(m["title"]), spansText(m["target"]), parenStatus(m)))
		id := str(m, "id")
		if id == "" {
			id = strconv.Itoa(i)
		}
		if c, ok := byKey[id]; ok && !used[c] {
			used[c] = true
			lines = append(lines, indent(indent(p.node(c)))...)
		}
	}
	for _, c := range t.children {
		if !used[c] {
			lines = append(lines, p.node(c)...)
		}
	}
	return lines
}

// parenStatus is a status in parentheses, or "".
func parenStatus(props map[string]any) string {
	if s := str(props, "status"); s != "" {
		return "(" + s + ")"
	}
	return ""
}

// stackLines is a stacktrace's "type: message" over "  at fn (file:line:col)"
// per frame.
func stackLines(pr map[string]any) []string {
	e, _ := pr["error"].(map[string]any)
	lines := nonEmpty(join(": ", str(e, "type"), str(e, "message")))
	frames, _ := pr["frames"].([]any)
	for _, f := range frames {
		m, _ := f.(map[string]any)
		fn := str(m, "fn")
		if fn == "" {
			fn = "<anonymous>"
		}
		if loc := str(m, "file"); loc != "" {
			for _, k := range []string{"line", "col"} {
				n, ok := num(m[k])
				if !ok {
					break
				}
				loc += ":" + fmtNum(n)
			}
			fn += " (" + loc + ")"
		}
		lines = append(lines, "  at "+fn)
	}
	return lines
}

// questionLines is each question's prompt over its choices as bullets.
func questionLines(pr map[string]any) []string {
	var lines []string
	items, _ := pr["items"].([]any)
	for _, it := range items {
		m, _ := it.(map[string]any)
		if id, ok := m["id"].(string); !ok || id == "" {
			continue
		}
		lines = append(lines, str(m, "prompt"))
		choices, _ := m["choices"].([]any)
		for _, c := range choices {
			label, _ := c.(string)
			if cm, ok := c.(map[string]any); ok {
				label = str(cm, "label")
				if label == "" {
					label = str(cm, "value")
				}
			}
			lines = append(lines, "  - "+label)
		}
	}
	return lines
}
