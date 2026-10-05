/**
 * The TSX runtime (`jsxImportSource: '@stencil-hq/tern'`). Lowercase
 * intrinsic elements are TSP kinds with typed props; `<span>` builds a
 * styled span inside text kinds; `html.*` components build `el` nodes.
 */
import { createNode, flatten, FragmentNode, Node, Span, withKey, type Children } from './nodes.js';
import type { BuildKind, SpanFx, SpanToken } from './props.js';
import type { ElementProps } from './ui.js';

/** The props of the `<span>` intrinsic. */
export interface SpanProps {
   /** Space-separated style tokens. */
   readonly s?: SpanToken;
   readonly fx?: SpanFx;
   readonly href?: string;
   readonly children?: string | number | readonly (string | number)[];
}

/** A function component. */
export type Component = (props: Readonly<Record<string, unknown>>) => Node | Span | null;

/** The TSX types. */
export declare namespace JSX {
   /** What a TSX expression builds. */
   export type Element = Node | Span;
   /** Every kind but `block`, with typed props, plus `span`. */
   export type IntrinsicElements = { [K in BuildKind]: ElementProps<K> } & { span: SpanProps };
   /** Props every element takes. */
   export interface IntrinsicAttributes {
      readonly key?: string | number;
   }
   /** Where TSX puts children. */
   export interface ElementChildrenAttribute {
      children: unknown;
   }
}

/** Builds the span of a `<span>` element. */
function spanOf(props: Readonly<Record<string, unknown>>): Span {
   let t = '';
   for (const part of flatten(props['children'])) {
      if (typeof part !== 'string') throw new TypeError('a span takes text children only');
      t += part;
   }
   const s = props['s'];
   const fx = props['fx'];
   const href = props['href'];
   const options: { fx?: string; href?: string } = {};
   if (typeof fx === 'string') options.fx = fx;
   if (typeof href === 'string') options.href = href;
   return new Span(t, typeof s === 'string' ? s : undefined, options);
}

/** Builds one TSX element. */
export function jsx(
   type: string | Component,
   props: Readonly<Record<string, unknown>>,
   key?: string | number
): JSX.Element {
   if (typeof type === 'function') {
      const built = type(key === undefined ? props : { ...props, key });
      if (built === null) return new FragmentNode([]);
      return built instanceof Node ? withKey(built, key) : built;
   }
   if (type === 'span') return spanOf(props);
   return createNode(type, key === undefined ? props : { ...props, key }, undefined);
}

/** Builds a TSX element with static children (same as `jsx`). */
export const jsxs = jsx;

/** `<>…</>`: children without a node of their own. */
export function Fragment(props: { readonly children?: Children }): Node {
   return new FragmentNode(flatten(props.children));
}
