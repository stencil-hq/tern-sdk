package ui_test

import (
	"encoding/json"
	"testing"

	tern "github.com/stencil-hq/tern-sdk/go"
	"github.com/stencil-hq/tern-sdk/go/el"
	"github.com/stencil-hq/tern-sdk/go/ui"
)

// props is el's node's props as compact JSON.
func props(t *testing.T, e tern.Element) string {
	t.Helper()
	b, err := tern.Marshal(e.Node().Props)
	if err != nil {
		t.Fatal(err)
	}
	return string(b)
}

func TestZeroPropsStayAbsent(t *testing.T) {
	for name, e := range map[string]tern.Element{
		"card": ui.Card{}, "text": ui.Text{}, "progress": ui.Progress{}, "editor": ui.Editor{},
		"tool": ui.Tool{}, "div": el.Div{}, "meter": ui.Meter{}, "overlay": ui.Overlay{},
	} {
		got := props(t, e)
		want := "null"
		if name == "div" {
			want = `{"tag":"div"}`
		}
		if got != want {
			t.Errorf("%s: %s", name, got)
		}
	}
}

func TestPointersSayZero(t *testing.T) {
	cases := []struct {
		e    tern.Element
		want string
	}{
		{ui.Progress{Value: new(0.0)}, `{"value":0}`},
		{ui.Code{Text: "x", Start: new(0)}, `{"start":0,"text":"x"}`},
		{ui.Editor{Field: ui.Field{Text: "ab", Cursor: new(0)}}, `{"cursor":0,"text":"ab"}`},
		{ui.Col{Common: ui.Common{Shrink: new(0.0), Grow: 1}}, `{"grow":1,"shrink":0}`},
		{ui.Tool{Collapsible: true, Collapsed: new(false)}, `{"collapsed":false,"collapsible":true}`},
	}
	for _, c := range cases {
		if got := props(t, c.e); got != c.want {
			t.Errorf("got %s, want %s", got, c.want)
		}
	}
}

func TestRichAndValueTypes(t *testing.T) {
	card := ui.Card{
		Head:    ui.Spans(ui.Span{T: "Deploy", S: "strong"}, ui.Span{T: " api", Fx: ui.FxShimmer}),
		Status:  ui.Running,
		Preview: ui.PreviewLines(3),
		Common: ui.Common{Key: "c", Tone: ui.ToneSuccess, Min: &ui.Bounds{W: ui.Ch(20)},
			Max: &ui.Bounds{H: ui.Frac(0.5)}, Basis: ui.BasisContent},
	}
	want := `{"basis":"content","head":[{"s":"strong","t":"Deploy"},{"fx":"shimmer","t":" api"}],"key":"c","max":{"h":0.5},"min":{"w":"20ch"},"preview":{"lines":3},"status":"running","tone":"success"}`
	if got := props(t, card); got != want {
		t.Errorf("card\n got %s\nwant %s", got, want)
	}
	if got := props(t, ui.Rule{Label: ui.T("log")}); got != `{"label":"log"}` {
		t.Errorf("rule %s", got)
	}
	if got := props(t, ui.Overlay{Anchor: ui.AtCaret("ed"), Size: ui.SizeSM}); got != `{"anchor":{"caret":"ed"},"size":"sm"}` {
		t.Errorf("overlay %s", got)
	}
	table := ui.Table{
		Cols: []ui.Column{{ID: "n", Head: ui.T("Name")}, {ID: "u", Align: ui.AlignEnd}},
		Rows: []ui.TableRow{{ID: "r", Cells: map[string]ui.Cell{"n": ui.T("a"), "u": ui.MeterCell{Value: new(0.5)}}}},
	}
	if got := props(t, table); got != `{"cols":[{"head":"Name","id":"n"},{"align":"end","id":"u"}],"rows":[{"cells":{"n":"a","u":{"meter":{"value":0.5}}},"id":"r"}]}` {
		t.Errorf("table %s", got)
	}
}

func TestPrefsControlsCarryTheirKind(t *testing.T) {
	p := ui.Prefs{Sections: []ui.PrefsSection{{Rows: []ui.PrefsRow{
		{ID: "a", Control: ui.PrefSwitch{On: true}},
		{ID: "b", Control: ui.PrefAction{}},
		{ID: "c", Control: ui.PrefChoice{Value: "x", Options: []ui.PrefOption{{Value: "x"}}}},
	}}}}
	want := `{"sections":[{"rows":[{"control":{"k":"switch","on":true},"id":"a"},{"control":{"k":"action"},"id":"b"},{"control":{"k":"choice","options":[{"value":"x"}],"value":"x"},"id":"c"}]}]}`
	if got := props(t, p); got != want {
		t.Errorf("prefs\n got %s\nwant %s", got, want)
	}
}

func TestPickerActionsAndOrder(t *testing.T) {
	p := ui.Picker{
		Query:   new(""),
		Order:   []ui.OrderEntry{{Group: "g", Label: ui.T("Group"), Count: new(2)}, {ID: "a"}},
		Actions: []ui.PickerAction{{ID: "open", Primary: true}},
	}
	want := `{"actions":[{"id":"open","primary":true}],"order":[{"count":2,"group":"g","label":"Group"},"a"],"query":""}`
	b, _ := json.Marshal(p.Node().Props)
	var v any
	_ = json.Unmarshal(b, &v)
	b, _ = json.Marshal(v)
	if string(b) != want {
		t.Errorf("picker\n got %s\nwant %s", b, want)
	}
}

func TestAgentStates(t *testing.T) {
	for _, state := range []ui.AgentState{
		ui.AgentPending, ui.AgentRunning, ui.AgentDone, ui.AgentFailed,
		ui.AgentAborted, ui.AgentIdle, ui.AgentParked, ui.AgentState("future"),
	} {
		if got := props(t, ui.Agent{Status: state}); got != `{"status":"`+string(state)+`"}` {
			t.Errorf("%s: %s", state, got)
		}
	}
}

func TestSpecializedCommonPropsAndHandlers(t *testing.T) {
	called := 0
	action := func(*tern.Action) { called++ }
	cases := []struct {
		e    tern.Element
		want string
	}{
		{ui.Tool{CommonWithoutTitle: ui.CommonWithoutTitle{
			Key: "tool", Actions: &ui.Actions{Click: "open"}, OnClick: action,
		}, Title: ui.T("Deploy")}, `{"actions":{"click":"open"},"key":"tool","title":"Deploy"}`},
		{ui.Prefs{CommonWithoutTitle: ui.CommonWithoutTitle{
			Key: "prefs", OnClick: action, Props: map[string]any{"title": "Override"},
		}, Title: "Settings"}, `{"key":"prefs","title":"Override"}`},
		{ui.List{CommonWithoutMax: ui.CommonWithoutMax{
			Key: "list", Title: "Tooltip", OnClick: action,
		}, Max: ui.MaxLines(3)}, `{"key":"list","max":{"lines":3},"title":"Tooltip"}`},
		{ui.Picker{PickerCommon: ui.PickerCommon{
			Key: "picker", OnClick: action, Max: &ui.Bounds{H: ui.Frac(0.5)},
		}, Title: ui.T("Choose"), Actions: []ui.PickerAction{{ID: "open"}}},
			`{"actions":[{"id":"open"}],"key":"picker","max":{"h":0.5},"title":"Choose"}`},
	}
	for _, c := range cases {
		if got := props(t, c.e); got != c.want {
			t.Errorf("%T: got %s, want %s", c.e, got, c.want)
		}
		n := c.e.Node()
		if n.OnClick == nil {
			t.Fatalf("%T lost its handler", c.e)
		}
		n.OnClick(&tern.Action{})
	}
	if called != len(cases) {
		t.Errorf("called %d handlers", called)
	}
	for _, value := range []any{nil, []ui.PickerAction{{ID: "override"}}} {
		p := ui.Picker{
			PickerCommon: ui.PickerCommon{Props: map[string]any{"actions": value}},
			Actions:      []ui.PickerAction{{ID: "original"}},
		}
		want := `{"actions":[{"id":"override"}]}`
		if value == nil {
			want = "null"
		}
		if got := props(t, p); got != want {
			t.Errorf("picker escape hatch: got %s, want %s", got, want)
		}
	}
}

func TestNullableHeatmapCells(t *testing.T) {
	chart := ui.Chart{Cells: [][]*float64{{new(0.0), nil, new(0.75)}}}
	if got := props(t, chart); got != `{"cells":[[0,null,0.75]]}` {
		t.Errorf("chart %s", got)
	}
}

func TestMeterCellsWrapAndRender(t *testing.T) {
	for _, c := range []struct {
		cell ui.MeterCell
		wire string
		text string
	}{
		{ui.MeterCell{}, `{"meter":{}}`, "[--------------------]\n"},
		{ui.MeterCell{Value: new(0.0)}, `{"meter":{"value":0}}`, "[--------------------] 0%\n"},
		{ui.MeterCell{Parts: []ui.Part{{Value: 0.25}, {Value: 0.5}}},
			`{"meter":{"parts":[{"value":0.25},{"value":0.5}]}}`, "[###############-----] 75%\n"},
	} {
		b, err := tern.Marshal(c.cell)
		if err != nil || string(b) != c.wire {
			t.Errorf("cell %s, %v; want %s", b, err, c.wire)
		}
		table := ui.Table{Cols: []ui.Column{{ID: "m"}},
			Rows: []ui.TableRow{{ID: "r", Cells: map[string]ui.Cell{"m": c.cell}}}}
		if got := tern.Plain(tern.Main(table), 80); got != c.text {
			t.Errorf("plain %q; want %q", got, c.text)
		}
	}
}

func TestUntypedPropsOverride(t *testing.T) {
	b := ui.Badge{Text: "x", Common: ui.Common{Tone: ui.ToneAccent, Props: map[string]any{"tone": nil, "future": 1}}}
	if got := props(t, b); got != `{"future":1,"text":"x"}` {
		t.Errorf("badge %s", got)
	}
}

func TestElControls(t *testing.T) {
	in := el.Input{Type: el.Radio, Name: "size", Value: "s", Checked: true}
	if got := props(t, in); got != `{"checked":true,"name":"size","tag":"input","type":"radio","value":"s"}` {
		t.Errorf("input %s", got)
	}
	td := el.Td{Class: "size", Text: "2.1K", Attrs: map[string]any{"colspan": 2}}
	if got := props(t, td); got != `{"attrs":{"colspan":2},"class":"size","tag":"td","text":"2.1K"}` {
		t.Errorf("td %s", got)
	}
	form := el.Form{Children: ui.Nodes(el.P{Text: "q"}, el.Hr{})}.Node()
	if form.Kind != tern.KindEl || len(form.Children) != 2 || form.Children[1].Node().Props["tag"] != "hr" {
		t.Errorf("form %+v", form)
	}
}

func TestHandlersReachTheNode(t *testing.T) {
	clicked := false
	n := el.Button{Text: "Go", Common: ui.Common{OnClick: func(*tern.Action) { clicked = true }}}.Node()
	if n.OnClick == nil {
		t.Fatal("handler lost")
	}
	n.OnClick(&tern.Action{})
	if !clicked {
		t.Fatal("handler not called")
	}
}
