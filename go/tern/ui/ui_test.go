package ui_test

import (
	"encoding/json"
	"strings"
	"testing"

	"github.com/stencil-hq/tern-sdk/go/tern"
	"github.com/stencil-hq/tern-sdk/go/tern/el"
	"github.com/stencil-hq/tern-sdk/go/tern/ui"
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

func TestJSONCarriesValueAndOpen(t *testing.T) {
	got := props(t, ui.JSON{Value: map[string]any{"a": 1}, Open: 1})
	if want := `{"open":1,"value":{"a":1}}`; got != want {
		t.Errorf("got %s, want %s", got, want)
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

func TestCodeWashAndFolds(t *testing.T) {
	code := ui.Code{
		Text:     "a\nb\nc",
		Nums:     []int{1, 2, 5},
		Marks:    []ui.CodeMark{{Line: 1, Wash: []float64{0.9, 0.3}}},
		Folds:    []ui.CodeFold{{From: 2, To: 3, Note: ui.T("cut"), Tone: ui.ToneError, Hovercard: ui.Hovercard{Element: ui.Text{Text: "why"}}}, {From: 5, To: 6}},
		Overview: true,
	}
	want := `{"folds":[{"from":2,"hovercard":{"k":"text","p":{"text":"why"}},"note":"cut","to":3,"tone":"error"},{"from":5,"to":6}],"marks":[{"line":1,"wash":[0.9,0.3]}],"nums":[1,2,5],"overview":true,"text":"a\nb\nc"}`
	if got := props(t, code); got != want {
		t.Errorf("code\n got %s\nwant %s", got, want)
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

func TestNewKindsAndCells(t *testing.T) {
	cases := []struct {
		e    tern.Element
		want string
	}{
		{ui.Button{Text: "Go", Keys: []string{"r"}, Variant: ui.ButtonPrimary, Size: ui.SizeSM,
			Disabled: ui.Because("busy")}, `{"disabled":"busy","keys":["r"],"size":"sm","text":"Go","variant":"primary"}`},
		{ui.Button{Disabled: ui.Off}, `{"disabled":true}`},
		{ui.Stats{Items: []ui.Stat{{Label: ui.T("Up"), Value: ui.T("3")}}},
			`{"items":[{"label":"Up","value":"3"}]}`},
		{ui.Tabs{Variant: ui.TabsSegmented}, `{"variant":"segmented"}`},
		{ui.Table{Selectable: true, Selected: "a", Empty: ui.T("none"),
			Cols: []ui.Column{{ID: "x", Mono: true}},
			Rows: []ui.TableRow{{ID: "a", Menu: []string{"kill"}, Disabled: ui.Off, Cells: map[string]ui.Cell{
				"b": ui.BadgeCell{Text: "ok", Tone: ui.ToneSuccess},
				"c": ui.ButtonCell{Text: "Go", Act: "go"},
				"s": ui.SparkCell{Values: []float64{1, 2}, Min: new(0.0)},
				"t": ui.TextCell{Text: ui.T("api"), Dot: ui.ToneError},
			}}}},
			`{"cols":[{"id":"x","mono":true}],"empty":"none","rows":[{"cells":{` +
				`"b":{"badge":{"text":"ok","tone":"success"}},"c":{"button":{"act":"go","text":"Go"}},` +
				`"s":{"spark":{"min":0,"values":[1,2]}},"t":{"dot":"error","text":"api"}},` +
				`"disabled":true,"id":"a","menu":["kill"]}],"selectable":true,"selected":"a"}`},
	}
	for _, c := range cases {
		if got := props(t, c.e); got != c.want {
			t.Errorf("got %s\nwant %s", got, c.want)
		}
	}
}

func TestSlotsHoistAsKeyedChildren(t *testing.T) {
	view := tern.Main(
		ui.Card{Common: ui.Common{Key: "c"}, HeadNode: ui.Text{Text: "h", Common: ui.Common{Key: "x"}},
			End: ui.Button{Text: "Go"}, Children: ui.Nodes(ui.Text{Text: "body"})},
		ui.Overlay{Common: ui.Common{Key: "o"}, Head: ui.T("Sheet"), Foot: ui.Button{Text: "OK"}},
	)
	ops, err := tern.NewReconciler("sf").Diff(view)
	if err != nil {
		t.Fatal(err)
	}
	b, err := tern.Marshal(ops)
	if err != nil {
		t.Fatal(err)
	}
	got := string(b)
	for _, want := range []string{
		`"end":"main.c.end","head":"main.c.head"`,
		`{"id":"main.c.0","k":"text"`, `{"id":"main.c.end","k":"button","p":{"key":"end","text":"Go"}}`,
		`{"id":"main.c.head","k":"text","p":{"key":"head","text":"h"}}`,
		`"foot":"main.o.foot","head":"Sheet"`, `{"id":"main.o.foot","k":"button"`,
	} {
		if !strings.Contains(got, want) {
			t.Errorf("missing %s in %s", want, got)
		}
	}
}

func TestControlKinds(t *testing.T) {
	cases := []struct {
		e    tern.Element
		want string
	}{
		{ui.Select{Value: "haiku", Variant: ui.SelectQuiet, Label: "Model", Search: true, Cursor: new(0),
			Options: []ui.SelectOption{{Value: "haiku", Label: "Haiku", Detail: "200k"}, {Value: "opus", Group: "Large"}}},
			`{"cursor":0,"label":"Model","options":[{"detail":"200k","label":"Haiku","value":"haiku"},` +
				`{"group":"Large","value":"opus"}],"search":true,"value":"haiku","variant":"quiet"}`},
		{ui.Switch{On: true, Variant: ui.SwitchSegment, Label: "Theme", Labels: [2]string{"Light", "Dark"},
			Disabled: ui.Because("locked")},
			`{"disabled":"locked","label":"Theme","labels":["Light","Dark"],"on":true,"variant":"segment"}`},
		{ui.Slider{Value: ui.SliderRange(200, 1200), Max: new(2000.0), Step: 100, Format: ui.SliderDuration,
			Ticks: ui.TickParts(4)}, `{"format":"duration","max":2000,"step":100,"ticks":4,"value":[200,1200]}`},
		{ui.Slider{Value: ui.SliderAt(0), Ticks: ui.NoTicks}, `{"ticks":false,"value":0}`},
		{ui.FormField{Label: ui.T("Port"), Error: ui.Says(ui.T("Must be 1–65535")), Required: true,
			LabelWidth: ui.Ch(14), Variant: ui.FieldGrid, Children: ui.Nodes(ui.Input{Field: ui.Field{Text: "0"}})},
			`{"error":"Must be 1–65535","label":"Port","labelWidth":"14ch","required":true,"variant":"grid"}`},
		{ui.FormField{Warning: ui.Flagged}, `{"warning":true}`},
		{ui.Otp{Text: "12", Length: 6, Group: new(0), Cursor: ui.NoCaret, Mask: ui.MaskWith("*"), State: ui.OtpError},
			`{"cursor":false,"group":0,"length":6,"mask":"*","state":"error","text":"12"}`},
		{ui.Otp{Cursor: ui.CaretAt(2), Mask: ui.MaskDots}, `{"cursor":2,"mask":true}`},
		{ui.Buttons{Variant: ui.ButtonsSplit, Size: ui.SizeSM, Label: "Deploy"},
			`{"label":"Deploy","size":"sm","variant":"split"}`},
		{ui.Calendar{Value: ui.Days("2026-10-19", ""), Range: true, WeekStart: ui.WeekSun,
			Marks: []ui.DayMark{{Date: "2026-10-20", Tone: ui.ToneError}}, Presets: []ui.Preset{{ID: "7d"}}},
			`{"marks":[{"date":"2026-10-20","tone":"error"}],"presets":[{"id":"7d"}],"range":true,` +
				`"value":["2026-10-19",null],"weekStart":"sun"}`},
		{ui.Date{Value: ui.Days("2026-10-19", "2026-10-30"), Open: true},
			`{"open":true,"value":["2026-10-19","2026-10-30"]}`},
		{ui.Calendar{Value: ui.Day("2026-10-30"), Min: "2026-10-01"}, `{"min":"2026-10-01","value":"2026-10-30"}`},
		{ui.Input{Field: ui.Field{Text: "s3cr3t", Variant: ui.InputField,
			Buttons: []ui.FieldButton{{Act: "copy", Icon: "copy", Title: "Copy token"}}},
			Secret: true, Keys: []string{"cmd", "f"}, Clear: true, Stepper: true, Unit: "bytes"},
			`{"buttons":[{"act":"copy","icon":"copy","title":"Copy token"}],"clear":true,"keys":["cmd","f"],` +
				`"secret":true,"stepper":true,"text":"s3cr3t","unit":"bytes","variant":"field"}`},
		{ui.Editor{Field: ui.Field{Buttons: []ui.FieldButton{{Act: "send", Text: "Send", Variant: ui.ButtonPrimary,
			Keys: []string{"enter"}}}}}, `{"buttons":[{"act":"send","keys":["enter"],"text":"Send","variant":"primary"}]}`},
	}
	for _, c := range cases {
		if got := props(t, c.e); got != c.want {
			t.Errorf("%T: got %s\nwant %s", c.e, got, c.want)
		}
	}
	if n := (ui.FormField{Children: ui.Nodes(ui.Switch{}, ui.Button{Text: "Go"})}).Node(); len(n.Children) != 2 {
		t.Errorf("field children %+v", n.Children)
	}
}

func TestNavigationKinds(t *testing.T) {
	cases := []struct {
		e    tern.Element
		want string
	}{
		{ui.Breadcrumb{Items: []ui.Crumb{{ID: "acme", Icon: "folder"}, {ID: "prod", Href: "https://x.dev"}},
			Max: 4, Open: true, Active: "acme", Variant: ui.BreadcrumbPath},
			`{"active":"acme","items":[{"icon":"folder","id":"acme"},{"href":"https://x.dev","id":"prod"}],` +
				`"max":4,"open":true,"variant":"path"}`},
		{ui.Pager{Page: 2, Total: new(312), Size: 25, Label: "rows", Loading: true},
			`{"label":"rows","loading":true,"page":2,"size":25,"total":312}`},
		{ui.Nav{Items: []ui.NavItem{{ID: "docs", Count: "3", Items: []ui.NavSubItem{{ID: "api", Detail: "v2"}}}},
			Active: "api", Variant: ui.NavLinks},
			`{"active":"api","items":[{"count":"3","id":"docs","items":[{"detail":"v2","id":"api"}]}],"variant":"links"}`},
		{ui.Menu{Text: "Sort", Caret: true, Trigger: ui.ButtonQuiet, Align: ui.AlignEnd, Items: []ui.MenuEntry{
			ui.MenuHeader("Order"),
			ui.MenuItem{ID: "asc", Radio: "dir", Checked: new(true), Keys: []string{"cmd", "1"}},
			ui.MenuSep, ui.MenuAct("reset"),
			ui.MenuItem{ID: "more", Danger: true, Items: []ui.MenuEntry{ui.MenuAct("x")}},
		}}, `{"align":"end","caret":true,"items":[{"header":"Order"},{"checked":true,"id":"asc","keys":["cmd","1"],` +
			`"radio":"dir"},"-","reset",{"danger":true,"id":"more","items":["x"]}],"text":"Sort","trigger":"quiet"}`},
		{ui.Menubar{Menus: []ui.MenubarMenu{{ID: "file", Items: []ui.MenuEntry{ui.MenuAct("new")}}},
			Open: "file", Variant: ui.MenubarPill}, `{"menus":[{"id":"file","items":["new"]}],"open":"file","variant":"pill"}`},
		{ui.Text{Text: "deploy.log", Common: ui.Common{Actions: &ui.Actions{Menu: []ui.MenuEntry{
			ui.MenuItem{ID: "open", Icon: "file", Keys: []string{"enter"}}, ui.MenuSep, ui.MenuAct("copy-path")}}}},
			`{"actions":{"menu":[{"icon":"file","id":"open","keys":["enter"]},"-","copy-path"]},"text":"deploy.log"}`},
	}
	for _, c := range cases {
		if got := props(t, c.e); got != c.want {
			t.Errorf("%T: got %s\nwant %s", c.e, got, c.want)
		}
	}
}

func TestDataAndLayoutKinds(t *testing.T) {
	cases := []struct {
		e    tern.Element
		want string
	}{
		{ui.Avatar{Text: "Cara Alvarez", Seed: "cara@acme.dev", Size: ui.SizeXS, Status: ui.ToneSuccess,
			Variant: ui.AvatarSquare, Common: ui.Common{Title: "Cara"}},
			`{"seed":"cara@acme.dev","size":"xs","status":"success","text":"Cara Alvarez","title":"Cara","variant":"square"}`},
		{ui.Avatars{Items: []ui.Face{{Text: "Ann"}, {Text: "Bo", Icon: "provider:anthropic"}}, Max: 4, Label: "2 reviewers"},
			`{"items":[{"text":"Ann"},{"icon":"provider:anthropic","text":"Bo"}],"label":"2 reviewers","max":4}`},
		{ui.Picker{Items: []ui.PickerItem{{ID: "a", Mark: &ui.Face{Text: "Ann", Seed: "ann@x.dev"}}}},
			`{"items":[{"id":"a","mark":{"seed":"ann@x.dev","text":"Ann"}}]}`},
		{ui.Attachment{CommonWithoutActions: ui.CommonWithoutActions{Href: "https://x.dev/a.pdf"}, Name: "a.pdf",
			Size: new(int64(2400000)), State: ui.AttachmentUploading, Progress: new(0.62), Removable: true,
			Actions: []ui.IconAction{{Act: "open"}}},
			`{"actions":[{"act":"open"}],"href":"https://x.dev/a.pdf","name":"a.pdf","progress":0.62,` +
				`"removable":true,"size":2400000,"state":"uploading"}`},
		{ui.Carousel{Index: 2, Loop: true, Variant: ui.CarouselStrip, H: ui.Lines(10)},
			`{"h":"10lines","index":2,"loop":true,"variant":"strip"}`},
		{ui.Carousel{H: ui.Px(240)}, `{"h":240}`},
		{ui.Empty{Title: ui.T("No deploys"), Icon: new(ui.IconName("")), Variant: ui.EmptyInline, Frame: true},
			`{"frame":true,"icon":"","title":"No deploys","variant":"inline"}`},
		{ui.Split{Sizes: []float64{2, 1}, Min: []ui.Extent{ui.Ch(20), {}}, H: ui.Lines(12), Dir: ui.SplitCol,
			Active: new(0)}, `{"active":0,"dir":"col","h":"12lines","min":["20ch",null],"sizes":[2,1]}`},
		{ui.Scroll{Stick: ui.StickEnd, Pinned: new(false), Noun: ui.Noun{One: "entry", Many: "entries"}, Gap: ui.GapSM},
			`{"gap":"sm","noun":["entry","entries"],"pinned":false,"stick":"end"}`},
		{ui.Scroll{Noun: ui.Noun{One: "line"}, Bar: ui.BarAlways}, `{"bar":"always","noun":"line"}`},
		{ui.Skeleton{Shape: ui.ShapeRows, Rows: 8, Variant: ui.SkeletonWire}, `{"rows":8,"shape":"rows","variant":"wire"}`},
		{ui.Rule{Variant: ui.RuleNote, Label: ui.T("Checkpoint"), Icon: "history", Time: "14:02"},
			`{"icon":"history","label":"Checkpoint","time":"14:02","variant":"note"}`},
		{ui.Text{Spans: []ui.Span{{T: "Go 1.26", Cite: []ui.Cite{{Href: "https://go.dev", N: 1}}}},
			Cite: ui.CiteNote, Sources: new(false)},
			`{"cite":"note","sources":false,"spans":[{"cite":[{"href":"https://go.dev","n":1}],"t":"Go 1.26"}]}`},
		{ui.Overlay{Side: ui.SideRight, Variant: ui.OverlayInset, SizeFrac: 0.4},
			`{"side":"right","size":0.4,"variant":"inset"}`},
		{ui.Overlay{Side: ui.SideBottom, Size: ui.SizeLG}, `{"side":"bottom","size":"lg"}`},
		{ui.Table{Cols: []ui.Column{{ID: "age", Sortable: true}}, Sort: &ui.TableSort{Col: "age", Dir: ui.SortDesc},
			Multi: true, Picked: []string{}, Variant: ui.TableBar, Bar: []ui.BarAction{{Act: "restart", Text: "Restart"}},
			Hint: []ui.Span{{T: "r restarts", S: "muted"}}},
			`{"bar":[{"act":"restart","text":"Restart"}],"cols":[{"id":"age","sortable":true}],` +
				`"hint":[{"s":"muted","t":"r restarts"}],"multi":true,"picked":[],"sort":{"col":"age","dir":"desc"},"variant":"bar"}`},
		{ui.Chart{Kind: ui.ChartLine, Labels: []string{"a", "", "c"}, Unit: "ms", Last: ui.HideLast,
			Lines: []ui.ChartSeries{{Label: "p50", Values: []ui.Point{{Value: 1}, {Gap: true}, {Value: 3}}}}},
			`{"kind":"line","labels":["a","","c"],"last":false,"series":[{"label":"p50","values":[{"value":1},null,{"value":3}]}],"unit":"ms"}`},
		{ui.Chart{Kind: ui.ChartDonut, Series: []ui.Point{{Value: 3, Label: "ok", Token: "success"}},
			Total: ui.T("3"), Caption: ui.T("runs")},
			`{"caption":"runs","kind":"donut","series":[{"label":"ok","token":"success","value":3}],"total":"3"}`},
		{ui.Chart{Kind: ui.ChartArea, Last: ui.LastSays(ui.T("now"))}, `{"kind":"area","last":"now"}`},
	}
	for _, c := range cases {
		if got := props(t, c.e); got != c.want {
			t.Errorf("%T: got %s\nwant %s", c.e, got, c.want)
		}
	}
}

func TestWorkKinds(t *testing.T) {
	cases := []struct {
		e    tern.Element
		want string
	}{
		{ui.Message{Side: ui.MessageEnd, Author: "Sam", Reveal: ui.RevealAlways, Pending: true,
			Reactions: []ui.Reaction{{Text: "♥", Count: 2, On: true}},
			Actions:   []ui.IconAction{{Act: "retry", Title: "Regenerate"}}},
			`{"actions":[{"act":"retry","title":"Regenerate"}],"author":"Sam","pending":true,` +
				`"reactions":[{"count":2,"on":true,"text":"♥"}],"reveal":"always","side":"end"}`},
		{ui.Trace{Title: ui.T("Reasoning"), Status: ui.Running, Variant: ui.TraceCard, Open: new(false),
			Lines: []ui.TraceLine{{ID: "s1", Title: ui.T("Search"), Status: ui.Done, Took: new(1200.0),
				Sources: []ui.Source{{Href: "https://go.dev"}}}}},
			`{"lines":[{"id":"s1","sources":[{"href":"https://go.dev"}],"status":"done","title":"Search","took":1200}],` +
				`"open":false,"status":"running","title":"Reasoning","variant":"card"}`},
		{ui.Stacktrace{Error: &ui.StackError{Type: "TypeError", Message: "x is undefined"}, Selected: new(0),
			Frames: []ui.StackFrame{{Fn: "main", File: "a.ts", Line: 3, App: new(false),
				Code: &ui.FrameCode{Lines: []string{"x()"}}}}},
			`{"error":{"message":"x is undefined","type":"TypeError"},"frames":[{"app":false,"code":{"lines":["x()"]},` +
				`"file":"a.ts","fn":"main","line":3}],"selected":0}`},
		{ui.Suggestions{Variant: ui.SuggestRows, Items: []ui.Suggestion{{Text: "Fix the test", Keys: []string{}}}},
			`{"items":[{"keys":[],"text":"Fix the test"}],"variant":"rows"}`},
		{ui.Questions{Items: []ui.Question{{ID: "db", Prompt: "Which DB?", Choices: []ui.Choice{{Value: "pg", Label: "Postgres"}},
			Other: &ui.Other{}}}, Answers: map[string]any{"db": "pg"}, Hint: new("")},
			`{"answers":{"db":"pg"},"hint":"","items":[{"choices":[{"label":"Postgres","value":"pg"}],"id":"db",` +
				`"other":{},"prompt":"Which DB?"}]}`},
		{ui.Tool{Status: ui.Asking, Confirm: ui.ConfirmScopes, Approved: ui.ApprovedSession,
			Ask: &ui.ToolAsk{Always: ui.NoAlways, Scopes: []ui.AskScope{{ID: "once"}}, Scope: "once"}},
			`{"approved":"session","ask":{"always":false,"scope":"once","scopes":[{"id":"once"}]},"confirm":"scopes","status":"ask"}`},
	}
	for _, c := range cases {
		if got := props(t, c.e); got != c.want {
			t.Errorf("%T: got %s\nwant %s", c.e, got, c.want)
		}
	}
	acted := false
	n := ui.Message{CommonWithoutActions: ui.CommonWithoutActions{
		OnAction: map[string]func(*tern.Action){"retry": func(*tern.Action) { acted = true }}}}.Node()
	if n.OnAction["retry"] == nil {
		t.Fatal("message lost its handler")
	}
	n.OnAction["retry"](&tern.Action{})
	if !acted {
		t.Error("message handler not called")
	}
}

func TestNewSlotsHoistAsKeyedChildren(t *testing.T) {
	view := tern.Main(
		ui.Rule{Common: ui.Common{Key: "r"}, Variant: ui.RuleNote, Label: ui.T("Checkpoint"),
			End: ui.Button{Text: "Restore"}},
		ui.Message{CommonWithoutActions: ui.CommonWithoutActions{Key: "m"}, Author: "Sam",
			Avatar: ui.Avatar{Text: "Sam"}, Children: ui.Nodes(ui.Md{Text: "hi"})},
	)
	ops, err := tern.NewReconciler("sf").Diff(view)
	if err != nil {
		t.Fatal(err)
	}
	b, err := tern.Marshal(ops)
	if err != nil {
		t.Fatal(err)
	}
	got := string(b)
	for _, want := range []string{
		`"end":"main.r.end"`, `{"id":"main.r.end","k":"button","p":{"key":"end","text":"Restore"}}`,
		`"avatar":"main.m.avatar"`, `{"id":"main.m.avatar","k":"avatar","p":{"key":"avatar","text":"Sam"}}`,
		`{"id":"main.m.0","k":"md"`,
	} {
		if !strings.Contains(got, want) {
			t.Errorf("missing %s in %s", want, got)
		}
	}
}
