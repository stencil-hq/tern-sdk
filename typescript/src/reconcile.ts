/**
 * The reconcile layer: views diffed into frame ops with derived ids, matching
 * tern-sdk's reconciler used by Tern's plugin worker and pinned by the shared corpus.
 */
import { flatten, FragmentNode, Node, Span, type Children } from './nodes.js';
import { isTextKind, type Json, type JsonObject, type Op, type WireNode } from './wire.js';

/** The regions of a surface, in the order they are added. */
export const REGIONS = ['main', 'dock', 'layer'] as const;

/** A region of a surface. */
export type RegionName = (typeof REGIONS)[number];

/** A region as `render` takes it: a root node, or a list (or fragment) of nodes wrapped in a `col`. */
export type Region = Children;

/** A view by region. */
export interface ViewInput {
   readonly main?: Region;
   readonly dock?: Region;
   readonly layer?: Region;
}

/** What `render` takes: a view by region, or nodes (a fragment, a list, one node) shown in `main`. */
export type Renderable = ViewInput | Node | Span | readonly Children[] | null;

/** A view rejected before anything was sent (two siblings with one id, a malformed node). */
export class ViewError extends Error {
   override name = 'ViewError';
}

/** One node of a view with its id assigned. */
export class Tree {
   readonly id: string;
   readonly node: Node;
   readonly children: readonly Tree[];

   constructor(id: string, node: Node) {
      this.id = id;
      this.node = node;
      const seen = new Set<string>();
      const children: Tree[] = [];
      node.children.forEach((child, i) => {
         const key = child.props['key'];
         const name =
            typeof key === 'string' ? key : typeof key === 'number' ? String(key) : String(i);
         const childId = `${id}.${name}`;
         if (seen.has(childId))
            throw new ViewError(`${id}: duplicate child key ${JSON.stringify(name)}`);
         seen.add(childId);
         children.push(new Tree(childId, child));
      });
      this.children = children;
   }

   /** The kind. */
   get kind(): string {
      return this.node.kind;
   }

   /** The props. */
   get props(): JsonObject {
      return this.node.props;
   }

   /** The node as a wire subtree with every id. */
   wire(): WireNode {
      const out: { id: string; k: string; p?: JsonObject; c?: WireNode[] } = {
         id: this.id,
         k: this.kind,
      };
      if (Object.keys(this.props).length > 0) out.p = this.props;
      if (this.children.length > 0) out.c = this.children.map(child => child.wire());
      return out;
   }
}

/** A list of children as a `col` region root. */
function column(children: readonly Children[]): Node {
   const nodes: Node[] = [];
   for (const part of flatten(children)) {
      if (!(part instanceof Node)) {
         throw new ViewError('a region lists nodes; wrap text in a text node');
      }
      nodes.push(part);
   }
   return new Node('col', {}, nodes);
}

/** A region's root node. */
function regionRoot(region: Region): Node | undefined {
   if (region === null || region === undefined || typeof region === 'boolean') return undefined;
   if (region instanceof Node && !(region instanceof FragmentNode)) return region;
   return column([region]);
}

/** Whether `view` is a view by region rather than nodes. */
function isViewInput(view: Renderable): view is ViewInput {
   return (
      typeof view === 'object' &&
      view !== null &&
      !Array.isArray(view) &&
      !(view instanceof Node) &&
      !(view instanceof Span)
   );
}

/** Deep JSON equality. */
export function jsonEqual(a: Json | undefined, b: Json | undefined): boolean {
   if (a === b) return true;
   if (typeof a !== 'object' || typeof b !== 'object' || a === null || b === null) return false;
   if (Array.isArray(a) || Array.isArray(b)) {
      if (!Array.isArray(a) || !Array.isArray(b) || a.length !== b.length) return false;
      for (let i = 0; i < a.length; i++) if (!jsonEqual(a[i], b[i])) return false;
      return true;
   }
   const ak = Object.keys(a);
   if (ak.length !== Object.keys(b).length) return false;
   for (const key of ak) {
      if (!(key in b) || !jsonEqual(Reflect.get(a, key), Reflect.get(b, key))) return false;
   }
   return true;
}

/** A view: the tree of each region, as rendered. */
export class View {
   /** `main`, `dock`, `layer`. */
   readonly regions: readonly (Tree | undefined)[];
   #index: Map<string, Tree> | undefined;

   constructor(regions: readonly (Tree | undefined)[] = [undefined, undefined, undefined]) {
      this.regions = regions;
   }

   /** The view of what `render` takes; throws a `ViewError` when two siblings share an id. */
   static from(view: Renderable): View {
      if (view === null) return new View();
      const input: ViewInput = isViewInput(view) ? view : { main: column([view]) };
      return new View(
         REGIONS.map(name => {
            const root = regionRoot(input[name]);
            return root ? new Tree(name, root) : undefined;
         })
      );
   }

   /** The node with id `id`, if the view has it. */
   find(id: string): Tree | undefined {
      if (!this.#index) {
         const index = new Map<string, Tree>();
         const visit = (tree: Tree): void => {
            index.set(tree.id, tree);
            for (const child of tree.children) visit(child);
         };
         for (const region of this.regions) if (region) visit(region);
         this.#index = index;
      }
      return this.#index.get(id);
   }

   /** The ops turning this view into `next` in surface `surface`. */
   ops(next: View, surface: string): Op[] {
      const ops: Op[] = [];
      REGIONS.forEach((_, i) => {
         const old = this.regions[i];
         const now = next.regions[i];
         if (old && !now) ops.push(['del', old.id]);
         else if (!old && now) ops.push(['add', now.id, surface, null, now.wire()]);
         else if (old && now && old.kind !== now.kind) {
            ops.push(['del', old.id]);
            ops.push(['add', now.id, surface, null, now.wire()]);
         } else if (old && now) diff(old, now, ops);
      });
      return ops;
   }
}

/** Ops turning `old` into `now`, which share id and kind. */
function diff(old: Tree, now: Tree, ops: Op[]): void {
   const text = isTextKind(now.kind);
   const set: Record<string, Json> = {};
   let changed = false;
   for (const key in now.props) {
      if (text && key === 'text') continue;
      const value = now.props[key];
      if (value === undefined || jsonEqual(old.props[key], value)) continue;
      set[key] = value;
      changed = true;
   }
   for (const key in old.props) {
      if (!(text && key === 'text') && !(key in now.props)) {
         set[key] = null;
         changed = true;
      }
   }
   if (changed) ops.push(['set', now.id, set]);
   if (text) {
      const before = old.props['text'];
      const after = now.props['text'];
      if (!jsonEqual(before, after)) {
         if (typeof after === 'string') {
            if (typeof before === 'string' && before !== '' && after.startsWith(before)) {
               ops.push(['text', now.id, 'append', after.slice(before.length)]);
            } else {
               ops.push(['text', now.id, 'replace', after]);
            }
         } else {
            ops.push(['set', now.id, { text: null }]);
         }
      }
   }
   children(old, now, ops);
}

/** Indices into `seq` of one longest strictly rising subsequence. */
export function rising(seq: readonly number[]): number[] {
   const tails: number[] = [];
   const prev: (number | undefined)[] = new Array(seq.length);
   seq.forEach((v, i) => {
      let lo = 0;
      let hi = tails.length;
      while (lo < hi) {
         const mid = (lo + hi) >> 1;
         if ((seq[tails[mid] ?? 0] ?? 0) < v) lo = mid + 1;
         else hi = mid;
      }
      prev[i] = lo > 0 ? tails[lo - 1] : undefined;
      tails[lo] = i;
   });
   const out: number[] = [];
   let cur = tails[tails.length - 1];
   while (cur !== undefined) {
      out.push(cur);
      cur = prev[cur];
   }
   return out.reverse();
}

/**
 * Ops turning `old`'s children into `now`'s: deletions first, then from the
 * last child back each new, re-kinded or out-of-order child is placed before
 * its next sibling, each kept child diffed right after it is placed.
 */
function children(old: Tree, now: Tree, ops: Op[]): void {
   const before = new Map<string, { at: number; tree: Tree }>();
   old.children.forEach((tree, at) => before.set(tree.id, { at, tree }));
   const kept = new Set(now.children.map(child => child.id));
   for (const child of old.children) if (!kept.has(child.id)) ops.push(['del', child.id]);
   const same: { at: number; was: number }[] = [];
   now.children.forEach((child, at) => {
      const o = before.get(child.id);
      if (o && o.tree.kind === child.kind) same.push({ at, was: o.at });
   });
   const stay = new Set(rising(same.map(s => s.was)).map(k => same[k]?.at));
   let next: string | null = null;
   for (let at = now.children.length - 1; at >= 0; at--) {
      const child = now.children[at];
      if (!child) continue;
      const o = before.get(child.id);
      if (!o) {
         ops.push(['add', child.id, now.id, next, child.wire()]);
      } else if (o.tree.kind !== child.kind) {
         ops.push(['del', child.id]);
         ops.push(['add', child.id, now.id, next, child.wire()]);
      } else {
         if (!stay.has(at)) ops.push(['move', child.id, now.id, next]);
         diff(o.tree, child, ops);
      }
      next = child.id;
   }
}
