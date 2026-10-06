//! The dynamic node every builder produces, the view of a surface's three
//! regions, and the macros that give every kind's builder the common props
//! and handlers.

use std::{fmt, sync::Arc};

use serde::Serialize;
use serde_json::{Map, Value};

use crate::{
	Error,
	wire::{Action, Change, Edit, Event},
};

/// A routed event turned into a program message (or not, when it doesn't
/// fit the handler).
pub type Handler<M> = Arc<dyn Fn(&Event) -> Option<M> + Send + Sync>;

/// What a handler answers to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Trigger {
	/// `actions.click` (set to `"click"` unless the node names it).
	Click,
	/// `actions.dblclick` (set to `"dblclick"` unless the node names it).
	Dblclick,
	/// A context-menu action, added to `actions.menu`.
	Menu(String),
	/// `action` events with this `act`.
	Action(String),
	/// Events of this `ev` (`toggle`, `select`, `change`, …).
	Event(&'static str),
}

/// A node: a kind, props, children and handlers. It has no id until it is
/// reconciled. Every kind's builder converts into it.
pub struct Node<M = ()> {
	/// The kind.
	pub(crate) kind:     String,
	/// Props.
	pub(crate) props:    Map<String, Value>,
	/// Children in order.
	pub(crate) children: Vec<Self>,
	/// Handlers, in the order they were attached.
	pub(crate) handlers: Vec<(Trigger, Handler<M>)>,
}

impl<M> Clone for Node<M> {
	fn clone(&self) -> Self {
		Self {
			kind:     self.kind.clone(),
			props:    self.props.clone(),
			children: self.children.clone(),
			handlers: self.handlers.clone(),
		}
	}
}

impl<M> fmt::Debug for Node<M> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Node")
			.field("kind", &self.kind)
			.field("props", &self.props)
			.field("children", &self.children)
			.field("handlers", &self.handlers.iter().map(|(t, _)| t).collect::<Vec<_>>())
			.finish()
	}
}

impl<M> Node<M> {
	/// A node of any kind, without props.
	pub fn new(kind: impl Into<String>) -> Self {
		Self {
			kind:     kind.into(),
			props:    Map::new(),
			children: Vec::new(),
			handlers: Vec::new(),
		}
	}

	/// The kind.
	pub fn kind(&self) -> &str {
		&self.kind
	}

	/// The props.
	pub const fn props(&self) -> &Map<String, Value> {
		&self.props
	}

	/// The children.
	pub fn child_nodes(&self) -> &[Self] {
		&self.children
	}

	/// A prop as text: a string, or spans joined.
	pub(crate) fn text_prop(&self, key: &str) -> Option<String> {
		match self.props.get(key)? {
			Value::String(s) => Some(s.clone()),
			Value::Array(spans) => Some(spans_text(spans)),
			_ => None,
		}
	}

	/// Sets a prop; `null` removes it.
	pub(crate) fn set(&mut self, key: &str, value: &impl Serialize) {
		match serde_json::to_value(value) {
			Ok(Value::Null) => {
				self.props.remove(key);
			},
			Ok(value) => {
				self.props.insert(key.to_owned(), value);
			},
			Err(err) => tracing::warn!(key, %err, "prop not serializable; left unset"),
		}
	}

	/// Sets the text of a kind that takes `text` or `spans`.
	pub(crate) fn set_text(&mut self, text: crate::ui::Text) {
		match text {
			crate::ui::Text::Plain(s) => {
				self.props.remove("spans");
				self.props.insert("text".into(), Value::String(s));
			},
			crate::ui::Text::Spans(spans) => {
				self.props.remove("text");
				self.set("spans", &spans);
			},
		}
	}

	/// Builds a node from its wire form `{k, p?, c?}` (no handlers).
	///
	/// # Errors
	/// [`Error::InvalidView`] when it isn't an object with a string `k`, or
	/// `p`/`c` have the wrong shape.
	pub fn from_json(value: &Value) -> Result<Self, Error> {
		let Some(obj) = value.as_object() else {
			return Err(Error::InvalidView(format!("a node must be an object, got {value}")));
		};
		let Some(kind) = obj.get("k").and_then(Value::as_str) else {
			return Err(Error::InvalidView("a node needs a string `k`".into()));
		};
		let props = match obj.get("p") {
			None | Some(Value::Null) => Map::new(),
			Some(Value::Object(p)) => p.clone(),
			Some(other) => {
				return Err(Error::InvalidView(format!("`p` must be an object, got {other}")));
			},
		};
		let children = match obj.get("c") {
			None | Some(Value::Null) => Vec::new(),
			Some(Value::Array(c)) => c.iter().map(Self::from_json).collect::<Result<_, _>>()?,
			Some(other) => return Err(Error::InvalidView(format!("`c` must be a list, got {other}"))),
		};
		Ok(Self { kind: kind.to_owned(), props, children, handlers: Vec::new() })
	}
}

/// Joins the `t` of spans (strings count as themselves).
pub fn spans_text(spans: &[Value]) -> String {
	spans
		.iter()
		.filter_map(|s| match s {
			Value::String(t) => Some(t.as_str()),
			Value::Object(o) => o.get("t").and_then(Value::as_str),
			_ => None,
		})
		.collect()
}

/// A list of nodes is a `col` of them (how a region given as a list is
/// wrapped).
impl<M> From<Vec<Self>> for Node<M> {
	fn from(children: Vec<Self>) -> Self {
		Self { children, ..Self::new("col") }
	}
}

/// An array of nodes is a `col` of them.
impl<M, C: Into<Self>, const N: usize> From<[C; N]> for Node<M> {
	fn from(children: [C; N]) -> Self {
		Self::from(children.into_iter().map(Into::into).collect::<Vec<Self>>())
	}
}

/// Gives access to the node a builder wraps.
pub trait AsNode<M> {
	/// The node.
	fn as_node(&mut self) -> &mut Node<M>;
}

impl<M> AsNode<M> for Node<M> {
	fn as_node(&mut self) -> &mut Self {
		self
	}
}

/// A surface's view: the root of each region (`main`, `dock`, `layer`).
/// A region left out is not shown.
///
/// A node given for a region is the region root itself; a list (a `Vec` or
/// an array) is wrapped in a `col` root. Tern draws a region's children into
/// an element of its own and ignores the root's kind and props (but
/// `role`), so a root should be a container. A bare node or list converts
/// into a view as `main`'s children:
///
/// ```
/// use tern_sdk::ui::{self, View};
///
/// let view: View = ui::md("**hi**").into();
/// let root = view.regions()[0].unwrap();
/// assert_eq!((root.kind(), root.child_nodes()[0].kind()), ("col", "md"));
/// ```
pub struct View<M = ()> {
	/// The flowing document.
	pub(crate) main:  Option<Node<M>>,
	/// Sticky at the pane's bottom while live.
	pub(crate) dock:  Option<Node<M>>,
	/// Overlays.
	pub(crate) layer: Option<Node<M>>,
}

impl<M> Default for View<M> {
	fn default() -> Self {
		Self { main: None, dock: None, layer: None }
	}
}

impl<M> Clone for View<M> {
	fn clone(&self) -> Self {
		Self { main: self.main.clone(), dock: self.dock.clone(), layer: self.layer.clone() }
	}
}

impl<M> fmt::Debug for View<M> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("View")
			.field("main", &self.main)
			.field("dock", &self.dock)
			.field("layer", &self.layer)
			.finish()
	}
}

impl<M> View<M> {
	/// The empty view.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets `main`, the flowing document: a root node, or a list of
	/// children wrapped in a `col`.
	pub fn main(mut self, region: impl Into<Node<M>>) -> Self {
		self.main = Some(region.into());
		self
	}

	/// Sets `dock`, sticky at the pane's bottom: a root node, or a list.
	pub fn dock(mut self, region: impl Into<Node<M>>) -> Self {
		self.dock = Some(region.into());
		self
	}

	/// Sets `layer`, the overlays: a root node, or a list.
	pub fn layer(mut self, region: impl Into<Node<M>>) -> Self {
		self.layer = Some(region.into());
		self
	}

	/// The regions in order `main`, `dock`, `layer`.
	pub const fn regions(&self) -> [Option<&Node<M>>; 3] {
		[self.main.as_ref(), self.dock.as_ref(), self.layer.as_ref()]
	}

	/// Builds a view from its wire form `{main?, dock?, layer?}` (`null` is
	/// the empty view; a region given as a list is wrapped in a `col`).
	///
	/// # Errors
	/// [`Error::InvalidView`] for an unknown region or a malformed node.
	pub fn from_json(value: &Value) -> Result<Self, Error> {
		let mut view = Self::new();
		match value {
			Value::Null => {},
			Value::Object(obj) => {
				for (name, node) in obj {
					let node = match node {
						Value::Null => continue,
						Value::Array(list) => Node::from(
							list
								.iter()
								.map(Node::from_json)
								.collect::<Result<Vec<_>, _>>()?,
						),
						other => Node::from_json(other)?,
					};
					match name.as_str() {
						"main" => view.main = Some(node),
						"dock" => view.dock = Some(node),
						"layer" => view.layer = Some(node),
						other => return Err(Error::InvalidView(format!("unknown region {other:?}"))),
					}
				}
			},
			other => {
				return Err(Error::InvalidView(format!(
					"expected {{main?, dock?, layer?}}, got {other}"
				)));
			},
		}
		Ok(view)
	}
}

/// A bare node is `main`'s one child.
impl<M> From<Node<M>> for View<M> {
	fn from(node: Node<M>) -> Self {
		Self::new().main(vec![node])
	}
}

/// A list of nodes is `main`'s children.
impl<M> From<Vec<Node<M>>> for View<M> {
	fn from(children: Vec<Node<M>>) -> Self {
		Self::new().main(children)
	}
}

/// An array of nodes is `main`'s children.
impl<M, C: Into<Node<M>>, const N: usize> From<[C; N]> for View<M> {
	fn from(children: [C; N]) -> Self {
		Self::new().main(children)
	}
}

/// Wraps a message as a handler that answers events of one shape.
pub fn msg_handler<M: Clone + Send + Sync + 'static>(msg: M) -> Handler<M> {
	Arc::new(move |_| Some(msg.clone()))
}

/// The handler of an `action` event, from its payload.
pub fn action_handler<M, F>(f: F) -> Handler<M>
where
	F: Fn(&Action) -> M + Send + Sync + 'static,
{
	Arc::new(move |e| match e {
		Event::Action(a) => Some(f(a)),
		_ => None,
	})
}

/// The handler of a `toggle` event.
pub fn toggle_handler<M, F>(f: F) -> Handler<M>
where
	F: Fn(bool) -> M + Send + Sync + 'static,
{
	Arc::new(move |e| match e {
		Event::Toggle(t) => Some(f(t.collapsed)),
		_ => None,
	})
}

/// The handler of a `select` (or `activate`) event, from its item.
pub fn item_handler<M, F>(f: F, activate: bool) -> Handler<M>
where
	F: Fn(&str) -> M + Send + Sync + 'static,
{
	Arc::new(move |e| match (e, activate) {
		(Event::Select(s), false) | (Event::Activate(s), true) => Some(f(&s.item)),
		_ => None,
	})
}

/// The handler of a `change` event.
pub fn change_handler<M, F>(f: F) -> Handler<M>
where
	F: Fn(&Change) -> M + Send + Sync + 'static,
{
	Arc::new(move |e| match e {
		Event::Change(c) => Some(f(c)),
		_ => None,
	})
}

/// The handler of an `edit` event.
pub fn edit_handler<M, F>(f: F) -> Handler<M>
where
	F: Fn(&Edit) -> M + Send + Sync + 'static,
{
	Arc::new(move |e| match e {
		Event::Edit(ed) => Some(f(ed)),
		_ => None,
	})
}

/// The handler of a `send` event, from its text.
pub fn send_handler<M, F>(f: F) -> Handler<M>
where
	F: Fn(&str) -> M + Send + Sync + 'static,
{
	Arc::new(move |e| match e {
		Event::Send(s) => Some(f(&s.text)),
		_ => None,
	})
}

/// The props every node takes, except `title`, `max` and `actions`.
macro_rules! base_methods {
	() => {
		/// Identity among siblings: keeps the node's id (and Tern's view
		/// state) across reorders. A string, or an integer.
		pub fn key(mut self, key: impl Into<$crate::ui::NodeKey>) -> Self {
			$crate::ui::node::AsNode::as_node(&mut self).set("key", &key.into());
			self
		}

		/// A name of yours, exposed to CSS as `data-role`.
		pub fn role(mut self, role: impl Into<String>) -> Self {
			$crate::ui::node::AsNode::as_node(&mut self).set("role", &role.into());
			self
		}

		/// The semantic color of the node's chrome.
		pub fn tone(mut self, tone: impl Into<$crate::ui::Tone>) -> Self {
			$crate::ui::node::AsNode::as_node(&mut self).set("tone", &tone.into());
			self
		}

		/// Keeps the node mounted but not laid out.
		pub fn hidden(mut self, hidden: bool) -> Self {
			$crate::ui::node::AsNode::as_node(&mut self).set("hidden", &hidden);
			self
		}

		/// A transient selection mark drawn over the node.
		pub fn mark(mut self, mark: impl Into<$crate::ui::Mark>) -> Self {
			$crate::ui::node::AsNode::as_node(&mut self).set("mark", &mark.into());
			self
		}

		/// The accessible name of a node that shows no text.
		pub fn aria(mut self, aria: impl Into<String>) -> Self {
			$crate::ui::node::AsNode::as_node(&mut self).set("aria", &aria.into());
			self
		}

		/// A link: ⌘-click opens it; `copy` and `open` use it.
		pub fn href(mut self, href: impl Into<String>) -> Self {
			$crate::ui::node::AsNode::as_node(&mut self).set("href", &href.into());
			self
		}

		/// Flex grow inside a `row` or `col`.
		pub fn grow(mut self, grow: f64) -> Self {
			$crate::ui::node::AsNode::as_node(&mut self).set("grow", &grow);
			self
		}

		/// Flex shrink inside a `row` or `col`.
		pub fn shrink(mut self, shrink: f64) -> Self {
			$crate::ui::node::AsNode::as_node(&mut self).set("shrink", &shrink);
			self
		}

		/// Flex basis: a fraction of the parent, or the content's size.
		pub fn basis(mut self, basis: impl Into<$crate::ui::Basis>) -> Self {
			$crate::ui::node::AsNode::as_node(&mut self).set("basis", &basis.into());
			self
		}

		/// Lower size bounds.
		pub fn min(mut self, min: $crate::ui::Bound) -> Self {
			$crate::ui::node::AsNode::as_node(&mut self).set("min", &min);
			self
		}

		/// Sets any prop untyped; `null` removes it.
		pub fn prop(mut self, name: &str, value: impl ::serde::Serialize) -> Self {
			$crate::ui::node::AsNode::as_node(&mut self).set(name, &value);
			self
		}
	};
}

/// The common `title` prop (a tooltip).
macro_rules! title_method {
	() => {
		/// A tooltip.
		pub fn title(mut self, title: impl Into<String>) -> Self {
			$crate::ui::node::AsNode::as_node(&mut self).set("title", &title.into());
			self
		}
	};
}

/// The common `max` prop.
macro_rules! max_method {
	() => {
		/// Upper size bounds (a `max.h` also clips the node).
		pub fn max(mut self, max: $crate::ui::Bound) -> Self {
			$crate::ui::node::AsNode::as_node(&mut self).set("max", &max);
			self
		}
	};
}

/// The common `actions` prop and the pointer handlers.
macro_rules! actions_methods {
	() => {
		/// What a pointer does, by action name.
		pub fn actions(mut self, actions: $crate::ui::Actions) -> Self {
			$crate::ui::node::AsNode::as_node(&mut self).set("actions", &actions);
			self
		}

		/// Sends `msg` on a click (sets `actions.click` to `"click"` unless
		/// the node names its click action itself).
		pub fn on_click(mut self, msg: M) -> Self
		where
			M: Clone + Send + Sync + 'static,
		{
			$crate::ui::node::AsNode::as_node(&mut self)
				.handlers
				.push(($crate::ui::node::Trigger::Click, $crate::ui::node::msg_handler(msg)));
			self
		}

		/// Sends `msg` on a double click (sets `actions.dblclick` to
		/// `"dblclick"` unless the node names it).
		pub fn on_dblclick(mut self, msg: M) -> Self
		where
			M: Clone + Send + Sync + 'static,
		{
			$crate::ui::node::AsNode::as_node(&mut self)
				.handlers
				.push(($crate::ui::node::Trigger::Dblclick, $crate::ui::node::msg_handler(msg)));
			self
		}

		/// Adds `act` to the context menu and sends `msg` when it is picked.
		pub fn on_menu(mut self, act: impl Into<String>, msg: M) -> Self
		where
			M: Clone + Send + Sync + 'static,
		{
			$crate::ui::node::AsNode::as_node(&mut self)
				.handlers
				.push((
					$crate::ui::node::Trigger::Menu(act.into()),
					$crate::ui::node::msg_handler(msg),
				));
			self
		}
	};
}

/// Handlers of the events a node sends.
macro_rules! event_methods {
	() => {
		/// Sends `msg` on `action` events whose action is `act` (`name` or
		/// `name=value`).
		pub fn on_action(mut self, act: impl Into<String>, msg: M) -> Self
		where
			M: Clone + Send + Sync + 'static,
		{
			$crate::ui::node::AsNode::as_node(&mut self)
				.handlers
				.push((
					$crate::ui::node::Trigger::Action(act.into()),
					$crate::ui::node::msg_handler(msg),
				));
			self
		}

		/// Maps `action` events whose action is `act` to a message.
		pub fn on_action_with(
			mut self,
			act: impl Into<String>,
			f: impl Fn(&$crate::wire::Action) -> M + Send + Sync + 'static,
		) -> Self {
			$crate::ui::node::AsNode::as_node(&mut self)
				.handlers
				.push((
					$crate::ui::node::Trigger::Action(act.into()),
					$crate::ui::node::action_handler(f),
				));
			self
		}

		/// Maps `toggle` events (the new `collapsed`) to a message.
		pub fn on_toggle(mut self, f: impl Fn(bool) -> M + Send + Sync + 'static) -> Self {
			$crate::ui::node::AsNode::as_node(&mut self)
				.handlers
				.push((
					$crate::ui::node::Trigger::Event("toggle"),
					$crate::ui::node::toggle_handler(f),
				));
			self
		}

		/// Maps `select` events (the item) to a message. On a list's item the
		/// item's own handler runs first.
		pub fn on_select(mut self, f: impl Fn(&str) -> M + Send + Sync + 'static) -> Self {
			$crate::ui::node::AsNode::as_node(&mut self)
				.handlers
				.push((
					$crate::ui::node::Trigger::Event("select"),
					$crate::ui::node::item_handler(f, false),
				));
			self
		}

		/// Maps `activate` events (the item) to a message.
		pub fn on_activate(mut self, f: impl Fn(&str) -> M + Send + Sync + 'static) -> Self {
			$crate::ui::node::AsNode::as_node(&mut self)
				.handlers
				.push((
					$crate::ui::node::Trigger::Event("activate"),
					$crate::ui::node::item_handler(f, true),
				));
			self
		}

		/// Maps `change` events to a message.
		pub fn on_change(
			mut self,
			f: impl Fn(&$crate::wire::Change) -> M + Send + Sync + 'static,
		) -> Self {
			$crate::ui::node::AsNode::as_node(&mut self)
				.handlers
				.push((
					$crate::ui::node::Trigger::Event("change"),
					$crate::ui::node::change_handler(f),
				));
			self
		}

		/// Sends `msg` on `focus` events (a click asked for the keys).
		pub fn on_focus(mut self, msg: M) -> Self
		where
			M: Clone + Send + Sync + 'static,
		{
			$crate::ui::node::AsNode::as_node(&mut self)
				.handlers
				.push(($crate::ui::node::Trigger::Event("focus"), $crate::ui::node::msg_handler(msg)));
			self
		}

		/// Maps `edit` events (native editing) to a message.
		pub fn on_edit(
			mut self,
			f: impl Fn(&$crate::wire::Edit) -> M + Send + Sync + 'static,
		) -> Self {
			$crate::ui::node::AsNode::as_node(&mut self)
				.handlers
				.push(($crate::ui::node::Trigger::Event("edit"), $crate::ui::node::edit_handler(f)));
			self
		}

		/// Sends `msg` on `undo` events.
		pub fn on_undo(mut self, msg: M) -> Self
		where
			M: Clone + Send + Sync + 'static,
		{
			$crate::ui::node::AsNode::as_node(&mut self)
				.handlers
				.push(($crate::ui::node::Trigger::Event("undo"), $crate::ui::node::msg_handler(msg)));
			self
		}

		/// Maps `send` events (the text to submit) to a message.
		pub fn on_send(mut self, f: impl Fn(&str) -> M + Send + Sync + 'static) -> Self {
			$crate::ui::node::AsNode::as_node(&mut self)
				.handlers
				.push(($crate::ui::node::Trigger::Event("send"), $crate::ui::node::send_handler(f)));
			self
		}
	};
}

/// Children of a container kind.
macro_rules! children_methods {
	() => {
		/// Adds a child.
		pub fn child(mut self, child: impl Into<$crate::ui::Node<M>>) -> Self {
			$crate::ui::node::AsNode::as_node(&mut self)
				.children
				.push(child.into());
			self
		}

		/// Adds children.
		pub fn children<C: Into<$crate::ui::Node<M>>>(
			mut self,
			children: impl IntoIterator<Item = C>,
		) -> Self {
			$crate::ui::node::AsNode::as_node(&mut self)
				.children
				.extend(children.into_iter().map(Into::into));
			self
		}
	};
}

/// Typed props: `method(Type) = "key";` sets prop `key` from
/// `impl Into<Type>`.
macro_rules! props {
	($($(#[$m:meta])* $method:ident($ty:ty) = $key:literal;)*) => {
		$(
			$(#[$m])*
			pub fn $method(mut self, value: impl Into<$ty>) -> Self {
				let value: $ty = value.into();
				$crate::ui::node::AsNode::as_node(&mut self).set($key, &value);
				self
			}
		)*
	};
}

/// Defines a kind's builder type wrapping a [`Node`].
macro_rules! kind_type {
	($(#[$m:meta])* $name:ident) => {
		$(#[$m])*
		pub struct $name<M = ()>(pub(crate) $crate::ui::Node<M>);

		impl<M> Clone for $name<M> {
			fn clone(&self) -> Self {
				Self(self.0.clone())
			}
		}

		impl<M> ::std::fmt::Debug for $name<M> {
			fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
				self.0.fmt(f)
			}
		}

		impl<M> From<$name<M>> for $crate::ui::Node<M> {
			fn from(n: $name<M>) -> Self {
				n.0
			}
		}

		/// A bare node is `main`'s one child.
		impl<M> From<$name<M>> for $crate::ui::View<M> {
			fn from(n: $name<M>) -> Self {
				Self::from(n.0)
			}
		}

		impl<M> $crate::ui::node::AsNode<M> for $name<M> {
			fn as_node(&mut self) -> &mut $crate::ui::Node<M> {
				&mut self.0
			}
		}
	};
}

pub(crate) use actions_methods;
pub(crate) use base_methods;
pub(crate) use children_methods;
pub(crate) use event_methods;
pub(crate) use kind_type;
pub(crate) use max_method;
pub(crate) use props;
pub(crate) use title_method;

impl<M> Node<M> {
	base_methods!();

	title_method!();

	max_method!();

	actions_methods!();

	event_methods!();

	children_methods!();
}
