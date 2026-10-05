/** The TSX development runtime: the same elements as `jsx-runtime`. */
import { jsx, type Component, type JSX } from './jsx-runtime.js';

export { Fragment, jsx, jsxs, type Component, type JSX, type SpanProps } from './jsx-runtime.js';

/** Builds one TSX element in development builds (source info is ignored). */
export function jsxDEV(
   type: string | Component,
   props: Readonly<Record<string, unknown>>,
   key?: string | number
): JSX.Element {
   return jsx(type, props, key);
}
