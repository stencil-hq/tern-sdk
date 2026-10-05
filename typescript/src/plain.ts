/**
 * The plain-text fallback renderer: a view as readable text without
 * escapes, for output outside Tern. It is not a copy of Tern's layout.
 */
import { View, type Renderable, type Tree } from './reconcile.js';
import { isJsonObject, type Json, type JsonObject } from './wire.js';

/** Tags `el` draws as blocks, each on lines of its own. */
const BLOCK_TAGS: Readonly<Record<string, true>> = {
   div: true,
   p: true,
   section: true,
   header: true,
   footer: true,
   nav: true,
   aside: true,
   main: true,
   article: true,
   figure: true,
   blockquote: true,
   ul: true,
   ol: true,
   li: true,
   dl: true,
   dt: true,
   dd: true,
   h1: true,
   h2: true,
   h3: true,
   h4: true,
   pre: true,
   hr: true,
   table: true,
   thead: true,
   tbody: true,
   tr: true,
   form: true,
};

/** The indent of a card's body and a nested list. */
const INDENT = '  ';

/** Checklist item marks by status. */
const CHECK_MARKS: Readonly<Record<string, string>> = {
   done: 'x',
   active: '>',
   dropped: '-',
   blocked: '!',
};

/** CSI, OSC and other escape sequences, for stripping ANSI text. */
const ESCAPES =
   // oxlint-disable-next-line no-control-regex
   /\x1b(?:\[[0-?]*[ -/]*[@-~]|\][^\x07\x1b]*(?:\x07|\x1b\\)|[PX^_][^\x1b]*\x1b\\|[@-Z\\-_])/g;

/** The plain text of spans or a string. */
function spansText(value: Json | undefined): string {
   if (typeof value === 'string') return value;
   if (typeof value === 'number') return String(value);
   if (!Array.isArray(value)) return '';
   let out = '';
   for (const part of value) {
      if (typeof part === 'string') out += part;
      else if (isJsonObject(part) && typeof part['t'] === 'string') out += part['t'];
   }
   return out;
}

function prop(props: JsonObject, key: string): string {
   return spansText(props[key]);
}

function numProp(props: JsonObject, key: string): number | undefined {
   const value = props[key];
   return typeof value === 'number' ? value : undefined;
}

function list(props: JsonObject, key: string): JsonObject[] {
   const value = props[key];
   return Array.isArray(value) ? value.filter(isJsonObject) : [];
}

/** `[####----] 50%` for a fraction, `[--------]` when unknown. */
export function bar(value: number | undefined, width = 20): string {
   if (value === undefined) return `[${'-'.repeat(width)}]`;
   const v = Math.min(1, Math.max(0, value));
   const fill = Math.round(v * width);
   return `[${'#'.repeat(fill)}${'-'.repeat(width - fill)}] ${Math.round(v * 100)}%`;
}

/** A duration in ms as `short` (`3m 05s`) or `clock` (`3:05`) reads it. */
function duration(ms: number, format: string): string {
   const total = Math.floor(Math.abs(ms) / 1000);
   const h = Math.floor(total / 3600);
   const m = Math.floor((total % 3600) / 60);
   const s = total % 60;
   const two = (n: number): string => String(n).padStart(2, '0');
   if (format === 'clock') return h > 0 ? `${h}:${two(m)}:${two(s)}` : `${m}:${two(s)}`;
   if (h > 0) return `${h}h ${two(m)}m`;
   if (m > 0) return `${m}m ${two(s)}s`;
   return `${s}s`;
}

/** Rows of cells as aligned columns; `right[i]` right-aligns column i. */
function columns(rows: readonly (readonly string[])[], right: readonly boolean[] = []): string[] {
   const widths: number[] = [];
   for (const row of rows) {
      row.forEach((cell, i) => {
         widths[i] = Math.max(widths[i] ?? 0, cell.length);
      });
   }
   return rows.map(row =>
      row
         .map((cell, i) => {
            const w = widths[i] ?? 0;
            if (right[i]) return cell.padStart(w);
            return i === row.length - 1 ? cell : cell.padEnd(w);
         })
         .join('  ')
         .trimEnd()
   );
}

/** A meter cell or spans as text. */
function cellText(value: Json | undefined): string {
   if (isJsonObject(value) && isJsonObject(value['meter'])) {
      const meter = value['meter'];
      const v = numProp(meter, 'value');
      let sum = 0;
      for (const part of list(meter, 'parts')) sum += numProp(part, 'value') ?? 0;
      return bar(v ?? (meter['parts'] === undefined ? undefined : sum), 8);
   }
   return spansText(value);
}

/** Renders trees into lines. */
class Plain {
   readonly cols: number;

   constructor(cols: number) {
      this.cols = cols;
   }

   /** The lines of a list of trees, one after another. */
   all(trees: readonly Tree[]): string[] {
      return trees.flatMap(tree => this.lines(tree));
   }

   /** One line of a tree, for rows. */
   inline(tree: Tree): string {
      return this.lines(tree)
         .map(line => line.trim())
         .filter(line => line !== '')
         .join(' ');
   }

   /** A head line over an indented body. */
   headed(head: string, body: readonly string[]): string[] {
      if (head === '') return [...body];
      return [head, ...body.map(line => (line === '' ? '' : INDENT + line))];
   }

   lines(tree: Tree): string[] {
      const p = tree.props;
      if (p['hidden'] === true) return [];
      switch (tree.kind) {
         case 'col':
         case 'list':
         case 'overlay':
            return this.headed(prop(p, 'head'), this.all(tree.children));
         case 'row':
            return [
               tree.children
                  .map(child => this.inline(child))
                  .filter(Boolean)
                  .join(' '),
            ];
         case 'status':
            return [
               tree.children
                  .map(child => this.inline(child))
                  .filter(Boolean)
                  .join(' | '),
            ];
         case 'card':
         case 'section': {
            const status = prop(p, 'status');
            const head = prop(p, 'head') + (status === '' ? '' : ` (${status})`);
            return this.headed(head, this.all(tree.children));
         }
         case 'rule': {
            const label = prop(p, 'label');
            const width = Math.min(this.cols, 40);
            if (label === '') return ['-'.repeat(width)];
            const side = Math.max(2, Math.floor((width - label.length - 2) / 2));
            return [`${'-'.repeat(side)} ${label} ${'-'.repeat(side)}`];
         }
         case 'spacer':
            return [''];
         case 'text':
         case 'shimmer':
         case 'seg': {
            const own = p['spans'] === undefined ? prop(p, 'text') : prop(p, 'spans');
            const kids = tree.children.map(child => this.inline(child)).filter(Boolean);
            return [[...kids, own].filter(Boolean).join(' ')].flatMap(line => line.split('\n'));
         }
         case 'md':
         case 'code':
         case 'math':
            return prop(p, 'text').split('\n');
         case 'ansi':
            return prop(p, 'text').replace(ESCAPES, '').split(/\r?\n/);
         case 'rows': {
            const lines = p['lines'];
            return Array.isArray(lines)
               ? lines
                    .filter(line => typeof line === 'string')
                    .map(line => line.replace(ESCAPES, ''))
               : [];
         }
         case 'diff': {
            const hunks = list(p, 'hunks');
            if (hunks.length === 0) return prop(p, 'text').split('\n');
            return hunks.flatMap(hunk => {
               const lines = hunk['lines'];
               return Array.isArray(lines) ? lines.filter(line => typeof line === 'string') : [];
            });
         }
         case 'editor':
         case 'input':
            return (prop(p, 'prompt') + prop(p, 'text')).split('\n');
         case 'kv': {
            const items = list(p, 'items');
            if (p['layout'] === 'inline') {
               return [items.map(item => `${prop(item, 'k')}: ${prop(item, 'v')}`).join('  ')];
            }
            return columns(items.map(item => [prop(item, 'k'), prop(item, 'v')]));
         }
         case 'table': {
            const cols = list(p, 'cols').filter(col => typeof col['id'] === 'string');
            const right = cols.map(col => col['align'] === 'end');
            const rows: string[][] = [];
            if (cols.some(col => prop(col, 'head') !== ''))
               rows.push(cols.map(col => prop(col, 'head')));
            for (const row of list(p, 'rows')) {
               const cells = isJsonObject(row['cells']) ? row['cells'] : {};
               rows.push(cols.map(col => cellText(cells[spansText(col['id'])])));
            }
            return columns(rows, right);
         }
         case 'tree': {
            const out: string[] = [];
            const walk = (items: JsonObject[], depth: number): void => {
               for (const item of items) {
                  out.push(`${INDENT.repeat(depth)}- ${prop(item, 'label')}`);
                  walk(list(item, 'children'), depth + 1);
               }
            };
            walk(list(p, 'nodes'), 0);
            return out;
         }
         case 'badge':
            return [`[${prop(p, 'text')}]`];
         case 'kbd': {
            const keys = p['keys'];
            return [Array.isArray(keys) ? keys.filter(k => typeof k === 'string').join('+') : ''];
         }
         case 'icon':
            return [];
         case 'image':
            return [`[${prop(p, 'alt') || 'image'}]`];
         case 'item': {
            const parts = [prop(p, 'label'), prop(p, 'detail'), prop(p, 'value')].filter(Boolean);
            return [`- ${parts.join('  ')}`];
         }
         case 'tabs': {
            const active = p['active'];
            return [
               list(p, 'items')
                  .map(tab =>
                     tab['id'] === active ? `[${prop(tab, 'label')}]` : prop(tab, 'label')
                  )
                  .join(' | '),
            ];
         }
         case 'picker': {
            const title = [prop(p, 'title'), prop(p, 'subtitle')].filter(Boolean).join(' ');
            const items = list(p, 'items').map(
               item => `- ${prop(item, 'label') || prop(item, 'id')}`
            );
            return this.headed(title, [...items, ...this.all(tree.children)]);
         }
         case 'spinner':
            return [prop(p, 'label')];
         case 'elapsed': {
            const ms = numProp(p, 'stopped') ?? numProp(p, 'age') ?? 0;
            return [duration(ms, prop(p, 'format'))];
         }
         case 'rate':
            return [[String(numProp(p, 'value') ?? 0), prop(p, 'unit')].filter(Boolean).join(' ')];
         case 'progress':
            return [[bar(numProp(p, 'value')), prop(p, 'label')].filter(Boolean).join(' ')];
         case 'meter': {
            let value = numProp(p, 'value');
            const parts = list(p, 'parts');
            if (parts.length > 0)
               value = parts.reduce((sum, part) => sum + (numProp(part, 'value') ?? 0), 0);
            return [[bar(value), prop(p, 'label'), prop(p, 'total')].filter(Boolean).join(' ')];
         }
         case 'chart': {
            const summary = prop(p, 'summary');
            if (summary !== '') return [summary];
            return [
               list(p, 'series')
                  .map(point => prop(point, 'label') || String(numProp(point, 'value') ?? ''))
                  .join(' '),
            ];
         }
         case 'effort':
            return [prop(p, 'level')];
         case 'toast':
            return [[prop(p, 'text'), prop(p, 'sub')].filter(Boolean).join(' - ')];
         case 'tool': {
            const status = prop(p, 'status');
            const meta = Array.isArray(p['meta']) ? p['meta'].map(spansText) : [];
            const head = [prop(p, 'title') || prop(p, 'name'), prop(p, 'target'), ...meta]
               .filter(Boolean)
               .join(' ');
            return this.headed(
               head + (status === '' ? '' : ` (${status})`),
               this.all(tree.children)
            );
         }
         case 'agent': {
            const status = prop(p, 'status');
            const head = [prop(p, 'name'), prop(p, 'task')].filter(Boolean).join(': ');
            return this.headed(
               head + (status === '' ? '' : ` (${status})`),
               this.all(tree.children)
            );
         }
         case 'checklist':
            return list(p, 'phases').flatMap(phase =>
               this.headed(
                  prop(phase, 'title'),
                  list(phase, 'items').map(item => {
                     return `[${CHECK_MARKS[prop(item, 'status')] ?? ' '}] ${prop(item, 'text')}`;
                  })
               )
            );
         case 'prefs':
            return this.headed(
               prop(p, 'title') || 'Settings',
               list(p, 'sections').flatMap(section =>
                  this.headed(
                     prop(section, 'title'),
                     list(section, 'rows').map(row => {
                        const control = isJsonObject(row['control']) ? row['control'] : {};
                        const value = control['on'] ?? control['value'] ?? control['values'];
                        const shown = Array.isArray(value)
                           ? value.join(', ')
                           : spansText(value ?? '');
                        const label = prop(row, 'label') || prop(row, 'id');
                        return shown === '' ? label : `${label}: ${shown}`;
                     })
                  )
               )
            );
         case 'el':
            return this.el(tree);
         default:
            return this.all(tree.children);
      }
   }

   /** An `el` subtree: inline tags on one line, block tags on lines of their own. */
   el(tree: Tree): string[] {
      const out: string[] = [];
      let line = '';
      const end = (): void => {
         if (line.trim() !== '') out.push(line.trimEnd());
         line = '';
      };
      const visit = (node: Tree, depth: number, marker: string): void => {
         if (node.kind !== 'el') {
            end();
            out.push(...this.lines(node));
            return;
         }
         const tag = prop(node.props, 'tag') || 'div';
         if (tag === 'table') {
            end();
            const rows: string[][] = [];
            const walk = (row: Tree): void => {
               if (prop(row.props, 'tag') === 'tr')
                  rows.push(row.children.map(cell => this.inline(cell)));
               else row.children.forEach(walk);
            };
            node.children.forEach(walk);
            out.push(...columns(rows));
            return;
         }
         if (tag === 'hr') {
            end();
            out.push('-'.repeat(Math.min(this.cols, 40)));
            return;
         }
         const block = BLOCK_TAGS[tag] === true;
         if (block) end();
         let text = prop(node.props, 'text');
         if (tag === 'input') {
            const checked = node.props['checked'] === true;
            text =
               prop(node.props, 'type') === 'radio'
                  ? checked
                     ? '(*)'
                     : '( )'
                  : checked
                    ? '[x]'
                    : '[ ]';
         }
         if (tag === 'li') line += INDENT.repeat(Math.max(0, depth - 1)) + marker;
         if (tag === 'button') line += `[ ${text}`;
         else if (text !== '') line += (line === '' || line.endsWith(' ') ? '' : ' ') + text;
         const listy = tag === 'ul' || tag === 'ol';
         node.children.forEach((child, i) => {
            const mark = tag === 'ol' ? `${i + 1}. ` : tag === 'ul' ? '- ' : marker;
            visit(child, listy ? depth + 1 : depth, mark);
         });
         if (tag === 'button') line += ' ]';
         if (block) end();
      };
      visit(tree, 0, '- ');
      end();
      return out;
   }
}

/**
 * A view as readable plain text without escapes: text kinds as their text,
 * rows on one line, cards as a head over an indented body, tables and `kv`
 * as aligned columns, lists as bullets, progress as `[####----] 50%`.
 */
export function plain(view: Renderable, cols = 80): string {
   const renderer = new Plain(cols);
   const regions = View.from(view).regions.filter((tree): tree is Tree => tree !== undefined);
   return regions
      .flatMap(region => renderer.all(region.children))
      .join('\n')
      .replace(/[ \t]+$/gm, '');
}
