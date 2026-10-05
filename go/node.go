package tern

import (
	"slices"
)

// Element is anything that builds a node: a Node, a Nodes list, and the
// typed builders of packages ui and el.
type Element interface {
	Node() Node
}

// Node is one node of a view: a kind, props, children and handlers. It has
// no id until it is reconciled. Node is also the untyped escape hatch for
// any kind and any prop:
//
//	tern.Node{Kind: "badge", Props: map[string]any{"text": "new", "tone": "accent"}}
type Node struct {
	// Kind is the node's kind ("card", "text", "el", …).
	Kind string
	// Props are the node's props; values are anything encoding/json writes.
	Props map[string]any
	// Children are the node's children, in order.
	Children []Element
	// Handlers run on the node's events.
	Handlers
}

// Node returns n itself.
func (n Node) Node() Node { return n }

// Nodes is a list of elements; as an element (and as a view region) it is a
// col holding them.
type Nodes []Element

// Node is a col holding the list.
func (ns Nodes) Node() Node { return Node{Kind: KindCol, Children: ns} }

// View is what a surface shows: up to three regions. A nil region is absent.
// Tern draws a region root's children straight into the region and ignores
// the root's own kind and props (but role), so give each region a Nodes
// list (or a col).
type View struct {
	Main, Dock, Layer Element
}

// Main is the view whose main region holds els (in a col) and nothing
// else: the shorthand for rendering a node or a list of nodes.
func Main(els ...Element) View { return View{Main: Nodes(els)} }

// Handlers are funcs attached to a node, run when its events arrive. They
// run on the goroutine that reads the session's input (Session.Next or
// Session.Run). A handled event is consumed; every other event reaches the
// program's loop.
type Handlers struct {
	// OnClick runs on the node's click action; it sets actions.click to
	// "click" unless the node names its click action itself.
	OnClick func(*Action)
	// OnDblClick runs on the node's dblclick action; it sets
	// actions.dblclick to "dblclick" unless the node names it itself.
	OnDblClick func(*Action)
	// OnMenu adds each name to actions.menu (sorted, after the names the node
	// lists itself) and runs on that action.
	OnMenu map[string]func(*Action)
	// OnAction runs on action events whose act (or act=value) is its key.
	OnAction map[string]func(*Action)
	// OnToggle runs when the user folds or unfolds the node.
	OnToggle func(*Toggle)
	// OnSelect runs on select events (for a list, its items' selects).
	OnSelect func(*Select)
	// OnActivate runs on activate events (for a list, its items').
	OnActivate func(*Activate)
	// OnChange runs when a control or prefs row changes.
	OnChange func(*Change)
	// OnFocus runs when a click asks for the keys in the node.
	OnFocus func(*Focus)
	// OnEdit runs on native editing edits to the field.
	OnEdit func(*Edit)
	// OnUndo runs on undo in the field.
	OnUndo func(*Undo)
	// OnSend runs when Tern submits text into the field.
	OnSend func(*Send)
}

// empty reports whether no handler is set.
func (h *Handlers) empty() bool {
	return h.OnClick == nil && h.OnDblClick == nil && len(h.OnMenu) == 0 && len(h.OnAction) == 0 &&
		h.OnToggle == nil && h.OnSelect == nil && h.OnActivate == nil && h.OnChange == nil &&
		h.OnFocus == nil && h.OnEdit == nil && h.OnUndo == nil && h.OnSend == nil
}

// bind returns props (normalized JSON values) with the actions the
// handlers need added.
func (h *Handlers) bind(props map[string]any) map[string]any {
	if h.OnClick == nil && h.OnDblClick == nil && len(h.OnMenu) == 0 {
		return props
	}
	actions := map[string]any{}
	if old, ok := props["actions"].(map[string]any); ok {
		for k, v := range old {
			actions[k] = v
		}
	}
	if h.OnClick != nil {
		if _, ok := actions["click"].(string); !ok {
			actions["click"] = "click"
		}
	}
	if h.OnDblClick != nil {
		if _, ok := actions["dblclick"].(string); !ok {
			actions["dblclick"] = "dblclick"
		}
	}
	if len(h.OnMenu) > 0 {
		menu, _ := actions["menu"].([]any)
		menu = slices.Clone(menu)
		names := make([]string, 0, len(h.OnMenu))
		for name := range h.OnMenu {
			names = append(names, name)
		}
		slices.Sort(names)
		for _, name := range names {
			if !slices.Contains(menu, any(name)) {
				menu = append(menu, name)
			}
		}
		actions["menu"] = menu
	}
	out := make(map[string]any, len(props)+1)
	for k, v := range props {
		out[k] = v
	}
	out["actions"] = actions
	return out
}

// bound is a node's handlers with the action names they answer to.
type bound struct {
	Handlers
	click, dblclick string
}

// route runs the handler for ev, reporting whether one ran.
func (b *bound) route(ev Event) bool {
	switch e := ev.(type) {
	case *Action:
		name := e.Act
		if e.Value != "" {
			name += "=" + e.Value
		}
		switch {
		case b.OnClick != nil && b.click != "" && (e.Act == b.click || name == b.click):
			b.OnClick(e)
		case b.OnDblClick != nil && b.dblclick != "" && (e.Act == b.dblclick || name == b.dblclick):
			b.OnDblClick(e)
		case b.OnMenu[name] != nil:
			b.OnMenu[name](e)
		case b.OnMenu[e.Act] != nil:
			b.OnMenu[e.Act](e)
		case b.OnAction[name] != nil:
			b.OnAction[name](e)
		case b.OnAction[e.Act] != nil:
			b.OnAction[e.Act](e)
		default:
			return false
		}
	case *Toggle:
		return call(b.OnToggle, e)
	case *Select:
		return call(b.OnSelect, e)
	case *Activate:
		return call(b.OnActivate, e)
	case *Change:
		return call(b.OnChange, e)
	case *Focus:
		return call(b.OnFocus, e)
	case *Edit:
		return call(b.OnEdit, e)
	case *Undo:
		return call(b.OnUndo, e)
	case *Send:
		return call(b.OnSend, e)
	default:
		return false
	}
	return true
}

// call runs f on e when f is set.
func call[E any](f func(E), e E) bool {
	if f == nil {
		return false
	}
	f(e)
	return true
}
