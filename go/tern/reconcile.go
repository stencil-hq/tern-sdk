package tern

import (
	"bytes"
	"encoding/json"
	"fmt"
	"reflect"
	"sort"
	"strconv"
)

// regionNames are the regions of a surface, in the order they are added.
var regionNames = [3]string{"main", "dock", "layer"}

// tree is one node of a view with its id assigned and props normalized.
type tree struct {
	id       string
	kind     string
	props    map[string]any
	children []*tree
}

// viewTree is a compiled view: each region's tree and the handlers by id.
type viewTree struct {
	regions  [3]*tree
	handlers map[string]*bound
}

// DuplicateKeyError rejects a view in which two siblings have one id.
type DuplicateKeyError struct {
	// Parent is the id of the siblings' parent.
	Parent string
	// Key is the key (or index) they share.
	Key string
}

func (e *DuplicateKeyError) Error() string {
	return fmt.Sprintf("tern: %s: duplicate child key %q", e.Parent, e.Key)
}

// compile assigns ids to v, normalizes props and binds handlers.
func compile(v View) (*viewTree, error) {
	vt := &viewTree{handlers: map[string]*bound{}}
	for i, el := range [3]Element{v.Main, v.Dock, v.Layer} {
		if el == nil {
			continue
		}
		t, err := vt.build(regionNames[i], el.Node())
		if err != nil {
			return nil, err
		}
		vt.regions[i] = t
	}
	return vt, nil
}

// build compiles node n under id.
func (vt *viewTree) build(id string, n Node) (*tree, error) {
	props := normalizeProps(n.Props)
	if !n.Handlers.empty() {
		props = n.Handlers.bind(props)
		b := &bound{Handlers: n.Handlers}
		if actions, ok := props["actions"].(map[string]any); ok {
			b.click, _ = actions["click"].(string)
			b.dblclick, _ = actions["dblclick"].(string)
		}
		vt.handlers[id] = b
	}
	t := &tree{id: id, kind: n.Kind, props: props}
	if len(n.Children) > 0 {
		t.children = make([]*tree, 0, len(n.Children))
		seen := make(map[string]struct{}, len(n.Children))
		for i, el := range n.Children {
			if el == nil {
				continue
			}
			child := el.Node()
			key := keyOf(child.Props, i)
			cid := id + "." + key
			if _, dup := seen[cid]; dup {
				return nil, &DuplicateKeyError{Parent: id, Key: key}
			}
			seen[cid] = struct{}{}
			ct, err := vt.build(cid, child)
			if err != nil {
				return nil, err
			}
			t.children = append(t.children, ct)
		}
	}
	return t, nil
}

// keyOf is a child's key: its key prop (a string, or a number in decimal),
// else its index.
func keyOf(props map[string]any, index int) string {
	switch k := props["key"].(type) {
	case string:
		return k
	case json.Number:
		return k.String()
	case int:
		return strconv.Itoa(k)
	case int64:
		return strconv.FormatInt(k, 10)
	case float64:
		return strconv.FormatFloat(k, 'f', -1, 64)
	}
	return strconv.Itoa(index)
}

// normalizeProps returns props with every value in encoding/json's generic
// form (map[string]any, []any, json.Number, string, bool, nil), so that
// deep equality is JSON equality.
func normalizeProps(props map[string]any) map[string]any {
	if len(props) == 0 {
		return nil
	}
	out := make(map[string]any, len(props))
	for k, v := range props {
		out[k] = normalize(v)
	}
	return out
}

// normalize returns v in encoding/json's generic form.
func normalize(v any) any {
	switch x := v.(type) {
	case nil, bool, string, json.Number:
		return v
	case map[string]any:
		out := make(map[string]any, len(x))
		for k, e := range x {
			out[k] = normalize(e)
		}
		return out
	case []any:
		out := make([]any, len(x))
		for i, e := range x {
			out[i] = normalize(e)
		}
		return out
	}
	b, err := Marshal(v)
	if err != nil {
		return nil
	}
	return decodeGeneric(b)
}

// decodeGeneric decodes JSON into generic values with numbers kept as text.
func decodeGeneric(b []byte) any {
	dec := json.NewDecoder(bytes.NewReader(b))
	dec.UseNumber()
	var out any
	if dec.Decode(&out) != nil {
		return nil
	}
	return out
}

// wire is t as an add op's subtree with every id.
func (t *tree) wire() WireNode {
	w := WireNode{ID: t.id, K: t.kind}
	if len(t.props) > 0 {
		w.P = t.props
	}
	if len(t.children) > 0 {
		w.C = make([]WireNode, len(t.children))
		for i, c := range t.children {
			w.C[i] = c.wire()
		}
	}
	return w
}

// diffViews is the ops turning old into next on surface sf.
func diffViews(old, next *viewTree, sf string) []Op {
	var ops []Op
	for i := range regionNames {
		var o, n *tree
		if old != nil {
			o = old.regions[i]
		}
		if next != nil {
			n = next.regions[i]
		}
		switch {
		case o == nil && n == nil:
		case n == nil:
			ops = append(ops, OpDel(o.id))
		case o == nil:
			ops = append(ops, OpAdd(sf, "", n.wire()))
		case o.kind != n.kind:
			ops = append(ops, OpDel(o.id), OpAdd(sf, "", n.wire()))
		default:
			ops = diffTree(o, n, ops)
		}
	}
	return ops
}

// diffTree appends the ops turning old into next, which share id and kind.
func diffTree(old, next *tree, ops []Op) []Op {
	text := IsTextKind(next.kind)
	set := map[string]any{}
	for k, v := range next.props {
		if text && k == "text" {
			continue
		}
		if ov, ok := old.props[k]; !ok || !reflect.DeepEqual(ov, v) {
			set[k] = v
		}
	}
	for k := range old.props {
		if text && k == "text" {
			continue
		}
		if _, ok := next.props[k]; !ok {
			set[k] = nil
		}
	}
	if len(set) > 0 {
		ops = append(ops, OpSet(next.id, set))
	}
	if text {
		before, hadBefore := old.props["text"]
		after, hasAfter := next.props["text"]
		if hadBefore != hasAfter || !reflect.DeepEqual(before, after) {
			a, aok := before.(string)
			b, bok := after.(string)
			switch {
			case aok && bok && a != "" && len(b) >= len(a) && b[:len(a)] == a:
				ops = append(ops, OpText(next.id, "append", b[len(a):]))
			case bok:
				ops = append(ops, OpText(next.id, "replace", b))
			default:
				ops = append(ops, OpSet(next.id, map[string]any{"text": nil}))
			}
		}
	}
	return diffChildren(old, next, ops)
}

// diffChildren appends the ops turning old's children into next's:
// deletions first, then from the last child back each new, re-kinded or
// out-of-order child is placed before its next sibling, kept ones diffed.
func diffChildren(old, next *tree, ops []Op) []Op {
	type at struct {
		index int
		node  *tree
	}
	before := make(map[string]at, len(old.children))
	for i, c := range old.children {
		before[c.id] = at{i, c}
	}
	kept := make(map[string]struct{}, len(next.children))
	for _, c := range next.children {
		kept[c.id] = struct{}{}
	}
	for _, c := range old.children {
		if _, ok := kept[c.id]; !ok {
			ops = append(ops, OpDel(c.id))
		}
	}
	var sameAt, sameOld []int
	for i, c := range next.children {
		if o, ok := before[c.id]; ok && o.node.kind == c.kind {
			sameAt = append(sameAt, i)
			sameOld = append(sameOld, o.index)
		}
	}
	stay := make(map[int]struct{}, len(sameAt))
	for _, k := range rising(sameOld) {
		stay[sameAt[k]] = struct{}{}
	}
	nextID := ""
	for i := len(next.children) - 1; i >= 0; i-- {
		c := next.children[i]
		o, ok := before[c.id]
		switch {
		case !ok:
			ops = append(ops, OpAdd(next.id, nextID, c.wire()))
		case o.node.kind != c.kind:
			ops = append(ops, OpDel(c.id), OpAdd(next.id, nextID, c.wire()))
		default:
			if _, ok := stay[i]; !ok {
				ops = append(ops, OpMove(c.id, next.id, nextID))
			}
			ops = diffTree(o.node, c, ops)
		}
		nextID = c.id
	}
	return ops
}

// rising is the indices into seq of one longest strictly rising subsequence,
// the same one Tern's reconciler picks.
func rising(seq []int) []int {
	var tails []int
	prev := make([]int, len(seq))
	for i, v := range seq {
		at := sort.Search(len(tails), func(j int) bool { return seq[tails[j]] >= v })
		prev[i] = -1
		if at > 0 {
			prev[i] = tails[at-1]
		}
		if at == len(tails) {
			tails = append(tails, i)
		} else {
			tails[at] = i
		}
	}
	if len(tails) == 0 {
		return nil
	}
	out := make([]int, len(tails))
	for k, cur := len(tails)-1, tails[len(tails)-1]; cur >= 0; k, cur = k-1, prev[cur] {
		out[k] = cur
	}
	return out
}

// Reconciler diffs views into frame ops with derived ids, keeping the last
// view it diffed against, exactly as Tern's plugin worker does.
type Reconciler struct {
	surface string
	last    *viewTree
}

// NewReconciler returns a reconciler for surface id sf with nothing sent.
func NewReconciler(sf string) *Reconciler { return &Reconciler{surface: sf} }

// Diff returns the ops turning the last view into v and makes v the last.
// A view with two siblings of one id is rejected, and nothing changes.
func (r *Reconciler) Diff(v View) ([]Op, error) {
	next, err := compile(v)
	if err != nil {
		return nil, err
	}
	ops := diffViews(r.last, next, r.surface)
	r.last = next
	return ops, nil
}
