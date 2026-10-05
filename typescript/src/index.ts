/**
 * `@stencil-hq/tern`: talk to Tern over the Tern Surface Protocol. Six
 * layers, each usable without the ones above it: wire, input, nodes,
 * reconcile, session and helpers.
 */
export * from './wire.js';
export { InputParser, MAX_INPUT_MESSAGE, type InputItem } from './input.js';
export { KeyDecoder, type Key, type KeyName } from './keys.js';
export type * from './props.js';
export { EL_TAGS } from './props.js';
export {
   flatten,
   fromJson,
   FragmentNode,
   node,
   Node,
   Span,
   span,
   toJson,
   type Child,
   type Children,
   type Handler,
   type Handlers,
   type NodeJson,
   type Part,
   type TextChildren,
} from './nodes.js';
export {
   html,
   ui,
   type Builder,
   type ChildrenOf,
   type ElementProps,
   type HtmlBuilder,
   type HtmlElementProps,
   type LeafKind,
   type PlainTextKind,
   type PropsOf,
} from './ui.js';
export { Fragment, type Component, type JSX, type SpanProps } from './jsx-runtime.js';
export { css } from './css.js';
export {
   jsonEqual,
   REGIONS,
   rising,
   Tree,
   View,
   ViewError,
   type Region,
   type RegionName,
   type Renderable,
   type ViewInput,
} from './reconcile.js';
export {
   Surface,
   type CloseOptions,
   type Dispatch,
   type OpenOptions,
   type Palette,
   type SurfaceLink,
} from './surface.js';
export {
   connect,
   Session,
   type Capabilities,
   type ConnectOptions,
   type SessionInput,
   type TermInput,
   type TermOutput,
} from './session.js';
export { bar, plain } from './plain.js';
export { print, type PrintOptions } from './print.js';
export { ask, type Answer, type AskOptions, type Unsupported } from './ask.js';
