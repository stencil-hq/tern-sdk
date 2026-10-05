/**
 * The node layer: the node model every builder (TSX, `ui`, `html`, `node`)
 * shares, styled spans, and the handlers a node carries.
 */
import type { SpanData, SpanFx, SpanToken } from './props.js';
import { isJsonObject } from './wire.js';
import type {
   ActionEvent,
   ActivateEvent,
   ChangeEvent,
   EditEvent,
   FocusEvent,
   Json,
   JsonObject,
   SelectEvent,
   SendEvent,
   ToggleEvent,
   UndoEvent,
} from './wire.js';

/** A handler of one event; it may be async. */
export type Handler<E> = (event: E) => void | Promise<void>;

/** Handlers a node carries instead of naming actions by hand. */
export interface Handlers {
   /** Runs on the node's `click` action; sets `actions.click` to `"click"` unless named. */
   readonly onClick?: Handler<ActionEvent>;
   /** Runs on the node's `dblclick` action; sets `actions.dblclick` to `"dblclick"` unless named. */
   readonly onDblClick?: Handler<ActionEvent>;
   /** Context-menu entries by action name; the names join `actions.menu`. */
   readonly onMenu?: Readonly<Record<string, Handler<ActionEvent>>>;
   /** Handlers of `action` events by `act`. */
   readonly onAction?: Readonly<Record<string, Handler<ActionEvent>>>;
   readonly onToggle?: Handler<ToggleEvent>;
   readonly onSelect?: Handler<SelectEvent>;
   readonly onActivate?: Handler<ActivateEvent>;
   readonly onChange?: Handler<ChangeEvent>;
   readonly onFocus?: Handler<FocusEvent>;
   readonly onEdit?: Handler<EditEvent>;
   readonly onUndo?: Handler<UndoEvent>;
   readonly onSend?: Handler<SendEvent>;
}

/** A wire node without an id: kind, props, children. */
export interface NodeJson {
   readonly k: string;
   readonly p?: JsonObject;
   readonly c?: readonly NodeJson[];
}

/**
 * A node: a kind, props, children and handlers. It has no id until a
 * surface reconciles it.
 */
export class Node {
   /** The TSP kind. */
   readonly kind: string;
   /** Props as they go on the wire (handler-derived `actions` included). */
   readonly props: JsonObject;
   /** Children in order. */
   readonly children: readonly Node[];
   /** Handlers routed to by the node's id. */
   readonly handlers: Handlers | undefined;

   constructor(
      kind: string,
      props: JsonObject = {},
      children: readonly Node[] = [],
      handlers?: Handlers
   ) {
      this.kind = kind;
      this.props = props;
      this.children = children;
      this.handlers = handlers;
   }

   /** The node as JSON without ids (`{k, p?, c?}`). */
   toJSON(): NodeJson {
      const out: { k: string; p?: JsonObject; c?: NodeJson[] } = { k: this.kind };
      if (Object.keys(this.props).length > 0) out.p = this.props;
      if (this.children.length > 0) out.c = this.children.map(child => child.toJSON());
      return out;
   }
}

/** Children grouped without a node of their own (`<>…</>`); flattened into the parent. */
export class FragmentNode extends Node {
   /** The children, strings and spans kept for text kinds. */
   readonly parts: readonly Part[];

   constructor(parts: readonly Part[], props: JsonObject = {}) {
      super('#fragment', props, []);
      this.parts = parts;
   }
}

/** One styled run of text, as `span()` and `<span>` build it. */
export class Span implements SpanData {
   readonly t: string;
   readonly s?: SpanToken;
   readonly fx?: SpanFx;
   readonly href?: string;

   constructor(t: string, s?: SpanToken, options: { fx?: SpanFx; href?: string } = {}) {
      this.t = t;
      if (s !== undefined && s !== '') this.s = s;
      if (options.fx !== undefined) this.fx = options.fx;
      if (options.href !== undefined) this.href = options.href;
   }
}

/** A styled span: text, style tokens (`"muted"`, `"strong accent"`), effect and link. */
export function span(t: string, s?: SpanToken, options: { fx?: SpanFx; href?: string } = {}): Span {
   return new Span(t, s, options);
}

/** One child as builders take it; `null`, `undefined` and booleans are skipped. */
export type Child = Node | Span | string | number | boolean | null | undefined;

/** Children: a child or nested lists of them. */
export type Children = Child | readonly Children[];

/** A child after flattening. */
export type Part = Node | Span | string;

/** Text-only children: strings and numbers, nested in lists. */
export type TextChildren = string | number | boolean | null | undefined | readonly TextChildren[];

/** Flattens children into parts: lists and fragments opened, skipped values dropped. */
export function flatten(input: unknown, out: Part[] = []): Part[] {
   if (input === null || input === undefined || typeof input === 'boolean') return out;
   if (typeof input === 'string') out.push(input);
   else if (typeof input === 'number') out.push(String(input));
   else if (Array.isArray(input)) for (const item of input) flatten(item, out);
   else if (input instanceof FragmentNode) for (const part of input.parts) out.push(part);
   else if (input instanceof Node || input instanceof Span) out.push(input);
   else throw new TypeError(`not a node, span or string: ${JSON.stringify(input)}`);
   return out;
}

/** A value made JSON for the wire: `undefined` and functions dropped, spans as plain objects. */
export function toJson(value: unknown): Json | undefined {
   switch (typeof value) {
      case 'string':
      case 'boolean':
         return value;
      case 'number':
         return Number.isFinite(value) ? value : null;
      case 'object': {
         if (value === null) return null;
         if (value instanceof Node)
            throw new TypeError(`a ${value.kind} node can't be a prop value`);
         if (Array.isArray(value)) return value.map(item => toJson(item) ?? null);
         const out: Record<string, Json> = {};
         for (const key in value) {
            const json = toJson(Reflect.get(value, key));
            if (json !== undefined) out[key] = json;
         }
         return out;
      }
      default:
         return undefined;
   }
}

/** Whether `value` is a function, taken as a handler of `E`. */
function isHandler<E>(value: unknown): value is Handler<E> {
   return typeof value === 'function';
}

/** A map of handlers by action name, or undefined when `value` isn't one. */
function handlerMap(value: unknown): Record<string, Handler<ActionEvent>> | undefined {
   if (typeof value !== 'object' || value === null) return undefined;
   const out: Record<string, Handler<ActionEvent>> = {};
   for (const name in value) {
      const fn: unknown = Reflect.get(value, name);
      if (isHandler<ActionEvent>(fn)) out[name] = fn;
   }
   return out;
}

type Mutable<T> = { -readonly [K in keyof T]: T[K] };

/** Takes handler prop `key` into `handlers`; false when `key` isn't a handler prop. */
function takeHandler(handlers: Mutable<Handlers>, key: string, value: unknown): boolean {
   switch (key) {
      case 'onClick':
         if (isHandler<ActionEvent>(value)) handlers.onClick = value;
         return true;
      case 'onDblClick':
         if (isHandler<ActionEvent>(value)) handlers.onDblClick = value;
         return true;
      case 'onMenu': {
         const map = handlerMap(value);
         if (map) handlers.onMenu = map;
         return true;
      }
      case 'onAction': {
         const map = handlerMap(value);
         if (map) handlers.onAction = map;
         return true;
      }
      case 'onToggle':
         if (isHandler<ToggleEvent>(value)) handlers.onToggle = value;
         return true;
      case 'onSelect':
         if (isHandler<SelectEvent>(value)) handlers.onSelect = value;
         return true;
      case 'onActivate':
         if (isHandler<ActivateEvent>(value)) handlers.onActivate = value;
         return true;
      case 'onChange':
         if (isHandler<ChangeEvent>(value)) handlers.onChange = value;
         return true;
      case 'onFocus':
         if (isHandler<FocusEvent>(value)) handlers.onFocus = value;
         return true;
      case 'onEdit':
         if (isHandler<EditEvent>(value)) handlers.onEdit = value;
         return true;
      case 'onUndo':
         if (isHandler<UndoEvent>(value)) handlers.onUndo = value;
         return true;
      case 'onSend':
         if (isHandler<SendEvent>(value)) handlers.onSend = value;
         return true;
      default:
         return false;
   }
}

/** `actions` with what the click, double-click and menu handlers need added. */
function withActions(props: Record<string, Json>, handlers: Handlers): void {
   const current = props['actions'];
   if (Array.isArray(current)) return;
   const actions: Record<string, Json> = {};
   if (isJsonObject(current)) {
      for (const key in current) {
         const value = current[key];
         if (value !== undefined) actions[key] = value;
      }
   }
   if (handlers.onClick && actions['click'] === undefined) actions['click'] = 'click';
   if (handlers.onDblClick && actions['dblclick'] === undefined) actions['dblclick'] = 'dblclick';
   if (handlers.onMenu) {
      const old = actions['menu'];
      const menu: Json[] = Array.isArray(old) ? [...old] : [];
      for (const name in handlers.onMenu) if (!menu.includes(name)) menu.push(name);
      actions['menu'] = menu;
   }
   if (Object.keys(actions).length > 0) props['actions'] = actions;
}

/** Kinds whose string and span children become `text` or `spans`. */
const SPAN_TEXT: Readonly<Record<string, true>> = { text: true, shimmer: true, seg: true };

/** Kinds whose string children become the `text` prop. */
const PLAIN_TEXT: Readonly<Record<string, true>> = {
   md: true,
   code: true,
   ansi: true,
   math: true,
   editor: true,
   input: true,
   badge: true,
   toast: true,
};

/** A run of strings and spans: `{text}` when all plain, else `{spans}`. */
function textProps(run: readonly (string | Span)[]): Record<string, Json> {
   if (run.every(part => typeof part === 'string')) return { text: run.join('') };
   return { spans: run.map(part => (typeof part === 'string' ? part : (toJson(part) ?? null))) };
}

/** Node children of a container: runs of strings and spans become `text` nodes. */
function containerChildren(parts: readonly Part[]): Node[] {
   const out: Node[] = [];
   let run: (string | Span)[] = [];
   const end = (): void => {
      if (run.length > 0) out.push(new Node('text', textProps(run)));
      run = [];
   };
   for (const part of parts) {
      if (part instanceof Node) {
         end();
         out.push(part);
      } else {
         run.push(part);
      }
   }
   end();
   return out;
}

/** Builds a node of `kind` from builder props (handler props included) and children. */
export function createNode(
   kind: string,
   input: object | null | undefined,
   children: unknown
): Node {
   const props: Record<string, Json> = {};
   const handlers: Mutable<Handlers> = {};
   let fromProps: unknown;
   if (input) {
      for (const key in input) {
         const value: unknown = Reflect.get(input, key);
         if (key === 'children') {
            fromProps = value;
            continue;
         }
         if (takeHandler(handlers, key, value)) continue;
         const json = toJson(value);
         if (json !== undefined) props[key] = json;
      }
   }
   const parts = flatten([fromProps, children]);
   let nodes: Node[] = [];
   if (SPAN_TEXT[kind] || PLAIN_TEXT[kind]) {
      const run: (string | Span)[] = [];
      for (const part of parts) {
         if (part instanceof Node && kind === 'seg') nodes.push(part);
         else if (part instanceof Node || (part instanceof Span && PLAIN_TEXT[kind])) {
            throw new TypeError(`${kind} takes text children, got ${describe(part)}`);
         } else run.push(part);
      }
      if (run.length > 0) Object.assign(props, textProps(run));
   } else if (kind === 'el') {
      let text = '';
      for (const part of parts) {
         if (part instanceof Span) throw new TypeError('el takes no spans; use html.span');
         if (typeof part === 'string') {
            if (nodes.length === 0) text += part;
            else nodes.push(new Node('el', { tag: 'span', text: part }));
         } else {
            nodes.push(part);
         }
      }
      if (text !== '') props['text'] = text;
   } else {
      nodes = containerChildren(parts);
   }
   const any = Object.keys(handlers).length > 0;
   if (any) withActions(props, handlers);
   return new Node(kind, props, nodes, any ? handlers : undefined);
}

/** A short description of a part for errors. */
function describe(part: Part): string {
   if (part instanceof Node) return `a ${part.kind} node`;
   if (part instanceof Span) return 'a span';
   return JSON.stringify(part);
}

/** Builds any kind, untyped: props (handler props included) and children. */
export function node(
   kind: string,
   props?: Readonly<Record<string, unknown>> | null,
   children?: Children
): Node {
   return createNode(kind, props ?? null, children);
}

/** The node with `key` set (when it has none), for keys given outside its props. */
export function withKey(target: Node, key: string | number | undefined): Node {
   if (key === undefined || target.props['key'] !== undefined) return target;
   if (target instanceof FragmentNode)
      return new FragmentNode(target.parts, { ...target.props, key });
   return new Node(target.kind, { ...target.props, key }, target.children, target.handlers);
}

/** Converts a JSON node without ids (`{k, p?, c?}`) into a node. */
export function fromJson(json: Json): Node {
   if (!isJsonObject(json)) {
      throw new TypeError(`a node must be an object, got ${JSON.stringify(json)}`);
   }
   const k = json['k'];
   if (typeof k !== 'string') throw new TypeError('a node needs a string `k`');
   const p = json['p'];
   const c = json['c'];
   const props: JsonObject = isJsonObject(p) ? p : {};
   const children = Array.isArray(c) ? c.map(fromJson) : [];
   return new Node(k, props, children);
}
