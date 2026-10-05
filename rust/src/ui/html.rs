//! `el` builders, one per allowed tag.
//!
//! Plain HTML elements a program styles with its own sheet, and checkbox
//! and radio controls whose flips come back as `change` events; a form's
//! values ride on every event from inside it.
//!
//! # Example
//! ```
//! use tern_sdk::ui::{Actions, Node, html};
//!
//! let form: Node = html::form()
//! 	.class("ask")
//! 	.child(html::p("Which size?"))
//! 	.child(
//! 		html::label()
//! 			.child(html::radio("size", "s"))
//! 			.child(html::span("Small")),
//! 	)
//! 	.child(html::button("Create").actions(Actions::new().click("submit")))
//! 	.into();
//! assert_eq!(form.child_nodes().len(), 3);
//! ```

use serde::Serialize;
use serde_json::{Map, Value};

use super::{
	InputType, Node,
	node::{
		actions_methods, base_methods, children_methods, event_methods, kind_type, max_method, props,
		title_method,
	},
};

kind_type!(
	/// `el`: one HTML element of an allowed tag, its `text`, then its
	/// children.
	El
);

impl<M> El<M> {
	base_methods!();

	title_method!();

	max_method!();

	actions_methods!();

	event_methods!();

	children_methods!();

	props! {
		/// The element's tag.
		tag(String) = "tag";
		/// Space-separated class names for your stylesheets.
		class(String) = "class";
		/// Plain text drawn before the children.
		text(String) = "text";
		/// `input` only: checkbox or radio.
		input_type(InputType) = "type";
		/// `input` only: the name in its form's values and its radio group.
		name(String) = "name";
		/// `input` only: the value it reports (default `"on"`).
		value(String) = "value";
		/// `input` only: the checked state you set.
		checked(bool) = "checked";
		/// `input` only: dimmed, never flips, left out of values.
		disabled(bool) = "disabled";
	}

	/// Sets all extra attributes (`data-*`, `aria-*`, `role`, `colspan`,
	/// `rowspan`).
	pub fn attrs(mut self, attrs: Map<String, Value>) -> Self {
		self.0.props.insert("attrs".into(), Value::Object(attrs));
		self
	}

	/// Sets one extra attribute (a string, number or boolean).
	pub fn attr(mut self, name: impl Into<String>, value: impl Serialize) -> Self {
		let Ok(value) = serde_json::to_value(value) else {
			return self;
		};
		if let Some(Value::Object(attrs)) = self.0.props.get_mut("attrs") {
			attrs.insert(name.into(), value);
		} else {
			let mut attrs = Map::new();
			attrs.insert(name.into(), value);
			self.0.props.insert("attrs".into(), Value::Object(attrs));
		}
		self
	}
}

/// An `el` of any tag.
pub fn el<M>(tag: impl Into<String>) -> El<M> {
	El(Node::new("el")).tag(tag)
}

/// Defines builders for tags without text.
macro_rules! bare_tags {
	($($(#[$m:meta])* $f:ident = $tag:literal;)*) => {
		$($(#[$m])* pub fn $f<M>() -> El<M> {
			el($tag)
		})*
	};
}

/// Defines builders for tags that take their text.
macro_rules! text_tags {
	($($(#[$m:meta])* $f:ident = $tag:literal;)*) => {
		$($(#[$m])* pub fn $f<M>(text: impl Into<String>) -> El<M> {
			el($tag).text(text)
		})*
	};
}

bare_tags! {
	/// `div`: a block.
	div = "div";
	/// `section`.
	section = "section";
	/// `header`.
	header = "header";
	/// `footer`.
	footer = "footer";
	/// `nav`.
	nav = "nav";
	/// `aside`.
	aside = "aside";
	/// `main`.
	main = "main";
	/// `article`.
	article = "article";
	/// `figure`.
	figure = "figure";
	/// `blockquote`.
	blockquote = "blockquote";
	/// `ul`: a bulleted list.
	ul = "ul";
	/// `ol`: a numbered list.
	ol = "ol";
	/// `dl`: a description list.
	dl = "dl";
	/// `hr`: a rule (no children).
	hr = "hr";
	/// `table`.
	table = "table";
	/// `thead`.
	thead = "thead";
	/// `tbody`.
	tbody = "tbody";
	/// `tr`: a table row.
	tr = "tr";
	/// `label`: names the controls inside it; a click flips the first.
	label = "label";
	/// `form`: groups controls; events from inside carry its values.
	form = "form";
}

text_tags! {
	/// `span`: inline text.
	span = "span";
	/// `p`: a paragraph.
	p = "p";
	/// `li`: a list item.
	li = "li";
	/// `dt`: a term.
	dt = "dt";
	/// `dd`: a definition.
	dd = "dd";
	/// `h1`.
	h1 = "h1";
	/// `h2`.
	h2 = "h2";
	/// `h3`.
	h3 = "h3";
	/// `h4`.
	h4 = "h4";
	/// `pre`: preformatted text.
	pre = "pre";
	/// `code`: inline code.
	code = "code";
	/// `kbd`: a key.
	kbd = "kbd";
	/// `strong`.
	strong = "strong";
	/// `b`.
	b = "b";
	/// `em`.
	em = "em";
	/// `i`.
	i = "i";
	/// `del`: deleted text.
	del = "del";
	/// `mark`: highlighted text.
	mark = "mark";
	/// `th`: a header cell.
	th = "th";
	/// `td`: a cell.
	td = "td";
	/// `button`: give it `actions` (or `on_click`) to make it do something.
	button = "button";
}

/// `input`: a control of `kind`.
pub fn input<M>(kind: impl Into<InputType>) -> El<M> {
	el("input").input_type(kind)
}

/// A checkbox named `name` reporting `value` (a lone box reports
/// `true`/`false` in its form's values).
pub fn checkbox<M>(name: impl Into<String>, value: impl Into<String>) -> El<M> {
	input(InputType::Checkbox).name(name).value(value)
}

/// A radio of group `name` reporting `value`.
pub fn radio<M>(name: impl Into<String>, value: impl Into<String>) -> El<M> {
	input(InputType::Radio).name(name).value(value)
}
