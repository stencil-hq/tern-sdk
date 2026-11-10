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
//!
//! A node the view shares with the last render (a clone of the same
//! [`Node`], not an equal copy) at the same id keeps that render's tree: it
//! is neither rebuilt nor diffed, and its handlers stay routed. A view that
//! keeps its unchanged subtrees as clones costs a render only its changes.

use std::{
	collections::{HashMap, HashSet},
	fmt::{self, Write as _},
	sync::Arc,
};

use serde_json::{Map, Value};

use crate::{
	Error,
	ui::{
		Node, View,
		node::{Handler, Inner, Trigger},
	},
	wire::{Event, Op, TextMode, WireNode, kind},
};

/// The regions of a surface, in the order they are added.
pub const REGIONS: [&str; 3] = ["main", "dock", "layer"];

/// One node of a view with its id assigned. Cloning is O(1): the content is
/// shared.
#[derive(Clone)]
pub struct Tree(Arc<Branch>);

/// What a [`Tree`] holds.
struct Branch {
	/// `parent.(key or index)`, or the region name at the top.
	id:       String,
	/// The kind.
	kind:     String,
	/// Props as the view gave them (with handler actions merged in).
	props:    Map<String, Value>,
	/// Children in order.
	children: Vec<Tree>,
	/// The address of the shared node this was built from, or 0. The
	/// render that built it holds that node, so while it is the last render
	/// a node at this address is that node, unchanged.
	source:   usize,
}

impl fmt::Debug for Tree {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Tree")
			.field("id", &self.0.id)
			.field("kind", &self.0.kind)
			.field("props", &self.0.props)
			.field("children", &self.0.children)
			.finish()
	}
}

impl PartialEq for Tree {
	fn eq(&self, other: &Self) -> bool {
		let (a, b) = (&*self.0, &*other.0);
		Arc::ptr_eq(&self.0, &other.0)
			|| (a.id == b.id && a.kind == b.kind && a.props == b.props && a.children == b.children)
	}
}

impl Tree {
	/// The node's id.
	pub fn id(&self) -> &str {
		&self.0.id
	}

	/// The node's kind.
	pub fn kind(&self) -> &str {
		&self.0.kind
	}

	/// The node's props.
	pub fn props(&self) -> &Map<String, Value> {
		&self.0.props
	}

	/// The node's children.
	pub fn children(&self) -> &[Self] {
		&self.0.children
	}

	/// The node as an `add` op carries it, ids at every level.
	pub fn wire(&self) -> WireNode {
		WireNode {
			id: self.0.id.clone(),
			k:  self.0.kind.clone(),
			p:  self.0.props.clone(),
			c:  self.0.children.iter().map(Self::wire).collect(),
		}
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
		Ok(Self(Arc::new(Branch { id, kind: kind.to_owned(), props, children, source: 0 })))
	}
}

/// One render in progress: what it builds, and what of the last render it
/// keeps or drops.
struct Build<'a, M> {
	/// See [`Rendered::sources`].
	sources:  Vec<Node<M>>,
	/// The handlers of the nodes built, by id, in build order.
	handlers: Vec<(String, Vec<(Route, Handler<M>)>)>,
	/// Ids of the last render's nodes built again.
	replaced: Vec<&'a str>,
	/// The last render's subtrees no node has now.
	dropped:  Vec<&'a Tree>,
	/// Whether a tree of the last render is kept.
	reused:   bool,
}

impl<'a, M> Build<'a, M> {
	/// The tree of `node` under `id`, where the last render had `last`.
	/// `held`: a node above is in `sources`.
	fn tree(
		&mut self,
		id: &str,
		node: Node<M>,
		last: Option<&'a Tree>,
		held: bool,
	) -> Result<Tree, Error> {
		let at = std::ptr::from_ref(node.inner()).addr();
		if let Some(last) = last.filter(|t| t.0.source == at) {
			self.reused = true;
			if !held {
				self.sources.push(node);
			}
			return Ok(last.clone());
		}
		visited();
		if let Some(last) = last {
			self.replaced.push(&last.0.id);
		}
		// A node no clone shares is used up; a shared one is held, so its
		// address names it until the next render.
		let (inner, source, held) = match node.try_into_inner() {
			Ok(inner) => (inner, 0, held),
			Err(node) => {
				let inner = node.inner().clone();
				if !held {
					self.sources.push(node);
				}
				(inner, at, true)
			},
		};
		let Inner { kind, mut props, children: mut nodes, handlers, slots } = inner;
		// Slotted children come after the body, as in every SDK, so a body
		// child's index (and the id derived from it) doesn't depend on its
		// slots.
		let slotted = |c: &Node<M>| {
			c.inner()
				.props
				.get("key")
				.and_then(Value::as_str)
				.is_some_and(|k| slots.contains(&k))
		};
		nodes.sort_by_key(slotted);
		for slot in &slots {
			props.insert((*slot).into(), Value::String(format!("{id}.{slot}")));
		}
		if !handlers.is_empty() {
			let found = attach(id, &mut props, handlers);
			self.handlers.push((id.to_owned(), found));
		}
		let old = last.map_or(&[][..], Tree::children);
		// While each child has the id of the last render's child at its
		// index, the ids are distinct as those were; from the first that
		// doesn't, `seen` holds every id so far.
		let mut seen: Option<HashSet<String>> = None;
		let mut by_id: Option<HashMap<&'a str, &'a Tree>> = None;
		let mut children: Vec<Tree> = Vec::with_capacity(nodes.len());
		let mut child_id = String::with_capacity(id.len() + 8);
		for (i, child) in nodes.into_iter().enumerate() {
			child_id.clear();
			child_id.push_str(id);
			child_id.push('.');
			let key_at = child_id.len();
			match child.inner().props.get("key") {
				Some(Value::String(k)) => child_id.push_str(k),
				Some(Value::Number(n)) => {
					let _ = write!(child_id, "{n}");
				},
				_ => {
					let _ = write!(child_id, "{i}");
				},
			}
			let at_index = old.get(i).filter(|t| t.0.id == child_id);
			if at_index.is_none() || seen.is_some() {
				let seen =
					seen.get_or_insert_with(|| children.iter().map(|c| c.0.id.clone()).collect());
				if !seen.insert(child_id.clone()) {
					return Err(Error::DuplicateId {
						path: id.to_owned(),
						key:  child_id[key_at..].to_owned(),
					});
				}
			}
			let last = at_index.or_else(|| {
				by_id
					.get_or_insert_with(|| old.iter().map(|t| (t.0.id.as_str(), t)).collect())
					.get(child_id.as_str())
					.copied()
			});
			children.push(self.tree(&child_id, child, last, held)?);
		}
		match &seen {
			None => self
				.dropped
				.extend(old.get(children.len()..).unwrap_or_default()),
			Some(seen) => self
				.dropped
				.extend(old.iter().filter(|t| !seen.contains(&t.0.id))),
		}
		Ok(Tree(Arc::new(Branch { id: id.to_owned(), kind, props, children, source })))
	}
}

/// Drops the handlers of `tree`'s nodes.
fn forget<M>(by_id: &mut HashMap<String, Vec<(Route, Handler<M>)>>, tree: &Tree) {
	by_id.remove(&tree.0.id);
	for child in &tree.0.children {
		forget(by_id, child);
	}
}

#[cfg(test)]
thread_local! {
	/// Nodes built or diffed, for tests that bound a render's work.
	static VISITED: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Counts a node built or diffed (in tests).
fn visited() {
	#[cfg(test)]
	VISITED.with(|n| n.set(n.get() + 1));
}

/// What a handler is routed by.
enum Route {
	/// `action` events with this action (`act` or `act=value`).
	Act(String),
	/// Events of this `ev`.
	Ev(&'static str),
}

/// A view as last rendered: its trees, its handlers by node id, and the
/// nodes its trees were built from, held so the next render can keep the
/// tree of a node it shares.
pub(crate) struct Rendered<M> {
	/// The trees.
	doc:     Doc,
	/// Each node's handlers, in the order attached.
	by_id:   HashMap<String, Vec<(Route, Handler<M>)>>,
	/// The topmost shared nodes the trees were built from. Holding them
	/// keeps each tree's source alive and unchanged, as changing a node
	/// that a clone shares copies it first.
	sources: Vec<Node<M>>,
}

impl<M> Default for Rendered<M> {
	fn default() -> Self {
		Self { doc: Doc::new(), by_id: HashMap::new(), sources: Vec::new() }
	}
}

impl<M> Rendered<M> {
	/// Reconciles `view`, keeping the tree of each node it shares with
	/// this render at the same id, and becomes its render (unchanged on
	/// error).
	///
	/// # Errors
	/// [`Error::DuplicateId`] when two siblings share an id.
	pub(crate) fn render(&mut self, view: View<M>) -> Result<Doc, Error> {
		let mut build = Build {
			sources:  Vec::new(),
			handlers: Vec::new(),
			replaced: Vec::new(),
			dropped:  Vec::new(),
			reused:   false,
		};
		let mut regions = [None, None, None];
		let View { main, dock, layer } = view;
		let nodes = [main, dock, layer].into_iter().zip(&self.doc.regions);
		for ((slot, name), (node, last)) in regions.iter_mut().zip(REGIONS).zip(nodes) {
			match node {
				Some(node) => *slot = Some(build.tree(name, node, last.as_ref(), false)?),
				None => build.dropped.extend(last),
			}
		}
		let Build { sources, handlers, replaced, dropped, reused } = build;
		if reused {
			for id in replaced {
				self.by_id.remove(id);
			}
			for tree in dropped {
				forget(&mut self.by_id, tree);
			}
			self.by_id.extend(handlers);
		} else {
			self.by_id = handlers.into_iter().collect();
		}
		self.sources = sources;
		self.doc = Doc { regions };
		Ok(self.doc.clone())
	}

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
				.is_some_and(|(act, value)| {
					act == a.act && a.value.as_ref().and_then(Value::as_str) == Some(value)
				}))
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
				match &pointer {
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
					Trigger::Menu(act) | Trigger::Hold(act) => {
						let list = if matches!(pointer, Trigger::Hold(_)) {
							"hold"
						} else {
							"menu"
						};
						let menu = actions
							.entry(list)
							.or_insert_with(|| Value::Array(Vec::new()));
						if let Value::Array(menu) = menu
							&& !menu
								.iter()
								.any(|m| m.as_str().or_else(|| m.get("id")?.as_str()) == Some(act.as_str()))
						{
							menu.push(Value::String(act.clone()));
						}
						Route::Act(act.clone())
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
		Rendered::default().render(view)
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
				(Some(old), None) => ops.push(Op::Del { id: old.0.id.clone() }),
				(None, Some(new)) => ops.push(add(new, surface, None)),
				(Some(old), Some(new)) if old.0.kind != new.0.kind => {
					ops.push(Op::Del { id: old.0.id.clone() });
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

/// Ops turning `old` into `new`, which share id and kind: none when they
/// are one tree.
fn diff(old: &Tree, new: &Tree, ops: &mut Vec<Op>) {
	if Arc::ptr_eq(&old.0, &new.0) {
		return;
	}
	visited();
	let (old, new) = (&*old.0, &*new.0);
	let text = kind::has_text(&new.kind);
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
fn children(old: &Branch, new: &Branch, ops: &mut Vec<Op>) {
	let same_ids = old.children.len() == new.children.len()
		&& old
			.children
			.iter()
			.zip(&new.children)
			.all(|(o, n)| Arc::ptr_eq(&o.0, &n.0) || o.0.id == n.0.id);
	if same_ids {
		// What the general case does when nothing is deleted or moved.
		let mut next: Option<&str> = None;
		for (o, child) in old.children.iter().zip(&new.children).rev() {
			if o.0.kind == child.0.kind {
				diff(o, child, ops);
			} else {
				ops.push(Op::Del { id: child.0.id.clone() });
				ops.push(add(child, &new.id, next));
			}
			next = Some(child.0.id.as_str());
		}
		return;
	}
	let before: HashMap<&str, (usize, &Tree)> = old
		.children
		.iter()
		.enumerate()
		.map(|(i, c)| (c.0.id.as_str(), (i, c)))
		.collect();
	let kept: HashSet<&str> = new.children.iter().map(|c| c.0.id.as_str()).collect();
	for child in &old.children {
		if !kept.contains(child.0.id.as_str()) {
			ops.push(Op::Del { id: child.0.id.clone() });
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
				.get(c.0.id.as_str())
				.filter(|(_, o)| o.0.kind == c.0.kind)
				.map(|(i, _)| (at, *i))
		})
		.collect();
	let stay: HashSet<usize> = rising(&same.iter().map(|(_, i)| *i).collect::<Vec<_>>())
		.into_iter()
		.map(|k| same[k].0)
		.collect();
	let mut next: Option<&str> = None;
	for (at, child) in new.children.iter().enumerate().rev() {
		match before.get(child.0.id.as_str()) {
			None => ops.push(add(child, &new.id, next)),
			Some((_, o)) if o.0.kind != child.0.kind => {
				ops.push(Op::Del { id: child.0.id.clone() });
				ops.push(add(child, &new.id, next));
			},
			Some((_, o)) => {
				if !stay.contains(&at) {
					ops.push(Op::Move {
						id:     child.0.id.clone(),
						parent: new.id.clone(),
						before: next.map(str::to_owned),
					});
				}
				diff(o, child, ops);
			},
		}
		next = Some(child.0.id.as_str());
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
	fn slotted_nodes_become_keyed_children_sent_by_id() {
		let view: View = View::new().main([crate::ui::card()
			.head(crate::ui::text("Jobs"))
			.end(crate::ui::button("Refresh"))
			.end(crate::ui::button("Reload"))
			.child(crate::ui::text("body"))
			.key("panel")]);
		let doc = Doc::from_view(view).unwrap();
		let card = &doc.regions()[0].as_ref().unwrap().children()[0];
		assert_eq!(card.props()["head"], json!("main.panel.head"));
		assert_eq!(card.props()["end"], json!("main.panel.end"));
		let ids: Vec<&str> = card.children().iter().map(Tree::id).collect();
		assert_eq!(ids, ["main.panel.0", "main.panel.head", "main.panel.end"]);
		assert_eq!(card.children()[2].props()["text"], json!("Reload"));
	}

	#[test]
	fn text_after_a_slotted_head_drops_the_child() {
		let card = crate::ui::card::<()>()
			.head(crate::ui::text("x"))
			.head("Plain");
		let doc = Doc::from_view(View::new().main([card])).unwrap();
		let card = &doc.regions()[0].as_ref().unwrap().children()[0];
		assert_eq!(card.props()["head"], json!("Plain"));
		assert!(card.children().is_empty());
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
		let mut routes = Rendered::default();
		let doc = routes
			.render(View::from(
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

	/// Entry `i` of a long list: its text and a clickable label.
	fn entry(i: u32, text: &str) -> Node<u32> {
		ui::col()
			.key(format!("e{i}"))
			.child(ui::text(text))
			.child(ui::text("tap").on_click(i))
			.into()
	}

	/// A click on node `id`.
	fn click(id: &str) -> Event {
		let raw = json!({"ev": "action", "sf": "s1", "id": id, "act": "click"});
		Event::decode(raw.as_object().unwrap().clone())
	}

	/// What `f` returns, and the nodes it built or diffed.
	fn visits<T>(f: impl FnOnce() -> T) -> (T, usize) {
		VISITED.with(|n| n.set(0));
		let out = f();
		(out, VISITED.with(std::cell::Cell::get))
	}

	#[test]
	fn a_render_builds_and_diffs_only_what_changed() {
		let view = |entries: &[Node<u32>]| View::new().main(entries.to_vec());
		let fresh = |entries: &[Node<u32>]| Doc::from_view(view(entries)).unwrap();
		let mut entries: Vec<Node<u32>> = (0..500).map(|i| entry(i, &format!("entry {i}"))).collect();
		let mut rendered = Rendered::default();
		let first = rendered.render(view(&entries)).unwrap();
		let before = entries.clone();
		entries[250] = entry(250, "entry 250, edited");
		let ((second, ops), visited) = visits(|| {
			let second = rendered.render(view(&entries)).unwrap();
			let ops = first.ops(&second, "s1");
			(second, ops)
		});
		assert_eq!(ops, [Op::Text {
			id:   "main.e250.0".into(),
			mode: TextMode::Append,
			text: ", edited".into(),
		}]);
		// The root and the edited entry with its two children, each built
		// and diffed once; the other 499 entries are neither.
		assert_eq!(visited, 8);
		// The same trees and ops as views reconciled afresh.
		assert_eq!(second, fresh(&entries));
		assert_eq!(ops, fresh(&before).ops(&fresh(&entries), "s1"));
		for i in [0, 250, 499] {
			assert_eq!(rendered.route(&click(&format!("main.e{i}.1"))), Some(i));
		}

		// A dropped entry's handlers go with it.
		let kept = entries[..499].to_vec();
		let third = rendered.render(view(&kept)).unwrap();
		assert_eq!(second.ops(&third, "s1"), [Op::Del { id: "main.e499".into() }]);
		assert_eq!(rendered.route(&click("main.e499.1")), None);
		assert_eq!(rendered.route(&click("main.e498.1")), Some(498));

		// The same root again is no work at all.
		let root = Node::from(kept);
		let fourth = rendered.render(View::new().main(root.clone())).unwrap();
		let (ops, visited) = visits(|| {
			let fifth = rendered.render(View::new().main(root.clone())).unwrap();
			fourth.ops(&fifth, "s1")
		});
		assert!(ops.is_empty());
		assert_eq!(visited, 0);
		assert_eq!(rendered.route(&click("main.e3.1")), Some(3));
	}

	#[test]
	fn a_failed_render_keeps_the_last() {
		let mut rendered = Rendered::default();
		let first = rendered
			.render(View::new().main([entry(1, "one")]))
			.unwrap();
		let duplicate = View::new().main([entry(2, "two"), entry(2, "again")]);
		assert!(matches!(rendered.render(duplicate), Err(Error::DuplicateId { .. })));
		assert_eq!(rendered.route(&click("main.e1.1")), Some(1));
		assert_eq!(rendered.route(&click("main.e2.1")), None);
		let again = rendered
			.render(View::new().main([entry(1, "one")]))
			.unwrap();
		assert!(first.ops(&again, "s1").is_empty());
	}

	#[test]
	fn reordered_and_inserted_children_diff_as_when_built_afresh() {
		let view = |entries: &[Node<u32>]| View::new().main(entries.to_vec());
		let fresh = |entries: &[Node<u32>]| Doc::from_view(view(entries)).unwrap();
		let entries: Vec<Node<u32>> = (0..6).map(|i| entry(i, &format!("entry {i}"))).collect();
		let mut rendered = Rendered::default();
		let first = rendered.render(view(&entries)).unwrap();
		let next = [&entries[3], &entries[0], &entries[1]]
			.into_iter()
			.cloned()
			.chain([entry(9, "new"), entry(4, "entry 4, edited"), entries[5].clone()])
			.collect::<Vec<_>>();
		let second = rendered.render(view(&next)).unwrap();
		assert_eq!(second, fresh(&next));
		assert_eq!(first.ops(&second, "s1"), fresh(&entries).ops(&fresh(&next), "s1"));
		for i in [0, 3, 4, 5, 9] {
			assert_eq!(rendered.route(&click(&format!("main.e{i}.1"))), Some(i));
		}
		assert_eq!(rendered.route(&click("main.e2.1")), None);
	}
}
