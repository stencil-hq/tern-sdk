/**
 * Plain function builders for every kind (`ui.card({head}, …children)`)
 * and every `el` tag (`html.button({onClick}, 'Create')`), sharing the node
 * model with TSX. The `html` builders double as TSX components
 * (`<html.form class="ask">`).
 */
import {
   createNode,
   Node,
   Span,
   type Children,
   type Handlers,
   type TextChildren,
} from './nodes.js';
import type { BuildKind, CommonProps, ControlProps, ElTag, HtmlProps, KindProps } from './props.js';

/** Kinds that take no children. */
export type LeafKind =
   | 'rule'
   | 'spacer'
   | 'diff'
   | 'rows'
   | 'image'
   | 'kv'
   | 'table'
   | 'tree'
   | 'kbd'
   | 'icon'
   | 'item'
   | 'tabs'
   | 'spinner'
   | 'elapsed'
   | 'rate'
   | 'progress'
   | 'meter'
   | 'chart'
   | 'effort'
   | 'checklist';

/** Kinds whose children are plain text, taken as the `text` prop. */
export type PlainTextKind =
   | 'md'
   | 'code'
   | 'ansi'
   | 'math'
   | 'editor'
   | 'input'
   | 'badge'
   | 'toast';

/** The children a kind takes. */
export type ChildrenOf<K extends BuildKind> = K extends LeafKind
   ? never
   : K extends PlainTextKind
     ? TextChildren
     : Children;

/** A kind's props: its own, the common ones it doesn't redefine, and handlers. */
export type PropsOf<K extends BuildKind> = Omit<CommonProps, keyof KindProps[K]> &
   KindProps[K] &
   Handlers;

/** A kind's props with its children, as TSX takes them. */
export type ElementProps<K extends BuildKind> = PropsOf<K> & { readonly children?: ChildrenOf<K> };

/** A builder of one kind: props first (optional), then children. */
export interface Builder<K extends BuildKind> {
   (props?: PropsOf<K> | null, ...children: ChildrenOf<K>[]): Node;
   (...children: ChildrenOf<K>[]): Node;
}

/** Whether a builder's first argument is its props rather than a child. */
function isProps(value: unknown): value is object {
   return (
      typeof value === 'object' &&
      value !== null &&
      !Array.isArray(value) &&
      !(value instanceof Node) &&
      !(value instanceof Span)
   );
}

/** Builds `kind` from a builder call's arguments. */
function build(kind: string, extra: object | null, args: readonly unknown[]): Node {
   const [first, ...rest] = args;
   if (first === null || isProps(first)) {
      return createNode(kind, first === null ? extra : { ...extra, ...first }, rest);
   }
   return createNode(kind, extra, args);
}

/** The builder of `kind`. */
function builder<K extends BuildKind>(kind: K): Builder<K> {
   return (...args: unknown[]) => build(kind, null, args);
}

/** Builders for every kind but Tern's own `block`. */
export const ui: { readonly [K in BuildKind]: Builder<K> } = {
   col: builder('col'),
   row: builder('row'),
   card: builder('card'),
   section: builder('section'),
   rule: builder('rule'),
   spacer: builder('spacer'),
   text: builder('text'),
   md: builder('md'),
   code: builder('code'),
   diff: builder('diff'),
   ansi: builder('ansi'),
   rows: builder('rows'),
   math: builder('math'),
   image: builder('image'),
   kv: builder('kv'),
   table: builder('table'),
   tree: builder('tree'),
   badge: builder('badge'),
   kbd: builder('kbd'),
   icon: builder('icon'),
   list: builder('list'),
   item: builder('item'),
   tabs: builder('tabs'),
   picker: builder('picker'),
   spinner: builder('spinner'),
   shimmer: builder('shimmer'),
   elapsed: builder('elapsed'),
   rate: builder('rate'),
   progress: builder('progress'),
   meter: builder('meter'),
   chart: builder('chart'),
   effort: builder('effort'),
   editor: builder('editor'),
   input: builder('input'),
   status: builder('status'),
   seg: builder('seg'),
   toast: builder('toast'),
   overlay: builder('overlay'),
   tool: builder('tool'),
   agent: builder('agent'),
   checklist: builder('checklist'),
   prefs: builder('prefs'),
   el: builder('el'),
};

/** Tags that take no children. */
type VoidTag = 'input' | 'hr';

/** The props of an `el` of tag `T`, as builders and TSX take them. */
export type HtmlElementProps<T extends ElTag> = Omit<CommonProps, keyof ControlProps> &
   (T extends 'input' ? ControlProps : HtmlProps) &
   Handlers & { readonly children?: T extends VoidTag ? never : Children };

/** A builder (and TSX component) of one `el` tag. */
export interface HtmlBuilder<T extends ElTag> {
   (
      props?: HtmlElementProps<T> | null,
      ...children: (T extends VoidTag ? never : Children)[]
   ): Node;
   (...children: (T extends VoidTag ? never : Children)[]): Node;
}

/** The builder of `el` tag `tag`. */
function tag<T extends ElTag>(name: T): HtmlBuilder<T> {
   return (...args: unknown[]) => build('el', { tag: name }, args);
}

/** Builders and TSX components for every `el` tag: `<html.button onClick={go}>Go</html.button>`. */
export const html: { readonly [T in ElTag]: HtmlBuilder<T> } = {
   div: tag('div'),
   span: tag('span'),
   p: tag('p'),
   section: tag('section'),
   header: tag('header'),
   footer: tag('footer'),
   nav: tag('nav'),
   aside: tag('aside'),
   main: tag('main'),
   article: tag('article'),
   figure: tag('figure'),
   blockquote: tag('blockquote'),
   ul: tag('ul'),
   ol: tag('ol'),
   li: tag('li'),
   dl: tag('dl'),
   dt: tag('dt'),
   dd: tag('dd'),
   h1: tag('h1'),
   h2: tag('h2'),
   h3: tag('h3'),
   h4: tag('h4'),
   pre: tag('pre'),
   code: tag('code'),
   kbd: tag('kbd'),
   strong: tag('strong'),
   b: tag('b'),
   em: tag('em'),
   i: tag('i'),
   del: tag('del'),
   mark: tag('mark'),
   hr: tag('hr'),
   table: tag('table'),
   thead: tag('thead'),
   tbody: tag('tbody'),
   tr: tag('tr'),
   th: tag('th'),
   td: tag('td'),
   label: tag('label'),
   button: tag('button'),
   form: tag('form'),
   input: tag('input'),
};
