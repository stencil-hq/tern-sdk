package tern

import (
	"encoding/json"
	"fmt"
	"math"
	"regexp"
	"strconv"
	"strings"
	"unicode"

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
	return t
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
		return p.headed(join(" ", spansText(pr["head"]), state), p.children(t))
	case KindSection:
		return p.headed(spansText(pr["head"]), p.children(t))
	case KindOverlay:
		return p.headed(spansText(pr["head"]), p.children(t))
	case KindTool:
		head := join(" ", spansText(pr["title"]), spansText(pr["target"]), metaText(pr["meta"]), spansText(pr["note"]), status(pr))
		return p.headed(head, p.children(t))
	case KindAgent:
		head := join(" ", str(pr, "name"), str(pr, "agent"), spansText(pr["task"]), status(pr))
		return p.headed(head, p.children(t))
	case KindRule:
		label := stripANSI(spansText(pr["label"]))
		if label == "" {
			return []string{strings.Repeat("─", p.cols)}
		}
		rest := max(p.cols-displayWidth(label)-4, 2)
		return []string{"── " + label + " " + strings.Repeat("─", rest)}
	case KindSpacer:
		return []string{""}
	case KindText, KindShimmer, KindSeg:
		return splitLines(richText(pr))
	case KindMd, KindCode, KindMath:
		return splitLines(str(pr, "text"))
	case KindEditor, KindInput:
		text := str(pr, "text")
		if text == "" {
			text = str(pr, "placeholder")
		}
		return splitLines(spansText(pr["prompt"]) + text)
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
		return tableLines(pr)
	case KindTree:
		list, _ := pr["nodes"].([]any)
		return treeLines(list)
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

// cellText is a table cell's text: its spans, or a meter cell's bar.
func cellText(v any) string {
	if cell, ok := v.(map[string]any); ok {
		if meter, ok := cell["meter"].(map[string]any); ok {
			val, known := meterValue(meter)
			return bar(val, known)
		}
	}
	return spansText(v)
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

// chartLines is a chart's summary, or its series values.
func chartLines(pr map[string]any) []string {
	if s := spansText(pr["summary"]); s != "" {
		return []string{s}
	}
	series, _ := pr["series"].([]any)
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
