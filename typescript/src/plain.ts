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

/** Slider units written against their number (`72%`); any other follows a space (`13 px`). */
const ATTACHED_UNITS = ['%', '‰', '°', '×', 'x', '′', '″', "'", '"'];

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

/** The eighths blocks a sparkline is drawn with, low to high. */
const SPARKS = '▁▂▃▄▅▆▇█';

/** Values as a block sparkline, scaled to `min`..`max` (default: their own range). */
export function sparkline(values: readonly number[], min?: number, max?: number): string {
   if (values.length === 0) return '';
   const lo = min ?? Math.min(...values);
   const hi = max ?? Math.max(...values);
   const span = hi - lo;
   let out = '';
   for (const v of values) {
      const at = span > 0 ? Math.round(((v - lo) / span) * (SPARKS.length - 1)) : 0;
      out += SPARKS[Math.min(SPARKS.length - 1, Math.max(0, at))];
   }
   return out;
}

/** A table cell as text: meters as bars, sparks as blocks, badges and buttons bracketed. */
function cellText(value: Json | undefined): string {
   if (!isJsonObject(value)) return spansText(value);
   const meter = value['meter'];
   if (isJsonObject(meter)) {
      const v = numProp(meter, 'value');
      let sum = 0;
      for (const part of list(meter, 'parts')) sum += numProp(part, 'value') ?? 0;
      return bar(v ?? (meter['parts'] === undefined ? undefined : sum), 8);
   }
   const badge = value['badge'];
   if (isJsonObject(badge)) return prop(badge, 'text');
   const button = value['button'];
   if (isJsonObject(button)) return `[${prop(button, 'text')}]`;
   const spark = value['spark'];
   if (isJsonObject(spark)) {
      const raw = spark['values'];
      const values = Array.isArray(raw) ? raw.filter(v => typeof v === 'number') : [];
      return sparkline(values, numProp(spark, 'min'), numProp(spark, 'max'));
   }
   const text = prop(value, 'text');
   return value['dot'] === undefined ? text : `● ${text}`;
}

/** The plain text of spans, `[n]` after each cited run (numbered as Tern numbers a text's sources). */
function citedText(value: Json | undefined): string {
   if (!Array.isArray(value)) return spansText(value);
   const cites = (part: Json): JsonObject[] => {
      if (!isJsonObject(part)) return [];
      const cite = part['cite'];
      const all: readonly unknown[] = Array.isArray(cite) ? cite : [cite];
      return all.filter(isJsonObject).filter(c => typeof c['href'] === 'string');
   };
   const numbers = new Map<string, number>();
   for (const part of value) {
      for (const c of cites(part)) {
         const n = numProp(c, 'n');
         const href = prop(c, 'href');
         if (n !== undefined && n >= 1 && !numbers.has(href)) numbers.set(href, Math.floor(n));
      }
   }
   const taken = new Set(numbers.values());
   let next = 1;
   let out = '';
   for (const part of value) {
      out += spansText([part]);
      const marks: number[] = [];
      for (const c of cites(part)) {
         const href = prop(c, 'href');
         let n = numbers.get(href);
         if (n === undefined) {
            while (taken.has(next)) next++;
            n = next;
            taken.add(n);
            numbers.set(href, n);
         }
         if (!marks.includes(n)) marks.push(n);
      }
      out += marks.map(n => `[${n}]`).join('');
   }
   return out;
}

/** A `calendar` or `date` value: a day, or a range's ends. */
function dateText(value: Json | undefined): string {
   if (typeof value === 'string') return value;
   if (!Array.isArray(value)) return '';
   const [from, to] = value;
   const start = typeof from === 'string' ? from : '';
   return typeof to === 'string' ? `${start} – ${to}` : start;
}

/** The numbers of a chart's series entries (bare or `{ value }`), gaps dropped. */
function chartValues(entries: Json | undefined): number[] {
   if (!Array.isArray(entries)) return [];
   const out: number[] = [];
   for (const entry of entries) {
      const value = isJsonObject(entry) ? entry['value'] : entry;
      if (typeof value === 'number') out.push(value);
   }
   return out;
}

/** A panel's slot props (`head`, `end`, `foot`, …) that name a child, split from its body. */
function slots(
   tree: Tree,
   names: readonly string[] = ['head', 'end', 'foot']
): { slot: Record<string, Tree>; body: Tree[] } {
   const slot: Record<string, Tree> = {};
   const body: Tree[] = [];
   for (const child of tree.children) {
      let name: string | undefined;
      for (const key of names) if (tree.props[key] === child.id) name = key;
      if (name === undefined) body.push(child);
      else slot[name] = child;
   }
   return { slot, body };
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
            return this.headed(prop(p, 'head'), this.all(tree.children));
         case 'overlay':
         case 'card':
         case 'section': {
            const { slot, body } = slots(tree);
            const status = prop(p, 'status');
            const head = [
               slot['head'] ? this.inline(slot['head']) : prop(p, 'head'),
               prop(p, 'meta'),
            ]
               .filter(Boolean)
               .join('  ');
            const end = slot['end'] ? this.inline(slot['end']) : '';
            const state = status === '' ? '' : `(${status})`;
            const titled = [[head, state].filter(Boolean).join(' '), end]
               .filter(Boolean)
               .join('  ');
            const foot = slot['foot'] ? this.lines(slot['foot']) : [];
            return this.headed(titled, [...this.all(body), ...foot]);
         }
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
         case 'rule': {
            const label = [prop(p, 'label'), prop(p, 'time')].filter(Boolean).join(' · ');
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
            const own = p['spans'] === undefined ? prop(p, 'text') : citedText(p['spans']);
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
         case 'input': {
            const text = prop(p, 'text');
            const masked = p['secret'] === true;
            return (prop(p, 'prompt') + (masked ? '•'.repeat(text.length) : text)).split('\n');
         }
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
            const empty = list(p, 'rows').length === 0 ? prop(p, 'empty') : '';
            return [...columns(rows, right), ...(empty === '' ? [] : [empty])];
         }
         case 'json':
            return JSON.stringify(p['value'] ?? null, null, 2).split('\n');
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
         case 'button':
            return [`[${prop(p, 'text') || prop(p, 'title') || prop(p, 'aria')}]`];
         case 'stats':
            return [
               list(p, 'items')
                  .map(item => {
                     const sub = prop(item, 'sub');
                     const line = `${prop(item, 'label')}: ${prop(item, 'value')}`;
                     return sub === '' ? line : `${line} (${sub})`;
                  })
                  .join('  ·  '),
            ];
         case 'kbd': {
            const keys = p['keys'];
            return [Array.isArray(keys) ? keys.filter(k => typeof k === 'string').join('+') : ''];
         }
         case 'icon':
            return [];
         case 'image':
            return [`[${prop(p, 'alt') || 'image'}]`];
         case 'media': {
            const rev = prop(p, 'rev');
            const path = prop(p, 'path') || 'media';
            return [rev ? `[${path} @ ${rev}]` : `[${path}]`];
         }
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
            const kind = p['kind'];
            if (kind === 'line' || kind === 'area') {
               const series = list(p, 'series').filter(s => Array.isArray(s['values']));
               if (series.length === 0) return [sparkline(chartValues(p['series']))];
               return series.map(s =>
                  [prop(s, 'label'), sparkline(chartValues(s['values']))].filter(Boolean).join(' ')
               );
            }
            if (kind === 'donut') {
               const entries = Array.isArray(p['series']) ? p['series'] : [];
               const unit = prop(p, 'unit');
               // A currency symbol leads the number; a chart attaches only units starting
               // with `%`, `×` or `°`.
               const lead = ['$', '€', '£', '¥'].includes(unit);
               const sep = unit === '' || /^[%×°]/.test(unit) ? '' : ' ';
               return [
                  entries
                     .map(entry => {
                        const value = isJsonObject(entry) ? numProp(entry, 'value') : entry;
                        const label = isJsonObject(entry) ? prop(entry, 'label') : '';
                        const shown =
                           typeof value !== 'number'
                              ? ''
                              : lead
                                ? `${value < 0 ? '−' : ''}${unit}${Math.abs(value)}`
                                : `${value}${sep}${unit}`;
                        return [label, shown].filter(Boolean).join(' ');
                     })
                     .filter(Boolean)
                     .join('  '),
               ];
            }
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
         case 'select': {
            const value = p['value'];
            const options = Array.isArray(p['options']) ? p['options'] : [];
            const option = options.find(
               o => o === value || (isJsonObject(o) && o['value'] === value)
            );
            const shown =
               value === undefined
                  ? prop(p, 'placeholder')
                  : isJsonObject(option)
                    ? prop(option, 'label') || prop(option, 'value')
                    : spansText(value);
            const label = prop(p, 'label');
            return [label === '' ? shown : `${label}: ${shown}`];
         }
         case 'switch':
            return [`[${p['on'] === true ? 'x' : ' '}] ${prop(p, 'label')}`];
         case 'slider': {
            const value = p['value'];
            const label = prop(p, 'label');
            if (value === undefined) return [label];
            const shown = Array.isArray(value)
               ? value.filter(v => typeof v === 'number').join('–')
               : typeof value === 'number'
                 ? String(value)
                 : spansText(value);
            const unit = prop(p, 'unit');
            const sep = unit === '' || ATTACHED_UNITS.includes(unit) ? '' : ' ';
            const text = `${shown}${sep}${unit}`;
            return [label === '' ? text : `${label}: ${text}`];
         }
         case 'field': {
            const label = prop(p, 'label');
            const head = label !== '' && p['required'] === true ? `${label} *` : label;
            // An error wins over a warning, and either hides the hint; `true` sets no line.
            const flagged = [p['error'], p['warning']].find(
               v => v !== undefined && v !== null && v !== false && v !== ''
            );
            const note =
               flagged === undefined ? prop(p, 'hint') : flagged === true ? '' : spansText(flagged);
            return [
               ...(head === '' ? [] : [head]),
               ...this.all(tree.children),
               ...(note === '' ? [] : [note]),
            ];
         }
         case 'otp': {
            const length = Math.min(12, Math.max(1, numProp(p, 'length') ?? 6));
            const typed = Array.from(prop(p, 'text')).slice(0, length);
            const mask = p['mask'];
            const dot =
               mask === true ? '•' : typeof mask === 'string' ? Array.from(mask)[0] : undefined;
            const cells = dot === undefined ? typed : typed.map(() => dot);
            return [cells.join('').padEnd(length, '_')];
         }
         case 'buttons':
            return [
               tree.children
                  .map(child => this.inline(child))
                  .filter(Boolean)
                  .join(' '),
            ];
         case 'avatar':
            return [prop(p, 'text')];
         case 'avatars': {
            const names = list(p, 'items')
               .map(item => prop(item, 'text'))
               .filter(Boolean)
               .join(', ');
            return [[names, prop(p, 'label')].filter(Boolean).join(' ')];
         }
         case 'breadcrumb': {
            const crumbs = list(p, 'items').filter(
               item => typeof item['id'] === 'string' || typeof item['id'] === 'number'
            );
            return [
               crumbs
                  .map(item => prop(item, 'label') || String(item['id']))
                  .join(p['variant'] === 'path' ? ' / ' : ' › '),
            ];
         }
         case 'pager': {
            const total = numProp(p, 'total');
            const size = numProp(p, 'size');
            const pages =
               numProp(p, 'pages') ||
               (total !== undefined && size ? Math.max(1, Math.ceil(total / size)) : undefined);
            const page = Math.max(1, Math.min(numProp(p, 'page') ?? 1, pages ?? Infinity));
            return [pages === undefined ? `Page ${page}` : `Page ${page} of ${pages}`];
         }
         case 'empty':
            return [prop(p, 'title'), prop(p, 'text'), ...this.all(tree.children)].filter(
               line => line !== ''
            );
         case 'skeleton':
            return [];
         case 'calendar':
            return p['value'] === undefined ? [] : [dateText(p['value'])];
         case 'date': {
            if (p['value'] !== undefined) return [dateText(p['value'])];
            const placeholder = prop(p, 'placeholder');
            return placeholder === '' ? [] : [placeholder];
         }
         case 'menu':
            return [`[${prop(p, 'text') || prop(p, 'title')}]`];
         case 'menubar':
            return [
               list(p, 'menus')
                  .filter(menu => typeof menu['id'] === 'string')
                  .map(menu => prop(menu, 'label') || prop(menu, 'id'))
                  .join('  '),
            ];
         case 'nav': {
            const active = p['active'];
            return [
               list(p, 'items')
                  .filter(item => typeof item['id'] === 'string')
                  .map(item => {
                     const label = prop(item, 'label') || prop(item, 'id');
                     const subs = list(item, 'items');
                     const on = item['id'] === active || subs.some(sub => sub['id'] === active);
                     return on ? `[${label}]` : label;
                  })
                  .join(' | '),
            ];
         }
         case 'carousel': {
            const last = tree.children.length - 1;
            const slide = tree.children[Math.max(0, Math.min(numProp(p, 'index') ?? 0, last))];
            return slide ? this.lines(slide) : [];
         }
         case 'message': {
            const { body } = slots(tree, ['avatar']);
            const head = [prop(p, 'author'), prop(p, 'time')].filter(Boolean).join(' ');
            const reactions = list(p, 'reactions')
               .filter(r => prop(r, 'text') !== '')
               .map(r => {
                  const count = numProp(r, 'count') ?? 0;
                  return count > 0 ? `${prop(r, 'text')} ${count}` : prop(r, 'text');
               })
               .join('  ');
            const status = prop(p, 'status');
            const rest = [
               ...this.all(body),
               ...(reactions === '' ? [] : [reactions]),
               ...(status === '' ? [] : [status]),
            ];
            return [
               ...(head === '' ? [] : [head]),
               ...rest.map(line => (line === '' ? '' : INDENT + line)),
            ];
         }
         case 'attachment':
            return [`[attachment: ${prop(p, 'name')}]`];
         case 'trace': {
            const status = prop(p, 'status');
            const title = prop(p, 'title') || prop(p, 'summary');
            const head = [title, status === '' ? '' : `(${status})`].filter(Boolean).join(' ');
            // A child keyed by a line's id (default its index) is that line's content.
            const placed = new Set<Tree>();
            const lines = list(p, 'lines').flatMap((line, i) => {
               const id = prop(line, 'id') || String(i);
               const state = prop(line, 'status');
               const text = [prop(line, 'title'), prop(line, 'target')].filter(Boolean).join(' ');
               const content = tree.children.find(child => String(child.props['key']) === id);
               if (content) placed.add(content);
               const under = content ? this.lines(content) : [];
               return [
                  `- ${text}${state === '' ? '' : ` (${state})`}`,
                  ...under.map(l => (l === '' ? '' : INDENT.repeat(2) + l)),
               ];
            });
            const body = this.all(tree.children.filter(child => !placed.has(child)));
            return [...(head === '' ? [] : [head]), ...lines, ...body];
         }
         case 'stacktrace': {
            const error = isJsonObject(p['error']) ? p['error'] : {};
            const head = [prop(error, 'type'), prop(error, 'message')].filter(Boolean).join(': ');
            const frames = list(p, 'frames').map(frame => {
               const fn = prop(frame, 'fn') || '<anonymous>';
               const file = prop(frame, 'file');
               if (file === '') return `at ${fn}`;
               const at = [file, numProp(frame, 'line'), numProp(frame, 'col')];
               return `at ${fn} (${at.filter(v => v !== undefined).join(':')})`;
            });
            return [...(head === '' ? [] : [head]), ...frames.map(frame => INDENT + frame)];
         }
         case 'suggestions':
            return list(p, 'items')
               .filter(item => prop(item, 'text') !== '')
               .map(item => `- ${prop(item, 'text')}`);
         case 'questions':
            return list(p, 'items')
               .filter(item => typeof item['id'] === 'string')
               .flatMap(item => {
                  const choices = Array.isArray(item['choices']) ? item['choices'] : [];
                  return this.headed(
                     prop(item, 'prompt'),
                     choices.map(choice =>
                        isJsonObject(choice)
                           ? `- ${prop(choice, 'label') || prop(choice, 'value')}`
                           : `- ${spansText(choice)}`
                     )
                  );
               });
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
