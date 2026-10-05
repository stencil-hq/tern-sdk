// Package el builds el nodes: plain HTML elements from Tern's fixed tag
// list, styled by the program's stylesheets. There is one type per tag;
// each takes a class, allowlisted attrs, text drawn before its children,
// and the props and handlers of ui.Common.
//
//	el.Form{Class: "ask", Children: ui.Nodes(
//		el.P{Text: "Which size?"},
//		el.Label{Children: ui.Nodes(el.Input{Type: el.Radio, Name: "size", Value: "s"}, el.Span{Text: "Small"})},
//		el.Button{Text: "Create", Common: ui.Common{Actions: &ui.Actions{Click: "submit"}}},
//	)}
package el

import (
	"github.com/stencil-hq/tern-sdk/go/tern"
	"github.com/stencil-hq/tern-sdk/go/tern/ui"
)

// Props are the props of every tag but input and hr.
type Props struct {
	ui.Common
	// Class is space-separated class names for stylesheets.
	Class string `json:"class,omitzero"`
	// Attrs are allowlisted attributes: data-*, aria-*, role, colspan,
	// rowspan; values are strings, numbers or booleans.
	Attrs map[string]any `json:"attrs,omitzero"`
	// Text is plain text drawn before the children.
	Text string `json:"text,omitzero"`
	// Children are drawn in order after Text.
	Children []tern.Element `json:"-"`
}

// tagged is Props with its tag, as the node's props.
type tagged struct {
	Tag string `json:"tag"`
	Props
}

// build makes the el node of tag.
func (p Props) build(tag string) tern.Node {
	return ui.Build(tern.KindEl, tagged{tag, p}, p.Children)
}

// Input is a checkbox or radio control (tag input); it has no text or children.
type Input struct {
	ui.Common
	// Class is space-separated class names for stylesheets.
	Class string `json:"class,omitzero"`
	// Attrs are allowlisted attributes (data-*, aria-*).
	Attrs map[string]any `json:"attrs,omitzero"`
	// Type is Checkbox or Radio.
	Type InputType `json:"type,omitzero"`
	// Name is the control's name in its form's values and its radio group.
	Name string `json:"name,omitzero"`
	// Value is what it reports (default "on").
	Value string `json:"value,omitzero"`
	// Checked is the checked state the program sets.
	Checked bool `json:"checked,omitzero"`
	// Disabled draws it dimmed; it never flips and is left out of values.
	Disabled bool `json:"disabled,omitzero"`
}

// InputType is a control's type.
type InputType string

// Control types.
const (
	Checkbox InputType = "checkbox"
	Radio    InputType = "radio"
)

// Node builds the input.
func (i Input) Node() tern.Node {
	return ui.Build(tern.KindEl, struct {
		Tag string `json:"tag"`
		Input
	}{"input", i}, nil)
}

// Hr is a horizontal rule (tag hr); it has no text or children.
type Hr struct {
	ui.Common
	// Class is space-separated class names for stylesheets.
	Class string `json:"class,omitzero"`
	// Attrs are allowlisted attributes.
	Attrs map[string]any `json:"attrs,omitzero"`
}

// Node builds the hr.
func (h Hr) Node() tern.Node {
	return ui.Build(tern.KindEl, struct {
		Tag string `json:"tag"`
		Hr
	}{"hr", h}, nil)
}

// Div is a <div> element.
type Div Props

// Node builds the div.
func (e Div) Node() tern.Node { return Props(e).build("div") }

// Span is a <span> element.
type Span Props

// Node builds the span.
func (e Span) Node() tern.Node { return Props(e).build("span") }

// P is a <p> element.
type P Props

// Node builds the p.
func (e P) Node() tern.Node { return Props(e).build("p") }

// Section is a <section> element.
type Section Props

// Node builds the section.
func (e Section) Node() tern.Node { return Props(e).build("section") }

// Header is a <header> element.
type Header Props

// Node builds the header.
func (e Header) Node() tern.Node { return Props(e).build("header") }

// Footer is a <footer> element.
type Footer Props

// Node builds the footer.
func (e Footer) Node() tern.Node { return Props(e).build("footer") }

// Nav is a <nav> element.
type Nav Props

// Node builds the nav.
func (e Nav) Node() tern.Node { return Props(e).build("nav") }

// Aside is a <aside> element.
type Aside Props

// Node builds the aside.
func (e Aside) Node() tern.Node { return Props(e).build("aside") }

// Main is a <main> element.
type Main Props

// Node builds the main.
func (e Main) Node() tern.Node { return Props(e).build("main") }

// Article is a <article> element.
type Article Props

// Node builds the article.
func (e Article) Node() tern.Node { return Props(e).build("article") }

// Figure is a <figure> element.
type Figure Props

// Node builds the figure.
func (e Figure) Node() tern.Node { return Props(e).build("figure") }

// Blockquote is a <blockquote> element.
type Blockquote Props

// Node builds the blockquote.
func (e Blockquote) Node() tern.Node { return Props(e).build("blockquote") }

// Ul is a <ul> element.
type Ul Props

// Node builds the ul.
func (e Ul) Node() tern.Node { return Props(e).build("ul") }

// Ol is a <ol> element.
type Ol Props

// Node builds the ol.
func (e Ol) Node() tern.Node { return Props(e).build("ol") }

// Li is a <li> element.
type Li Props

// Node builds the li.
func (e Li) Node() tern.Node { return Props(e).build("li") }

// Dl is a <dl> element.
type Dl Props

// Node builds the dl.
func (e Dl) Node() tern.Node { return Props(e).build("dl") }

// Dt is a <dt> element.
type Dt Props

// Node builds the dt.
func (e Dt) Node() tern.Node { return Props(e).build("dt") }

// Dd is a <dd> element.
type Dd Props

// Node builds the dd.
func (e Dd) Node() tern.Node { return Props(e).build("dd") }

// H1 is a <h1> element.
type H1 Props

// Node builds the h1.
func (e H1) Node() tern.Node { return Props(e).build("h1") }

// H2 is a <h2> element.
type H2 Props

// Node builds the h2.
func (e H2) Node() tern.Node { return Props(e).build("h2") }

// H3 is a <h3> element.
type H3 Props

// Node builds the h3.
func (e H3) Node() tern.Node { return Props(e).build("h3") }

// H4 is a <h4> element.
type H4 Props

// Node builds the h4.
func (e H4) Node() tern.Node { return Props(e).build("h4") }

// Pre is a <pre> element.
type Pre Props

// Node builds the pre.
func (e Pre) Node() tern.Node { return Props(e).build("pre") }

// Code is a <code> element.
type Code Props

// Node builds the code.
func (e Code) Node() tern.Node { return Props(e).build("code") }

// Kbd is a <kbd> element.
type Kbd Props

// Node builds the kbd.
func (e Kbd) Node() tern.Node { return Props(e).build("kbd") }

// Strong is a <strong> element.
type Strong Props

// Node builds the strong.
func (e Strong) Node() tern.Node { return Props(e).build("strong") }

// B is a <b> element.
type B Props

// Node builds the b.
func (e B) Node() tern.Node { return Props(e).build("b") }

// Em is a <em> element.
type Em Props

// Node builds the em.
func (e Em) Node() tern.Node { return Props(e).build("em") }

// I is a <i> element.
type I Props

// Node builds the i.
func (e I) Node() tern.Node { return Props(e).build("i") }

// Del is a <del> element.
type Del Props

// Node builds the del.
func (e Del) Node() tern.Node { return Props(e).build("del") }

// Mark is a <mark> element.
type Mark Props

// Node builds the mark.
func (e Mark) Node() tern.Node { return Props(e).build("mark") }

// Table is a <table> element.
type Table Props

// Node builds the table.
func (e Table) Node() tern.Node { return Props(e).build("table") }

// Thead is a <thead> element.
type Thead Props

// Node builds the thead.
func (e Thead) Node() tern.Node { return Props(e).build("thead") }

// Tbody is a <tbody> element.
type Tbody Props

// Node builds the tbody.
func (e Tbody) Node() tern.Node { return Props(e).build("tbody") }

// Tr is a <tr> element.
type Tr Props

// Node builds the tr.
func (e Tr) Node() tern.Node { return Props(e).build("tr") }

// Th is a <th> element.
type Th Props

// Node builds the th.
func (e Th) Node() tern.Node { return Props(e).build("th") }

// Td is a <td> element.
type Td Props

// Node builds the td.
func (e Td) Node() tern.Node { return Props(e).build("td") }

// Label is a <label> element.
type Label Props

// Node builds the label.
func (e Label) Node() tern.Node { return Props(e).build("label") }

// Button is a <button> element.
type Button Props

// Node builds the button.
func (e Button) Node() tern.Node { return Props(e).build("button") }

// Form is a <form> element.
type Form Props

// Node builds the form.
func (e Form) Node() tern.Node { return Props(e).build("form") }
