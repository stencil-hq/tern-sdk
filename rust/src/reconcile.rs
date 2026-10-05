//! Layer 4: reconcile. A view diffed into frame ops with derived ids.
//! Tern's plugin worker uses this reconciler directly.
//!
//! A region root's id is its region's name; a child's is `<parent>.<key>`,
//! where `key` is its `key` prop (a string, or an integer in decimal), else
//! its index among its siblings. Two siblings with one id reject the whole
//! view. The same id and kind become one `set` of the changed props (removed
//! ones `null`) and, for text kinds, a `text` op (`append` when the new text
//! extends a non-empty old one); a kind change is `del` + `add`; children
//! are deleted, then placed from the last back (added, re-added or moved
//! before their next sibling), each kept child diffed right after it is
//! placed.

use std::collections::{HashMap, HashSet};

use serde_json::{Map, Value};

use crate::{
	Error,
	ui::{
		Node, View,
		node::{Handler, Trigger},
	},
	wire::{Event, Op, TextMode, WireNode, kind},
};

/// The regions of a surface, in the order they are added.
pub const REGIONS: [&str; 3] = ["main", "dock", "layer"];

/// One node of a view with its id assigned.
#[derive(Clone, Debug, PartialEq)]
pub struct Tree {
	/// `parent.(key or index)`, or the region name at the top.
	id:       String,
	/// The kind.
	kind:     String,
	/// Props as the view gave them (with handler actions merged in).
	props:    Map<String, Value>,
	/// Children in order.
	children: Vec<Self>,
}

impl Tree {
	/// The node's id.
	pub fn id(&self) -> &str {
		&self.id
	}

	/// The node's kind.
	pub fn kind(&self) -> &str {
		&self.kind
	}

	/// The node's props.
	pub const fn props(&self) -> &Map<String, Value> {
		&self.props
	}

	/// The node's children.
	pub fn children(&self) -> &[Self] {
		&self.children
	}

	/// The node as an `add` op carries it, ids at every level.
	pub fn wire(&self) -> WireNode {
		WireNode {
			id: self.id.clone(),
			k:  self.kind.clone(),
			p:  self.props.clone(),
			c:  self.children.iter().map(Self::wire).collect(),
		}
	}

	/// Builds the tree of `node` under `id`, collecting its handlers.
	fn build<M>(id: String, node: Node<M>, routes: &mut Routes<M>) -> Result<Self, Error> {
		let Node { kind, mut props, children: nodes, handlers } = node;
		if !handlers.is_empty() {
			let found = attach(&id, &mut props, handlers);
			routes.by_id.insert(id.clone(), found);
		}
		let mut seen = HashSet::with_capacity(nodes.len());
		let mut children = Vec::with_capacity(nodes.len());
		for (i, child) in nodes.into_iter().enumerate() {
			let key = match child.props.get("key") {
				Some(Value::String(k)) => k.clone(),
				Some(Value::Number(n)) => n.to_string(),
				_ => i.to_string(),
			};
			let child_id = format!("{id}.{key}");
			if !seen.insert(child_id.clone()) {
				return Err(Error::DuplicateId { path: id, key });
			}
			children.push(Self::build(child_id, child, routes)?);
		}
		Ok(Self { id, kind, props, children })
	}

	/// Builds a wire node with the same validation used by plugin views.
	fn from_json(id: String, node: &Value) -> Result<Self, Error> {
		let Some(obj) = node.as_object() else {
			return Err(Error::InvalidView(format!("{id}: a node must be a table, got {node}")));
		};
		let Some(kind) = obj.get("k").and_then(Value::as_str) else {
			return Err(Error::InvalidView(format!("{id}: a node needs a string `k`")));
		};
		let props = match obj.get("p") {
			None | Some(Value::Null) => Map::new(),
			Some(Value::Object(p)) => p.clone(),
			Some(other) => {
				return Err(Error::InvalidView(format!("{id}: `p` must be a table, got {other}")));
			},
		};
		let nodes: &[Value] = match obj.get("c") {
			None | Some(Value::Null) => &[],
			Some(Value::Array(c)) => c,
			Some(Value::Object(c)) if c.is_empty() => &[],
			Some(other) => {
				return Err(Error::InvalidView(format!("{id}: `c` must be a list, got {other}")));
			},
		};
		let mut seen = HashSet::with_capacity(nodes.len());
		let mut children = Vec::with_capacity(nodes.len());
		for (i, child) in nodes.iter().enumerate() {
			let key = match child.get("p").and_then(|p| p.get("key")) {
				Some(Value::String(k)) => k.clone(),
				Some(Value::Number(n)) => n.to_string(),
				_ => i.to_string(),
			};
			let child_id = format!("{id}.{key}");
			if !seen.insert(child_id.clone()) {
				return Err(Error::DuplicateId { path: id, key });
			}
			children.push(Self::from_json(child_id, child)?);
		}
		Ok(Self { id, kind: kind.to_owned(), props, children })
	}

	/// Whether the `text` prop is primary text, updated with the `text` op.
	fn has_text(&self) -> bool {
		kind::has_text(&self.kind)
	}
}

/// What a handler is routed by.
enum Route {
	/// `action` events with this action (`act` or `act=value`).
	Act(String),
	/// Events of this `ev`.
	Ev(&'static str),
}

/// The handlers of a reconciled view, by node id.
pub(crate) struct Routes<M> {
	/// Each node's handlers, in the order attached.
	by_id: HashMap<String, Vec<(Route, Handler<M>)>>,
}

impl<M> Default for Routes<M> {
	fn default() -> Self {
		Self { by_id: HashMap::new() }
	}
}

impl<M> Routes<M> {
	/// The message of the first handler on node `id` that answers `event`.
	fn at(&self, id: &str, event: &Event, matches: impl Fn(&Route) -> bool) -> Option<M> {
		self
			.by_id
			.get(id)?
			.iter()
			.filter(|(r, _)| matches(r))
			.find_map(|(_, h)| h(event))
	}

	/// Routes `event` to a handler of the node it names: for a list's
	/// `select`/`activate`, the item's own handlers first, then the list's.
	pub(crate) fn route(&self, event: &Event) -> Option<M> {
		let id = event.id()?;
		match event {
			Event::Action(a) => self.at(id, event, |r| {
				matches!(r, Route::Act(n) if *n == a.act || n.split_once('=')
					.is_some_and(|(act, value)| act == a.act && a.value.as_deref() == Some(value)))
			}),
			Event::Select(s) | Event::Activate(s) => {
				let name = event.name();
				let is = |r: &Route| matches!(r, Route::Ev(e) if *e == name);
				(s.item != id)
					.then(|| self.at(&s.item, event, is))
					.flatten()
					.or_else(|| self.at(id, event, is))
			},
			_ => {
				let name = event.name();
				self.at(id, event, |r| matches!(r, Route::Ev(e) if *e == name))
			},
		}
	}
}

/// Merges the actions a node's handlers need into its props and returns
/// their routes.
fn attach<M>(
	id: &str,
	props: &mut Map<String, Value>,
	handlers: Vec<(Trigger, Handler<M>)>,
) -> Vec<(Route, Handler<M>)> {
	let mut out = Vec::with_capacity(handlers.len());
	for (trigger, handler) in handlers {
		let route = match trigger {
			Trigger::Action(act) => Route::Act(act),
			Trigger::Event(ev) => Route::Ev(ev),
			pointer => {
				let actions = props
					.entry("actions")
					.or_insert_with(|| Value::Object(Map::new()));
				let Value::Object(actions) = actions else {
					tracing::warn!(id, "`actions` is not an object here; pointer handler ignored");
					continue;
				};
				match pointer {
					Trigger::Click | Trigger::Dblclick => {
						let gesture = if pointer == Trigger::Click {
							"click"
						} else {
							"dblclick"
						};
						let act = actions
							.entry(gesture)
							.or_insert_with(|| Value::String(gesture.into()))
							.as_str()
							.unwrap_or(gesture)
							.to_owned();
						Route::Act(act)
					},
					Trigger::Menu(act) => {
						let menu = actions
							.entry("menu")
							.or_insert_with(|| Value::Array(Vec::new()));
						if let Value::Array(menu) = menu
							&& !menu.iter().any(|m| m.as_str() == Some(&act))
						{
							menu.push(Value::String(act.clone()));
						}
						Route::Act(act)
					},
					Trigger::Action(_) | Trigger::Event(_) => unreachable!("matched above"),
				}
			},
		};
		out.push((route, handler));
	}
	out
}

/// A view with ids assigned: the tree of each region, as last sent.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Doc {
	/// `main`, `dock`, `layer`.
	regions: [Option<Tree>; 3],
}

impl Doc {
	/// The empty view.
	pub fn new() -> Self {
		Self::default()
	}

	/// Reconciles a view's tree (handlers are dropped).
	///
	/// # Errors
	/// [`Error::DuplicateId`] when two siblings share an id.
	pub fn from_view<M>(view: View<M>) -> Result<Self, Error> {
		Self::build(view).map(|(doc, _)| doc)
	}

	/// Builds a view from its wire form `{main?, dock?, layer?}` of nodes
	/// `{k, p?, c?}` (`null` is the empty view). Region roots must be nodes,
	/// not lists; use [`View`] builders for implicit `col` wrapping.
	///
	/// Missing or null `p` and `c` mean no props or children. An empty
	/// object `c` also means no children (an empty Luau table). Other `p`
	/// values must be objects and other `c` values must be lists. Errors
	/// include the node path; duplicate-key errors also name the key.
	///
	/// # Errors
	/// [`Error::InvalidView`] for a malformed view, [`Error::DuplicateId`]
	/// when two siblings share an id.
	pub fn from_json(value: &Value) -> Result<Self, Error> {
		let mut regions = [None, None, None];
		match value {
			Value::Null => {},
			Value::Object(obj) => {
				if let Some(extra) = obj.keys().find(|k| !REGIONS.contains(&k.as_str())) {
					return Err(Error::InvalidView(format!(
						"view: unknown region {extra:?} (main, dock or layer)"
					)));
				}
				for (slot, name) in regions.iter_mut().zip(REGIONS) {
					if let Some(node) = obj.get(name).filter(|n| !n.is_null()) {
						*slot = Some(Tree::from_json(name.to_owned(), node)?);
					}
				}
			},
			other => {
				return Err(Error::InvalidView(format!(
					"view: expected {{main?, dock?, layer?}}, got {other}"
				)));
			},
		}
		Ok(Self { regions })
	}

	/// Reconciles a view, collecting the routes of its handlers.
	pub(crate) fn build<M>(view: View<M>) -> Result<(Self, Routes<M>), Error> {
		let mut routes = Routes::default();
		let mut regions = [None, None, None];
		let View { main, dock, layer } = view;
		for ((slot, name), node) in regions.iter_mut().zip(REGIONS).zip([main, dock, layer]) {
			if let Some(node) = node {
				*slot = Some(Tree::build(name.to_owned(), node, &mut routes)?);
			}
		}
		Ok((Self { regions }, routes))
	}

	/// Drops transient regions removed when a surface closes.
	pub(crate) fn retain_main(&mut self) {
		self.regions[1] = None;
		self.regions[2] = None;
	}

	/// The region trees in order `main`, `dock`, `layer`.
	pub const fn regions(&self) -> &[Option<Tree>; 3] {
		&self.regions
	}

	/// The ops turning this view into `next` on surface `surface`.
	pub fn ops(&self, next: &Self, surface: &str) -> Vec<Op> {
		let mut ops = Vec::new();
		for (old, new) in self.regions.iter().zip(&next.regions) {
			match (old, new) {
				(None, None) => {},
				(Some(old), None) => ops.push(Op::Del { id: old.id.clone() }),
				(None, Some(new)) => ops.push(add(new, surface, None)),
				(Some(old), Some(new)) if old.kind != new.kind => {
					ops.push(Op::Del { id: old.id.clone() });
					ops.push(add(new, surface, None));
				},
				(Some(old), Some(new)) => diff(old, new, &mut ops),
			}
		}
		ops
	}
}

/// The `add` op of `tree` under `parent`, before `before`.
fn add(tree: &Tree, parent: &str, before: Option<&str>) -> Op {
	Op::Add { parent: parent.to_owned(), before: before.map(str::to_owned), node: tree.wire() }
}

/// Ops turning `old` into `new`, which share id and kind.
fn diff(old: &Tree, new: &Tree, ops: &mut Vec<Op>) {
	let text = new.has_text();
	let mut set = Map::new();
	for (key, value) in &new.props {
		if text && key == "text" {
			continue;
		}
		if old.props.get(key) != Some(value) {
			set.insert(key.clone(), value.clone());
		}
	}
	for key in old.props.keys() {
		let primary = text && key == "text";
		if !primary && !new.props.contains_key(key) {
			set.insert(key.clone(), Value::Null);
		}
	}
	if !set.is_empty() {
		ops.push(Op::Set { id: new.id.clone(), props: set });
	}
	if text {
		let before = old.props.get("text");
		let after = new.props.get("text");
		if before != after {
			match (before.and_then(Value::as_str), after.and_then(Value::as_str)) {
				(Some(a), Some(b)) if !a.is_empty() && b.starts_with(a) => ops.push(Op::Text {
					id:   new.id.clone(),
					mode: TextMode::Append,
					text: b[a.len()..].to_owned(),
				}),
				(_, Some(b)) => ops.push(Op::Text {
					id:   new.id.clone(),
					mode: TextMode::Replace,
					text: b.to_owned(),
				}),
				(_, None) => {
					let mut props = Map::new();
					props.insert("text".into(), Value::Null);
					ops.push(Op::Set { id: new.id.clone(), props });
				},
			}
		}
	}
	children(old, new, ops);
}

/// Ops turning `old`'s children into `new`'s: deletions first, then from
/// the last child back each new, re-kinded or out-of-order child is placed
/// before its (already placed) next sibling, and kept children diff.
fn children(old: &Tree, new: &Tree, ops: &mut Vec<Op>) {
	let before: HashMap<&str, (usize, &Tree)> = old
		.children
		.iter()
		.enumerate()
		.map(|(i, c)| (c.id.as_str(), (i, c)))
		.collect();
	let kept: HashSet<&str> = new.children.iter().map(|c| c.id.as_str()).collect();
	for child in &old.children {
		if !kept.contains(child.id.as_str()) {
			ops.push(Op::Del { id: child.id.clone() });
		}
	}
	// Children that keep id and kind; those on the longest run of rising old
	// positions stay put, the rest move.
	let same: Vec<(usize, usize)> = new
		.children
		.iter()
		.enumerate()
		.filter_map(|(at, c)| {
			before
				.get(c.id.as_str())
				.filter(|(_, o)| o.kind == c.kind)
				.map(|(i, _)| (at, *i))
		})
		.collect();
	let stay: HashSet<usize> = rising(&same.iter().map(|(_, i)| *i).collect::<Vec<_>>())
		.into_iter()
		.map(|k| same[k].0)
		.collect();
	let mut next: Option<&str> = None;
	for (at, child) in new.children.iter().enumerate().rev() {
		match before.get(child.id.as_str()) {
			None => ops.push(add(child, &new.id, next)),
			Some((_, o)) if o.kind != child.kind => {
				ops.push(Op::Del { id: child.id.clone() });
				ops.push(add(child, &new.id, next));
			},
			Some((_, o)) => {
				if !stay.contains(&at) {
					ops.push(Op::Move {
						id:     child.id.clone(),
						parent: new.id.clone(),
						before: next.map(str::to_owned),
					});
				}
				diff(o, child, ops);
			},
		}
		next = Some(child.id.as_str());
	}
}

/// Indices into `seq` of one longest strictly rising subsequence.
fn rising(seq: &[usize]) -> Vec<usize> {
	// tails[k]: index of the smallest tail of a rising run of length k + 1.
	let mut tails: Vec<usize> = Vec::new();
	let mut prev: Vec<Option<usize>> = vec![None; seq.len()];
	for (i, &v) in seq.iter().enumerate() {
		let at = tails.partition_point(|&t| seq[t] < v);
		prev[i] = at.checked_sub(1).map(|p| tails[p]);
		if at == tails.len() {
			tails.push(i);
		} else {
			tails[at] = i;
		}
	}
	let mut out = Vec::with_capacity(tails.len());
	let mut cur = tails.last().copied();
	while let Some(i) = cur {
		out.push(i);
		cur = prev[i];
	}
	out.reverse();
	out
}

#[cfg(test)]
mod tests {
	use serde_json::json;

	use super::*;
	use crate::ui;

	#[test]
	fn json_empty_luau_tables_are_empty_children() {
		let bare = Doc::from_json(&json!({"main": {"k": "col"}})).unwrap();
		for children in [Value::Null, json!({}), json!([])] {
			let doc = Doc::from_json(&json!({
				"main": {"k": "col", "p": null, "c": children},
				"dock": null, "layer": null,
			}))
			.unwrap();
			assert_eq!(doc, bare);
			assert!(doc.ops(&bare, "p").is_empty());
		}
		let nested = Doc::from_json(&json!({
			"main": {"k": "col", "c": [{"k": "col", "p": {}, "c": {}}]},
		}))
		.unwrap();
		assert!(
			nested.regions()[0].as_ref().unwrap().children()[0]
				.children()
				.is_empty()
		);
		assert_eq!(Doc::from_json(&Value::Null).unwrap(), Doc::new());
	}

	#[test]
	fn json_rejects_unknown_regions_even_when_null() {
		for node in [Value::Null, json!({"k": "col"})] {
			let error = Doc::from_json(&json!({"side": node})).unwrap_err();
			assert_eq!(error.to_string(), "view: unknown region \"side\" (main, dock or layer)");
		}
		for value in [json!(1), json!([]), json!("main")] {
			assert!(matches!(Doc::from_json(&value), Err(Error::InvalidView(_))));
		}
	}

	#[test]
	fn json_rejects_malformed_nodes_with_paths() {
		for (node, message) in [
			(json!(1), "a node must be a table, got 1"),
			(json!([]), "a node must be a table, got []"),
			(json!({}), "a node needs a string `k`"),
			(json!({"k": false}), "a node needs a string `k`"),
			(json!({"k": "col", "p": []}), "`p` must be a table, got []"),
			(json!({"k": "col", "p": 1}), "`p` must be a table, got 1"),
			(json!({"k": "col", "c": {"bad": 1}}), "`c` must be a list, got {\"bad\":1}"),
			(json!({"k": "col", "c": false}), "`c` must be a list, got false"),
			(json!({"k": "col", "c": "bad"}), "`c` must be a list, got \"bad\""),
		] {
			let error = Doc::from_json(&json!({"main": node})).unwrap_err();
			assert_eq!(error.to_string(), format!("main: {message}"));
			let error = Doc::from_json(&json!({
				"main": {"k": "col", "c": [{"k": "col", "p": {"key": "nested"}, "c": [node]}]},
			}))
			.unwrap_err();
			assert_eq!(error.to_string(), format!("main.nested.0: {message}"));
		}
	}

	#[test]
	fn duplicate_child_errors_name_parent_and_key() {
		for (keys, path, key) in [
			([json!("a"), json!("a")], "main", "a"),
			([json!(7), json!("7")], "main", "7"),
			([json!("a.b"), json!("a.b")], "main", "a.b"),
		] {
			let children = keys.map(|key| json!({"k": "text", "p": {"key": key}}));
			let node = json!({"k": "col", "c": children});
			let error = Doc::from_json(&json!({"main": node})).unwrap_err();
			assert_eq!(error.to_string(), format!("{path}: duplicate child key {key:?}"));
			assert!(matches!(error, Error::DuplicateId { path: p, key: k } if p == path && k == key));
			let error = Doc::from_json(&json!({
				"main": {"k": "col", "c": [node]},
			}))
			.unwrap_err();
			assert_eq!(error.to_string(), format!("main.0: duplicate child key {key:?}"));
		}
		let view = View::<()>::new().main([ui::text("a").key("x"), ui::text("b").key("x")]);
		assert_eq!(Doc::from_view(view).unwrap_err().to_string(), "main: duplicate child key \"x\"");
	}

	#[test]
	fn integer_keys_derive_decimal_ids() {
		let doc = Doc::from_json(&json!({
			"main": {"k": "col", "c": [
				{"k": "text", "p": {"key": -42}},
				{"k": "text", "p": {"key": u64::MAX}},
			]},
		}))
		.unwrap();
		let children = doc.regions()[0].as_ref().unwrap().children();
		assert_eq!(children[0].id(), "main.-42");
		assert_eq!(children[1].id(), "main.18446744073709551615");
	}

	#[test]
	fn pointer_actions_match_both_name_and_value() {
		let (doc, routes) = Doc::build(View::from(
			ui::text("open")
				.prop("actions", json!({"click": "open=one=two", "dblclick": "open=other"}))
				.on_click(1)
				.on_dblclick(2)
				.on_action("fallback", 3),
		))
		.unwrap();
		assert_eq!(
			doc.regions()[0].as_ref().unwrap().children()[0].props()["actions"],
			json!({"click": "open=one=two", "dblclick": "open=other"})
		);
		for (act, value, expected) in [
			("open", Some("one=two"), Some(1)),
			("open", Some("other"), Some(2)),
			("open", Some("wrong"), None),
			("open", None, None),
			("wrong", Some("one=two"), None),
			("fallback", Some("anything"), Some(3)),
		] {
			let mut raw = json!({"ev": "action", "sf": "s1", "id": "main.0", "act": act});
			if let Some(value) = value {
				raw["value"] = json!(value);
			}
			let event = Event::decode(raw.as_object().unwrap().clone());
			assert_eq!(routes.route(&event), expected);
		}
	}

	#[test]
	fn retaining_main_drops_only_transient_regions() {
		let mut doc = Doc::from_view(
			View::<()>::new()
				.main([ui::text("kept")])
				.dock([ui::text("draft")])
				.layer([ui::text("dialog")]),
		)
		.unwrap();
		let main = doc.regions()[0].clone();
		doc.retain_main();
		assert_eq!(doc.regions(), &[main, None, None]);
		let next = Doc::from_view(View::<()>::new().main([ui::text("kept")])).unwrap();
		assert!(doc.ops(&next, "s1").is_empty());
	}
}
