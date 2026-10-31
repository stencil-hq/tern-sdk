package tern

import (
	"strings"
	"testing"
	"unicode"
)

// el is an el node of tag.
func el(tag string, props map[string]any, children ...Element) Node {
	p := map[string]any{"tag": tag}
	for k, v := range props {
		p[k] = v
	}
	return Node{Kind: KindEl, Props: p, Children: children}
}

func TestPlain(t *testing.T) {
	cases := []struct {
		name string
		view View
		want string
	}{
		{"empty view", View{}, ""},
		{"card head over an indented body", Main(Node{Kind: KindCard, Props: map[string]any{"head": "Deploy", "status": "running"},
			Children: []Element{Node{Kind: KindMd, Props: map[string]any{"text": "line 1\nline 2"}}}}),
			"Deploy (running)\n  line 1\n  line 2\n"},
		{"row on one line", Main(Node{Kind: KindRow, Children: []Element{
			Node{Kind: KindBadge, Props: map[string]any{"text": "v2"}},
			Node{Kind: KindText, Props: map[string]any{"spans": []any{map[string]any{"t": "a"}, map[string]any{"t": "b", "s": "muted"}}}},
		}}), "[v2] ab\n"},
		{"progress bar", Main(Node{Kind: KindProgress, Props: map[string]any{"value": 0.5, "label": "half"}}),
			"[##########----------] 50% half\n"},
		{"indeterminate progress", Main(Node{Kind: KindProgress}), "[--------------------]\n"},
		{"ansi escapes stripped", Main(Node{Kind: KindAnsi, Props: map[string]any{"text": "\x1b[31mred\x1b[0m ok\x1b]8;;x\x1b\\"}}), "red ok\n"},
		{"kv aligned", Main(Node{Kind: KindKV, Props: map[string]any{"items": []any{
			map[string]any{"k": "service", "v": "api"}, map[string]any{"k": "region", "v": "eu"}}}}),
			"service  api\nregion   eu\n"},
		{"table aligned with end columns right", Main(Node{Kind: KindTable, Props: map[string]any{
			"cols": []any{map[string]any{"id": "n", "head": "Name"}, map[string]any{"id": "s", "head": "Size", "align": "end"}},
			"rows": []any{map[string]any{"id": "a", "cells": map[string]any{"n": "a.go", "s": "12K"}},
				map[string]any{"id": "b", "cells": map[string]any{"n": "README.md", "s": "4"}}}}}),
			"Name       Size\na.go        12K\nREADME.md     4\n"},
		{"list as bullets", Main(Node{Kind: KindList, Children: []Element{
			Node{Kind: KindItem, Props: map[string]any{"label": "one", "value": "1"}},
			Node{Kind: KindItem, Props: map[string]any{"label": "two", "hidden": true}},
		}}), "- one  1\n"},
		{"el blocks and inlines", Main(el("form", nil,
			el("p", map[string]any{"text": "Which size?"}),
			el("label", nil, el("input", map[string]any{"type": "radio", "checked": true}), el("span", map[string]any{"text": "Small"})),
			el("ul", nil, el("li", map[string]any{"text": "x"})),
			el("button", map[string]any{"text": "Create"}),
		)), "Which size?\n(*) Small\n  - x\n[Create]\n"},
		{"el table aligns number columns right", Main(el("table", nil,
			el("tr", nil, el("th", map[string]any{"text": "Name"}), el("th", map[string]any{"text": "Size"})),
			el("tr", nil, el("td", map[string]any{"text": "Cargo.toml"}), el("td", map[string]any{"text": "2.1K"})),
			el("tr", nil, el("td", map[string]any{"text": "src/"}), el("td", nil)),
			el("tr", nil, el("td", map[string]any{"text": "build.rs"}), el("td", map[string]any{"text": "312"})),
		)), "Name        Size\nCargo.toml  2.1K\nsrc/\nbuild.rs     312\n"},
		{"checklist", Main(Node{Kind: KindChecklist, Props: map[string]any{"phases": []any{map[string]any{"items": []any{
			map[string]any{"id": "a", "text": "plan", "status": "done"}, map[string]any{"id": "b", "text": "ship"}}}}}}),
			"[x] plan\n[ ] ship\n"},
		{"elapsed clock", Main(Node{Kind: KindElapsed, Props: map[string]any{"age": 65000, "format": "clock"}}), "1:05\n"},
		{"status groups", Main(Node{Kind: KindStatus, Children: []Element{
			Node{Kind: KindSeg, Props: map[string]any{"text": "right", "side": "right"}},
			Node{Kind: KindSeg, Props: map[string]any{"text": "left"}},
		}}), "left  right\n"},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			if got := Plain(c.view, 40); got != c.want {
				t.Fatalf("got\n%q\nwant\n%q", got, c.want)
			}
		})
	}
}

func TestPlainSanitizesEveryOutputPath(t *testing.T) {
	const dirty = "\x1b[31mclean\x1b[0m\x1b]8;;https://example.test\x1b\\\x1b]8;;\x07\x1bPignored\x1b\\\u009b2J\u009dignored\u009c\x00\x07\x7f\u0085"
	for _, c := range []struct {
		kind  string
		props map[string]any
	}{
		{KindText, map[string]any{"text": dirty}},
		{KindText, map[string]any{"spans": []any{map[string]any{"t": dirty}}}},
		{KindMd, map[string]any{"text": dirty}},
		{KindCode, map[string]any{"text": dirty}},
		{KindMath, map[string]any{"text": dirty}},
		{KindEditor, map[string]any{"text": dirty}},
		{KindInput, map[string]any{"placeholder": dirty, "prompt": dirty}},
		{KindAnsi, map[string]any{"text": dirty}},
		{KindRows, map[string]any{"lines": []string{dirty}}},
		{KindCard, map[string]any{"head": dirty, "status": dirty}},
		{KindSection, map[string]any{"head": dirty}},
		{KindOverlay, map[string]any{"head": dirty}},
		{KindTool, map[string]any{"title": dirty, "target": dirty, "note": dirty, "meta": []string{dirty}}},
		{KindAgent, map[string]any{"name": dirty, "agent": dirty, "task": dirty}},
		{KindRule, map[string]any{"label": dirty}},
		{KindDiff, map[string]any{"text": dirty}},
		{KindDiff, map[string]any{"hunks": []any{map[string]any{"lines": []string{dirty}}}}},
		{KindKV, map[string]any{"items": []any{map[string]any{"k": dirty, "v": dirty}}}},
		{KindTable, map[string]any{"cols": []any{map[string]any{"id": "x", "head": dirty}},
			"rows": []any{map[string]any{"cells": map[string]any{"x": dirty}}}}},
		{KindTree, map[string]any{"nodes": []any{map[string]any{"label": dirty}}}},
		{KindBadge, map[string]any{"text": dirty}},
		{KindKbd, map[string]any{"keys": []string{dirty}}},
		{KindIcon, map[string]any{"aria": dirty}},
		{KindImage, map[string]any{"alt": dirty}},
		{KindMedia, map[string]any{"path": dirty, "rev": dirty}},
		{KindList, map[string]any{"empty": dirty}},
		{KindItem, map[string]any{"label": dirty, "detail": dirty, "value": dirty}},
		{KindTabs, map[string]any{"items": []any{map[string]any{"label": dirty}}}},
		{KindPicker, map[string]any{"title": dirty, "items": []any{map[string]any{"id": "x", "label": dirty}}}},
		{KindSpinner, map[string]any{"label": dirty}},
		{KindShimmer, map[string]any{"text": dirty}},
		{KindRate, map[string]any{"unit": dirty}},
		{KindProgress, map[string]any{"label": dirty}},
		{KindMeter, map[string]any{"label": dirty, "total": dirty}},
		{KindChart, map[string]any{"summary": dirty}},
		{KindChart, map[string]any{"series": []any{map[string]any{"label": dirty}}}},
		{KindEffort, map[string]any{"level": dirty}},
		{KindSeg, map[string]any{"text": dirty}},
		{KindToast, map[string]any{"text": dirty, "sub": dirty}},
		{KindChecklist, map[string]any{"phases": []any{map[string]any{"title": dirty,
			"items": []any{map[string]any{"text": dirty, "note": dirty}}}}}},
		{KindPrefs, map[string]any{"title": dirty, "sections": []any{map[string]any{
			"title": dirty, "rows": []any{map[string]any{"label": dirty,
				"control": map[string]any{"k": "text", "value": dirty}}}}}}},
		{KindEl, map[string]any{"tag": "p", "text": dirty}},
		{KindButton, map[string]any{"text": dirty}},
		{KindStats, map[string]any{"items": []any{map[string]any{"label": dirty, "value": dirty, "sub": dirty}}}},
		{KindSelect, map[string]any{"label": dirty, "value": dirty}},
		{KindSlider, map[string]any{"label": dirty, "value": 1, "unit": dirty}},
		{KindOtp, map[string]any{"text": dirty, "length": 12}},
		{KindBreadcrumb, map[string]any{"items": []any{map[string]any{"id": dirty}}}},
		{KindMessage, map[string]any{"author": dirty, "status": dirty}},
		{KindTrace, map[string]any{"title": dirty, "lines": []any{map[string]any{"title": dirty}}}},
		{KindStacktrace, map[string]any{"error": map[string]any{"type": dirty}}},
		{KindQuestions, map[string]any{"items": []any{map[string]any{"id": "q", "prompt": dirty}}}},
		{KindCard, map[string]any{"head": dirty, "meta": dirty}},
		{KindTable, map[string]any{"cols": []any{map[string]any{"id": "x"}}, "empty": dirty}},
		{KindTable, map[string]any{"cols": []any{map[string]any{"id": "a"}, map[string]any{"id": "b"},
			map[string]any{"id": "c"}}, "rows": []any{map[string]any{"cells": map[string]any{
			"a": map[string]any{"text": dirty, "dot": "success"},
			"b": map[string]any{"badge": map[string]any{"text": dirty}},
			"c": map[string]any{"button": map[string]any{"text": dirty, "act": "x"}}}}}}},
	} {
		t.Run(c.kind, func(t *testing.T) {
			got := Plain(Main(Node{Kind: c.kind, Props: c.props}), 80)
			if !strings.Contains(got, "clean") {
				t.Fatalf("lost readable text: %q", got)
			}
			for _, r := range got {
				if unicode.IsControl(r) && r != '\n' && r != '\t' {
					t.Errorf("control U+%04X remains in %q", r, got)
				}
			}
			if strings.Contains(got, "ignored") || strings.Contains(got, "example.test") {
				t.Errorf("escape payload remains: %q", got)
			}
		})
	}
	got := Plain(Main(Node{Kind: KindText, Props: map[string]any{"text": "a\tb\nc"}}), 80)
	if got != "a\tb\nc\n" {
		t.Errorf("allowed whitespace changed: %q", got)
	}
}

func TestPlainDisplayCellAlignment(t *testing.T) {
	cases := []struct {
		name string
		view View
		want string
	}{
		{"kv", Main(Node{Kind: KindKV, Props: map[string]any{"items": []any{
			map[string]any{"k": "\x1b[31m界\x1b[0m", "v": "1"},
			map[string]any{"k": "e\u0301", "v": "2"},
			map[string]any{"k": "Ａ", "v": "3"},
		}}}), "界  1\ne\u0301   2\nＡ  3\n"},
		{"table", Main(Node{Kind: KindTable, Props: map[string]any{
			"cols": []any{map[string]any{"id": "a", "align": "end"}, map[string]any{"id": "b"}},
			"rows": []any{
				map[string]any{"cells": map[string]any{"a": "界", "b": "1"}},
				map[string]any{"cells": map[string]any{"a": "e\u0301", "b": "2"}},
			},
		}}), "界  1\n e\u0301  2\n"},
		{"el table", Main(el("table", nil,
			el("tr", nil, el("td", map[string]any{"text": "界"}), el("td", map[string]any{"text": "1"})),
			el("tr", nil, el("td", map[string]any{"text": "e\u0301"}), el("td", map[string]any{"text": "2"})),
		)), "界  1\ne\u0301   2\n"},
		{"rule", Main(Node{Kind: KindRule, Props: map[string]any{"label": "\x1b[31m界e\u0301\x1b[0m"}}),
			"── 界e\u0301 ─────\n"},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			if got := Plain(c.view, 12); got != c.want {
				t.Errorf("got %q, want %q", got, c.want)
			}
		})
	}
}

func TestPlainElapsedFloorsTenths(t *testing.T) {
	for _, c := range []struct {
		props map[string]any
		want  string
	}{
		{map[string]any{}, "0.0s\n"},
		{map[string]any{"age": 99}, "0.0s\n"},
		{map[string]any{"age": 12399}, "12.3s\n"},
		{map[string]any{"age": 59999}, "59.9s\n"},
		{map[string]any{"age": 65000}, "1m 05s\n"},
		{map[string]any{"age": 3723000}, "1h 02m\n"},
		{map[string]any{"age": -12399}, "0.0s\n"},
		{map[string]any{"age": 99999, "stopped": 12399}, "12.3s\n"},
		{map[string]any{"age": 12399, "format": "clock"}, "0:12\n"},
		{map[string]any{"age": 3723000, "format": "clock"}, "1:02:03\n"},
	} {
		if got := Plain(Main(Node{Kind: KindElapsed, Props: c.props}), 80); got != c.want {
			t.Errorf("%v: got %q, want %q", c.props, got, c.want)
		}
	}
}

func TestPlainOrderedLists(t *testing.T) {
	view := Main(el("ol", nil,
		el("li", map[string]any{"text": "first\ncontinued"},
			el("ul", nil, el("li", map[string]any{"text": "bullet"}))),
		el("li", map[string]any{"text": "hidden", "hidden": true}),
		el("li", map[string]any{"text": "second"},
			el("ol", nil, el("li", map[string]any{"text": "nested"}))),
	), el("ol", nil, el("li", map[string]any{"text": "restart"})))
	want := "  1. first\n     continued\n       - bullet\n  2. second\n       1. nested\n  1. restart\n"
	if got := Plain(view, 80); got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}

func TestPlainWrappedMeterCells(t *testing.T) {
	for _, c := range []struct {
		meter map[string]any
		want  string
	}{
		{map[string]any{"value": 0.5}, "[##########----------] 50%\n"},
		{map[string]any{"parts": []any{map[string]any{"value": 0.25}, map[string]any{"value": 0.5}}},
			"[###############-----] 75%\n"},
		{map[string]any{"value": 0, "parts": []any{map[string]any{"value": 0.75}}},
			"[--------------------] 0%\n"},
		{map[string]any{}, "[--------------------]\n"},
	} {
		table := Node{Kind: KindTable, Props: map[string]any{
			"cols": []any{map[string]any{"id": "m"}},
			"rows": []any{map[string]any{"cells": map[string]any{"m": map[string]any{"meter": c.meter}}}},
		}}
		if got := Plain(Main(table), 80); got != c.want {
			t.Errorf("got %q, want %q", got, c.want)
		}
	}
}

func TestPlainRuleSpansCols(t *testing.T) {
	got := Plain(Main(Node{Kind: KindRule}, Node{Kind: KindRule, Props: map[string]any{"label": "log"}}), 12)
	lines := strings.Split(strings.TrimSuffix(got, "\n"), "\n")
	if len(lines) != 2 || lines[0] != strings.Repeat("─", 12) || !strings.HasPrefix(lines[1], "── log ") {
		t.Fatalf("%q", got)
	}
}

func TestPlainSlotsAndSparks(t *testing.T) {
	card := Node{Kind: KindOverlay, Props: map[string]any{"head": "Deploy", "meta": "2 left"},
		Children: []Element{Node{Kind: KindText, Props: map[string]any{"text": "body"}}},
		Slots: map[string]Element{
			"end":  Node{Kind: KindBadge, Props: map[string]any{"text": "new"}},
			"foot": Node{Kind: KindButton, Props: map[string]any{"text": "OK"}},
		}}
	table := Node{Kind: KindTable, Props: map[string]any{"cols": []any{map[string]any{"id": "s"}},
		"rows": []any{map[string]any{"cells": map[string]any{"s": map[string]any{
			"spark": map[string]any{"values": []any{0, 7, 3.5}}}}}}}}
	got := Plain(Main(card, table), 80)
	if want := "Deploy  2 left  [new]\n  body\n  [OK]\n▁█▅\n"; got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}

func TestPlainNewKinds(t *testing.T) {
	n := func(kind string, props map[string]any, children ...Element) Node {
		return Node{Kind: kind, Props: props, Children: children}
	}
	text := func(s, key string) Node {
		p := map[string]any{"text": s}
		if key != "" {
			p["key"] = key
		}
		return n(KindText, p)
	}
	cases := []struct {
		name string
		node Node
		want string
	}{
		{"select shows the option's label", n(KindSelect, map[string]any{"label": "Model", "value": "haiku",
			"options": []any{"opus", map[string]any{"value": "haiku", "label": "Haiku"}}}), "Model: Haiku\n"},
		{"select without a value shows the placeholder", n(KindSelect, map[string]any{"placeholder": "Pick"}), "Pick\n"},
		{"switch on", n(KindSwitch, map[string]any{"on": true, "label": "Wrap"}), "[x] Wrap\n"},
		{"switch off", n(KindSwitch, nil), "[ ]\n"},
		{"slider unit attaches", n(KindSlider, map[string]any{"label": "Opacity", "value": 72, "unit": "%"}), "Opacity: 72%\n"},
		{"slider range", n(KindSlider, map[string]any{"value": []any{200, 1200}, "unit": "ms"}), "200–1200 ms\n"},
		{"slider without a value", n(KindSlider, map[string]any{"label": "Size"}), "Size\n"},
		{"field error wins over hint", n(KindField, map[string]any{"label": "Port", "required": true, "hint": "1–65535",
			"error": "Bad port"}, n(KindInput, map[string]any{"text": "0"})), "Port *\n0\nBad port\n"},
		{"field flagged hides the hint", n(KindField, map[string]any{"label": "Port", "hint": "h", "warning": true},
			n(KindInput, map[string]any{"text": "0"})), "Port\n0\n"},
		{"field hint", n(KindField, map[string]any{"hint": "h"}), "h\n"},
		{"otp pads to its length", n(KindOtp, map[string]any{"text": "12", "length": 4}), "12__\n"},
		{"otp masks and cuts at the length", n(KindOtp, map[string]any{"text": "1234567", "mask": true}), "••••••\n"},
		{"otp masks with a character", n(KindOtp, map[string]any{"text": "12", "mask": "*#"}), "**____\n"},
		{"buttons on one line", n(KindButtons, nil, n(KindButton, map[string]any{"text": "Undo"}),
			n(KindButton, map[string]any{"text": "Redo"})), "[Undo] [Redo]\n"},
		{"avatar", n(KindAvatar, map[string]any{"text": "Cara"}), "Cara\n"},
		{"avatars", n(KindAvatars, map[string]any{"items": []any{map[string]any{"text": "Ann"}, map[string]any{"text": "Bo"}},
			"label": "2 reviewers"}), "Ann, Bo 2 reviewers\n"},
		{"breadcrumb", n(KindBreadcrumb, map[string]any{"items": []any{map[string]any{"id": "acme", "label": "Acme"},
			map[string]any{"id": "k8s"}}}), "Acme › k8s\n"},
		{"breadcrumb path", n(KindBreadcrumb, map[string]any{"variant": "path", "items": []any{map[string]any{"id": "src"},
			map[string]any{"id": "main.go"}}}), "src / main.go\n"},
		{"pager counts pages", n(KindPager, map[string]any{"page": 2, "total": 312, "size": 25}), "Page 2 of 13\n"},
		{"pager pages win", n(KindPager, map[string]any{"pages": 4, "total": 312, "size": 25}), "Page 1 of 4\n"},
		{"pager alone", n(KindPager, nil), "Page 1\n"},
		{"empty", n(KindEmpty, map[string]any{"title": "No deploys", "text": "Push to deploy"},
			n(KindButton, map[string]any{"text": "Deploy"})), "No deploys\nPush to deploy\n[Deploy]\n"},
		{"skeleton draws nothing", n(KindCol, nil, n(KindSkeleton, nil), text("x", "")), "x\n"},
		{"calendar range waiting for its end", n(KindCalendar, map[string]any{"value": []any{"2026-10-19", nil}}), "2026-10-19\n"},
		{"date range", n(KindDate, map[string]any{"value": []any{"2026-10-19", "2026-10-30"}}), "2026-10-19 – 2026-10-30\n"},
		{"date placeholder", n(KindDate, map[string]any{"placeholder": "Pick a date"}), "Pick a date\n"},
		{"menu", n(KindMenu, map[string]any{"text": "Sort", "items": []any{"asc"}}), "[Sort]\n"},
		{"menu by its title", n(KindMenu, map[string]any{"title": "More"}), "[More]\n"},
		{"menubar", n(KindMenubar, map[string]any{"menus": []any{map[string]any{"id": "file", "label": "File"},
			map[string]any{"id": "view"}}}), "File  view\n"},
		{"nav brackets the active sub-item's parent", n(KindNav, map[string]any{"active": "api", "items": []any{
			map[string]any{"id": "home", "label": "Home"},
			map[string]any{"id": "docs", "label": "Docs", "items": []any{map[string]any{"id": "api"}}}}}), "Home | [Docs]\n"},
		{"carousel shows its slide", n(KindCarousel, map[string]any{"index": 5}, text("a", ""), text("b", "")), "b\n"},
		{"message", n(KindMessage, map[string]any{"author": "Sam", "time": "14:02", "status": "Read",
			"reactions": []any{map[string]any{"text": "♥", "count": 2}, map[string]any{"text": "👍", "count": 0}}},
			n(KindMd, map[string]any{"text": "hi"})), "Sam 14:02\n  hi\n  ♥ 2  👍\n  Read\n"},
		{"attachment", n(KindAttachment, map[string]any{"name": "a.pdf"}), "[attachment: a.pdf]\n"},
		{"trace lines with their content", n(KindTrace, map[string]any{"title": "Worked", "status": "done",
			"lines": []any{map[string]any{"id": "s1", "title": "Search", "target": "go.dev", "status": "done"},
				map[string]any{"title": "Read"}}}, text("body", ""), text("found", "s1"), text("read it", "1")),
			"Worked (done)\n- Search go.dev (done)\n    found\n- Read\n    read it\nbody\n"},
		{"stacktrace", n(KindStacktrace, map[string]any{"error": map[string]any{"type": "TypeError", "message": "x"},
			"frames": []any{map[string]any{"fn": "main", "file": "a.ts", "line": 3, "col": 7}, map[string]any{}}}),
			"TypeError: x\n  at main (a.ts:3:7)\n  at <anonymous>\n"},
		{"suggestions", n(KindSuggestions, map[string]any{"items": []any{map[string]any{"text": "Fix it"},
			map[string]any{"detail": "no text"}}}), "- Fix it\n"},
		{"questions", n(KindQuestions, map[string]any{"items": []any{map[string]any{"id": "db", "prompt": "Which DB?",
			"choices": []any{"pg", map[string]any{"value": "my", "label": "MySQL"}}}, map[string]any{"prompt": "no id"}}}),
			"Which DB?\n  - pg\n  - MySQL\n"},
		{"secret input", n(KindInput, map[string]any{"text": "abc", "secret": true, "reveal": true}), "•••\n"},
		{"cited text", n(KindText, map[string]any{"spans": []any{
			map[string]any{"t": "Go", "cite": []any{map[string]any{"href": "a"}}},
			map[string]any{"t": " and Rust", "cite": map[string]any{"href": "b", "n": 1}},
			map[string]any{"t": ".", "cite": []any{map[string]any{"href": "a"}, map[string]any{"href": "c"}, map[string]any{}}},
		}}), "Go[2] and Rust[1].[2][3]\n"},
		{"line chart sparkline skips gaps", n(KindChart, map[string]any{"kind": "line",
			"series": []any{1, nil, map[string]any{"value": 3}, 2}}), "▁█▅\n"},
		{"area chart series", n(KindChart, map[string]any{"kind": "area", "series": []any{
			map[string]any{"label": "p50", "values": []any{1, 3}}, map[string]any{"values": []any{2, 2}}}}), "p50 ▁█\n▁▁\n"},
		{"donut segments", n(KindChart, map[string]any{"kind": "donut", "unit": "%", "series": []any{
			map[string]any{"label": "ok", "value": 60}, 40, "x"}}), "ok 60%  40%\n"},
		{"donut unit after a space", n(KindChart, map[string]any{"kind": "donut", "unit": "ms",
			"series": []any{map[string]any{"label": "p50", "value": 3}}}), "p50 3 ms\n"},
		{"summary wins", n(KindChart, map[string]any{"kind": "line", "summary": "flat", "series": []any{1, 2}}), "flat\n"},
	}
	for _, c := range cases {
		if got := Plain(Main(c.node), 80); got != c.want {
			t.Errorf("%s: got %q, want %q", c.name, got, c.want)
		}
	}
	rule := Plain(Main(Node{Kind: KindRule, Props: map[string]any{"label": "Checkpoint", "time": "14:02", "variant": "note"}}), 40)
	if want := "── Checkpoint · 14:02 " + strings.Repeat("─", 40-18-4) + "\n"; rule != want {
		t.Errorf("rule %q, want %q", rule, want)
	}
}
