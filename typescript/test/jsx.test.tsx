import { describe, expect, test } from 'bun:test';
import { html, node, plain, span, ui, View, type Renderable } from '../src/index.js';

/** A node (or span) as JSON without ids. */
function json(value: unknown): unknown {
   return JSON.parse(JSON.stringify(value));
}

/** The `add` op of a first render: the main region with ids. */
function added(view: Renderable): unknown {
   const ops = new View().ops(View.from(view), 's1');
   return JSON.parse(JSON.stringify(ops[0]?.[4]));
}

describe('TSX', () => {
   test('keyed fragment components retain their children and text parts', () => {
      const Row = () => (
         <>
            <text key="label">Hello</text>
            <rule key="rule" />
         </>
      );
      const Text = () => (
         <>
            Hello <span s="accent">world</span>
         </>
      );
      expect(
         json(
            <col>
               <Row key="row" />
            </col>
         )
      ).toEqual({
         k: 'col',
         c: [
            { k: 'text', p: { key: 'label', text: 'Hello' } },
            { k: 'rule', p: { key: 'rule' } },
         ],
      });
      expect(
         json(
            <text>
               <Text key="text" />
            </text>
         )
      ).toEqual({
         k: 'text',
         p: { spans: ['Hello ', { t: 'world', s: 'accent' }] },
      });
      expect(View.from(<Row key="row" />).find('main.label')?.kind).toBe('text');
   });

   test('json carries its value and open levels and prints pretty', () => {
      const n = ui.json({ value: { a: 1 }, open: 1 });
      expect(json(n)).toEqual({ k: 'json', p: { value: { a: 1 }, open: 1 } });
      expect(plain(n)).toBe('{\n  "a": 1\n}');
   });

   test('kind-specific props, agent statuses and picker reasons survive the builders', () => {
      const title = [span('Styled', 'accent')];
      expect(ui.tool({ title }).props['title']).toEqual([{ t: 'Styled', s: 'accent' }]);
      expect(ui.picker({ title, actions: [{ id: 'go', disabled: 'Offline' }] }).props).toEqual({
         title: [{ t: 'Styled', s: 'accent' }],
         actions: [{ id: 'go', disabled: 'Offline' }],
      });
      expect(ui.list({ max: 5 }).props['max']).toBe(5);
      expect(ui.prefs({ title: 'Settings' }).props['title']).toBe('Settings');
      for (const status of [
         'pending',
         'running',
         'done',
         'failed',
         'aborted',
         'idle',
         'parked',
      ] as const)
         expect(ui.agent({ status }).props['status']).toBe(status);
   });

   test('code bars, folds and their hover cards go on the wire as given', () => {
      const n = ui.code({
         text: 'a\nb\nc',
         nums: [1, 2, 5],
         marks: [{ line: 1, heat: 0.5, wash: [0.9, 0.3] }],
         folds: [
            {
               from: 2,
               to: 3,
               label: '… [2 lines]',
               tone: 'error',
               hovercard: ui.text({ text: 'cut' }),
            },
         ],
         overview: true,
      });
      expect(json(n)).toEqual({
         k: 'code',
         p: {
            text: 'a\nb\nc',
            nums: [1, 2, 5],
            marks: [{ line: 1, heat: 0.5, wash: [0.9, 0.3] }],
            folds: [
               {
                  from: 2,
                  to: 3,
                  label: '… [2 lines]',
                  tone: 'error',
                  hovercard: { k: 'text', p: { text: 'cut' } },
               },
            ],
            overview: true,
         },
      });
   });

   test('meter cells carry the required meter wrapper on the wire', () => {
      const cells = { usage: { meter: { value: 0.5, title: 'Half' } } };
      const table = ui.table({ cols: [{ id: 'usage' }], rows: [{ id: 'a', cells }] });
      expect(table.props['rows']).toEqual([{ id: 'a', cells }]);
      expect(plain(table)).toBe('[####----] 50%');
      expect(
         plain(
            ui.table({
               cols: [{ id: 'usage' }],
               rows: [
                  {
                     id: 'a',
                     cells: { usage: { meter: { parts: [{ value: 0.25 }, { value: 0.5 }] } } },
                  },
               ],
            })
         )
      ).toBe('[######--] 75%');
   });
   test('intrinsic kinds take typed props; string children become text', () => {
      const src = 'fn main() {}';
      const tree = (
         <card head="Deploy" status="running" collapsible>
            <md>{'**bold**'}</md>
            <code lang="rust">{src}</code>
            <badge tone="accent">v2</badge>
         </card>
      );
      expect(json(tree)).toEqual({
         k: 'card',
         p: { head: 'Deploy', status: 'running', collapsible: true },
         c: [
            { k: 'md', p: { text: '**bold**' } },
            { k: 'code', p: { lang: 'rust', text: src } },
            { k: 'badge', p: { tone: 'accent', text: 'v2' } },
         ],
      });
   });

   test('<span> inside <text> builds styled spans', () => {
      const tree = (
         <text>
            plain <span s="accent">bold</span>
            <span s="muted" fx="pulse" href="https://x.dev">
               link
            </span>
         </text>
      );
      expect(json(tree)).toEqual({
         k: 'text',
         p: {
            spans: [
               'plain ',
               { t: 'bold', s: 'accent' },
               { t: 'link', s: 'muted', fx: 'pulse', href: 'https://x.dev' },
            ],
         },
      });
   });

   test('html elements, handlers and derived ids make the protocol book form', () => {
      const submit = (): void => {};
      const size = (key: string, label: string) => (
         <html.label key={key}>
            <html.input key="r" type="radio" name="size" value={key} />
            <html.span key="t">{label}</html.span>
         </html.label>
      );
      const view = (
         <html.form key="ask" class="ask">
            <html.p key="q">Which size?</html.p>
            <html.div key="sizes" class="sizes">
               {size('s', 'Small')}
               {size('m', 'Medium')}
            </html.div>
            <html.button key="go" actions={{ click: 'submit' }} onClick={submit}>
               Create
            </html.button>
         </html.form>
      );
      expect(added(view)).toEqual({
         id: 'main',
         k: 'col',
         c: [
            {
               id: 'main.ask',
               k: 'el',
               p: { tag: 'form', key: 'ask', class: 'ask' },
               c: [
                  { id: 'main.ask.q', k: 'el', p: { tag: 'p', key: 'q', text: 'Which size?' } },
                  {
                     id: 'main.ask.sizes',
                     k: 'el',
                     p: { tag: 'div', key: 'sizes', class: 'sizes' },
                     c: ['s', 'm'].map(k => ({
                        id: `main.ask.sizes.${k}`,
                        k: 'el',
                        p: { tag: 'label', key: k },
                        c: [
                           {
                              id: `main.ask.sizes.${k}.r`,
                              k: 'el',
                              p: { tag: 'input', key: 'r', type: 'radio', name: 'size', value: k },
                           },
                           {
                              id: `main.ask.sizes.${k}.t`,
                              k: 'el',
                              p: { tag: 'span', key: 't', text: k === 's' ? 'Small' : 'Medium' },
                           },
                        ],
                     })),
                  },
                  {
                     id: 'main.ask.go',
                     k: 'el',
                     p: { tag: 'button', key: 'go', actions: { click: 'submit' }, text: 'Create' },
                  },
               ],
            },
         ],
      });
   });

   test('click, double-click and menu handlers name their actions', () => {
      const fn = (): void => {};
      const tree = (
         <text onClick={fn} onDblClick={fn} onMenu={{ rerun: fn }} actions={{ menu: ['copy'] }} />
      );
      expect(json(tree)).toEqual({
         k: 'text',
         p: { actions: { menu: ['copy', 'rerun'], click: 'click', dblclick: 'dblclick' } },
      });
   });

   test('components, fragments, arrays and skipped children', () => {
      const Row = (props: { readonly name: string }) => (
         <row>
            <badge>{props.name}</badge>
         </row>
      );
      const names = ['a', 'b'];
      const flag = false;
      const tree = (
         <col gap="sm">
            <>
               {names.map(name => (
                  <Row key={name} name={name} />
               ))}
            </>
            {null}
            {undefined}
            {flag && <rule />}
            {'loose text'}
         </col>
      );
      expect(json(tree)).toEqual({
         k: 'col',
         p: { gap: 'sm' },
         c: [
            { k: 'row', p: { key: 'a' }, c: [{ k: 'badge', p: { text: 'a' } }] },
            { k: 'row', p: { key: 'b' }, c: [{ k: 'badge', p: { text: 'b' } }] },
            { k: 'text', p: { text: 'loose text' } },
         ],
      });
   });

   test('TSX and the ui builders build the same nodes', () => {
      const fromTsx = (
         <section head={[span('Logs', 'strong')]}>
            <progress value={0.5} label="half" />
            <spinner style="dots" label="Waiting" tone="pending" />
         </section>
      );
      const fromUi = ui.section(
         { head: [span('Logs', 'strong')] },
         ui.progress({ value: 0.5, label: 'half' }),
         ui.spinner({ style: 'dots', label: 'Waiting', tone: 'pending' })
      );
      expect(json(fromTsx)).toEqual(json(fromUi));
      expect(json(ui.md('# hi'))).toEqual({ k: 'md', p: { text: '# hi' } });
   });

   test('a fragment renders into main', () => {
      const ops = new View().ops(
         View.from(
            <>
               <md>a</md>
               <rule />
            </>
         ),
         's1'
      );
      expect(JSON.parse(JSON.stringify(ops))).toEqual([
         [
            'add',
            'main',
            's1',
            null,
            {
               id: 'main',
               k: 'col',
               c: [
                  { id: 'main.0', k: 'md', p: { text: 'a' } },
                  { id: 'main.1', k: 'rule' },
               ],
            },
         ],
      ]);
   });

   test('regions given by name; a list is wrapped in a col', () => {
      const view = View.from({ main: <col role="chat" />, dock: [<editor placeholder="Ask" />] });
      expect(view.regions.map(region => region?.wire())).toEqual([
         { id: 'main', k: 'col', p: { role: 'chat' } },
         { id: 'dock', k: 'col', c: [{ id: 'dock.0', k: 'editor', p: { placeholder: 'Ask' } }] },
         undefined,
      ]);
   });

   test('text kinds refuse node children', () => {
      expect(() => node('md', null, ui.rule())).toThrow(TypeError);
   });
});

describe('plain', () => {
   test('el children dispatch native kinds through the fallback renderer', () => {
      expect(
         plain(
            <html.div>
               <progress value={0.5} />
               <spinner label="Working" />
            </html.div>
         )
      ).toBe('[##########----------] 50%\nWorking');
   });
   test('cards, rows, kv, tables, progress and lists read as text', () => {
      const view = (
         <>
            <card head="Deploy" status="done">
               <kv
                  items={[
                     { k: 'service', v: 'api' },
                     { k: 'region', v: 'eu-west-1' },
                  ]}
               />
               <progress value={0.5} label="uploading" />
            </card>
            <row>
               <badge>v2</badge>
               <text>ready</text>
            </row>
            <table
               cols={[
                  { id: 'n', head: 'Name' },
                  { id: 's', head: 'Size', align: 'end' },
               ]}
               rows={[
                  { id: 'a', cells: { n: 'Cargo.toml', s: '2.1K' } },
                  { id: 'b', cells: { n: 'build.rs', s: '312' } },
               ]}
            />
            <list>
               <item label="one" detail="first" />
            </list>
            <spinner label="Waiting" />
            <elapsed age={65000} />
         </>
      );
      expect(plain(view)).toBe(
         [
            'Deploy (done)',
            '  service  api',
            '  region   eu-west-1',
            '  [##########----------] 50% uploading',
            '[v2] ready',
            'Name        Size',
            'Cargo.toml  2.1K',
            'build.rs     312',
            '- one  first',
            'Waiting',
            '1m 05s',
         ].join('\n')
      );
   });

   test('el block tags take lines of their own, inline tags join them', () => {
      const view = (
         <html.form>
            <html.p>
               Which <html.strong>size</html.strong>?
            </html.p>
            <html.label>
               <html.input type="radio" checked />
               <html.span>Large</html.span>
            </html.label>
            <html.ul>
               <html.li>one</html.li>
               <html.li>two</html.li>
            </html.ul>
            <html.button>Create</html.button>
         </html.form>
      );
      expect(plain(view)).toBe(
         ['Which size ?', '(*) Large', '- one', '- two', '[ Create ]'].join('\n')
      );
   });

   test('ansi escapes are stripped', () => {
      expect(plain([ui.ansi('\x1b[31mred\x1b[0m\r\nnext')])).toBe('red\nnext');
   });
});

describe('panels, buttons and rich tables', () => {
   test('slot nodes become children keyed by slot, sent as their ids', () => {
      const view = (
         <card head={<text>Jobs</text>} meta="3 running" end={<button>Refresh</button>}>
            <text>body</text>
         </card>
      );
      expect(added([view])).toEqual({
         id: 'main',
         k: 'col',
         c: [
            {
               id: 'main.0',
               k: 'card',
               p: { meta: '3 running', head: 'main.0.head', end: 'main.0.end' },
               c: [
                  { id: 'main.0.0', k: 'text', p: { text: 'body' } },
                  { id: 'main.0.head', k: 'text', p: { text: 'Jobs', key: 'head' } },
                  { id: 'main.0.end', k: 'button', p: { text: 'Refresh', key: 'end' } },
               ],
            },
         ],
      });
      expect(plain([view])).toBe('Jobs  3 running  [Refresh]\n  body');
      const overlay = ui.overlay({ head: 'Delete?', foot: ui.button('Cancel') }, ui.text('Sure'));
      expect(plain([overlay])).toBe('Delete?\n  Sure\n  [Cancel]');
   });

   test('a button with onClick asks for the click action', () => {
      expect(json(<button onClick={() => {}}>Go</button>)).toEqual({
         k: 'button',
         p: { text: 'Go', actions: { click: 'click' } },
      });
   });

   test('stats and rich table cells have plain fallbacks', () => {
      expect(
         plain([
            ui.stats({
               items: [
                  { label: 'Builds', value: '42', sub: '+3' },
                  { label: 'Failed', value: '2' },
               ],
            }),
         ])
      ).toBe('Builds: 42 (+3)  ·  Failed: 2');
      const table = ui.table({
         cols: [{ id: 's' }, { id: 'b' }, { id: 'x' }, { id: 'k' }],
         rows: [
            {
               id: 'r',
               cells: {
                  s: { text: 'ok', dot: 'success' },
                  b: { badge: { text: 'new' } },
                  x: { button: { text: 'Approve', act: 'approve' } },
                  k: { spark: { values: [0, 7] } },
               },
            },
         ],
      });
      expect(plain([table])).toBe('● ok  new  [Approve]  ▁█');
      expect(plain([ui.table({ cols: [{ id: 'a', head: 'A' }], empty: 'No jobs' })])).toBe(
         'A\nNo jobs'
      );
   });
});

describe('controls, chat and work kinds', () => {
   test('new kinds and extended props go on the wire as typed', () => {
      expect(
         json(
            <select
               label="Model"
               value="opus"
               options={[
                  'haiku',
                  { value: 'opus', label: 'Opus 4', detail: '200k', group: 'Claude' },
               ]}
               variant="quiet"
               search
               query="op"
            />
         )
      ).toEqual({
         k: 'select',
         p: {
            label: 'Model',
            value: 'opus',
            options: ['haiku', { value: 'opus', label: 'Opus 4', detail: '200k', group: 'Claude' }],
            variant: 'quiet',
            search: true,
            query: 'op',
         },
      });
      expect(json(<slider value={[2, 8]} min={0} max={10} ticks={[0, 5, 10]} />)).toEqual({
         k: 'slider',
         p: { value: [2, 8], min: 0, max: 10, ticks: [0, 5, 10] },
      });
      // @ts-expect-error: a switch is on or off
      expect(json(<switch on="yes" />)).toEqual({ k: 'switch', p: { on: 'yes' } });
      expect(
         json(
            <menu
               text="Sort"
               items={[
                  { header: 'Sort by' },
                  { id: 'name', radio: 'sort', checked: true, keys: ['cmd', 'N'] },
                  '-',
                  { id: 'export', items: [{ id: 'csv', disabled: 'Soon' }] },
               ]}
            />
         )
      ).toEqual({
         k: 'menu',
         p: {
            text: 'Sort',
            items: [
               { header: 'Sort by' },
               { id: 'name', radio: 'sort', checked: true, keys: ['cmd', 'N'] },
               '-',
               { id: 'export', items: [{ id: 'csv', disabled: 'Soon' }] },
            ],
         },
      });
      expect(
         ui.table({
            cols: [{ id: 'name', sortable: true }],
            sort: { col: 'name', dir: 'desc' },
            multi: true,
            picked: ['a'],
            variant: 'bar',
            bar: [{ act: 'restart', text: 'Restart', variant: 'primary' }],
            hint: 'r restarts',
         }).props
      ).toEqual({
         cols: [{ id: 'name', sortable: true }],
         sort: { col: 'name', dir: 'desc' },
         multi: true,
         picked: ['a'],
         variant: 'bar',
         bar: [{ act: 'restart', text: 'Restart', variant: 'primary' }],
         hint: 'r restarts',
      });
      expect(
         ui.tool({
            status: 'ask',
            confirm: 'scopes',
            ask: { text: 'Push?', always: false, scopes: [{ id: 'once' }], scope: 'once' },
            approved: 'session',
         }).props
      ).toEqual({
         status: 'ask',
         confirm: 'scopes',
         ask: { text: 'Push?', always: false, scopes: [{ id: 'once' }], scope: 'once' },
         approved: 'session',
      });
      expect(ui.overlay({ side: 'bottom', variant: 'inset', size: 0.4 }).props).toEqual({
         side: 'bottom',
         variant: 'inset',
         size: 0.4,
      });
      expect(
         ui.input({
            text: 'abc',
            icon: 'search',
            keys: ['cmd', 'f'],
            clear: true,
            prefix: 'https://',
            buttons: [{ act: 'copy', icon: 'copy', title: 'Copy' }],
         }).props
      ).toEqual({
         text: 'abc',
         icon: 'search',
         keys: ['cmd', 'f'],
         clear: true,
         prefix: 'https://',
         buttons: [{ act: 'copy', icon: 'copy', title: 'Copy' }],
      });
      expect(
         ui.chart({
            kind: 'area',
            series: [
               { label: 'p50', values: [1, null, { value: 3, title: 'peak' }] },
               { label: 'p95', token: 'warning', values: [2, 4, 6] },
            ],
            labels: ['a', '', 'c'],
            unit: 'ms',
         }).props['series']
      ).toEqual([
         { label: 'p50', values: [1, null, { value: 3, title: 'peak' }] },
         { label: 'p95', token: 'warning', values: [2, 4, 6] },
      ]);
   });

   test('rule ends and message avatars are slots', () => {
      expect(
         added([
            ui.message(
               { author: 'Ana', avatar: ui.avatar({ text: 'Ana', status: 'success' }) },
               'Hi'
            ),
            ui.rule({ variant: 'pin', label: 'Restored', end: ui.button('Undo') }),
         ])
      ).toEqual({
         id: 'main',
         k: 'col',
         c: [
            {
               id: 'main.0',
               k: 'message',
               p: { author: 'Ana', avatar: 'main.0.avatar' },
               c: [
                  { id: 'main.0.0', k: 'text', p: { text: 'Hi' } },
                  {
                     id: 'main.0.avatar',
                     k: 'avatar',
                     p: { text: 'Ana', status: 'success', key: 'avatar' },
                  },
               ],
            },
            {
               id: 'main.1',
               k: 'rule',
               p: { variant: 'pin', label: 'Restored', end: 'main.1.end' },
               c: [{ id: 'main.1.end', k: 'button', p: { text: 'Undo', key: 'end' } }],
            },
         ],
      });
   });

   test('rich actions.menu rows keep their shape and onMenu adds only missing ids', () => {
      const n = ui.text(
         {
            actions: { menu: [{ id: 'open', icon: 'file', keys: 'enter' }, '-'] },
            onMenu: { open: () => {}, trash: () => {} },
         },
         'deploy.log'
      );
      expect(n.props['actions']).toEqual({
         menu: [{ id: 'open', icon: 'file', keys: 'enter' }, '-', 'trash'],
      });
   });

   test('cited spans go on the wire and print numbered markers', () => {
      const pnpm = { href: 'https://pnpm.io/cli/install', title: 'pnpm install' };
      const n = (
         <text cite="note">
            <span cite={pnpm}>The lockfile drifted</span> after the bump.
         </text>
      );
      expect(json(n)).toEqual({
         k: 'text',
         p: {
            cite: 'note',
            spans: [{ t: 'The lockfile drifted', cite: pnpm }, ' after the bump.'],
         },
      });
      expect(plain(n)).toBe('The lockfile drifted[1] after the bump.');
      const numbered = ui.text({
         spans: [
            span('A', '', { cite: [{ href: 'h1' }, { href: 'h2', n: 1 }] }),
            ' and ',
            span('B', '', { cite: { href: 'h1' } }),
         ],
      });
      expect(plain(numbered)).toBe('A[2][1] and B[2]');
   });

   test('new kinds have plain fallbacks', () => {
      const cases: [Renderable, string][] = [
         [
            ui.select({
               label: 'Model',
               value: 'opus',
               options: ['haiku', { value: 'opus', label: 'Opus 4' }],
            }),
            'Model: Opus 4',
         ],
         [ui.select({ placeholder: 'Pick one' }), 'Pick one'],
         [ui.switch({ on: true, label: 'Wrap' }), '[x] Wrap'],
         [ui.slider({ label: 'Opacity', value: 72, unit: '%' }), 'Opacity: 72%'],
         [ui.slider({ value: [2, 8], unit: 'px' }), '2–8 px'],
         [
            ui.field(
               { label: 'Name', required: true, error: 'Required', hint: 'Your name' },
               ui.input({ text: 'x' })
            ),
            'Name *\nx\nRequired',
         ],
         [ui.otp({ text: '12', length: 4 }), '12__'],
         [ui.otp({ text: '12', length: 4, mask: true }), '••__'],
         [ui.buttons(ui.button('Undo'), ui.button('Redo')), '[Undo] [Redo]'],
         [ui.avatar({ text: 'Ana' }), 'Ana'],
         [
            ui.avatars({ items: [{ text: 'Ana' }, { text: 'Ben' }], label: 'reviewing' }),
            'Ana, Ben reviewing',
         ],
         [ui.breadcrumb({ items: [{ id: 'home', label: 'Home' }, { id: 'docs' }] }), 'Home › docs'],
         [ui.breadcrumb({ variant: 'path', items: [{ id: 'src' }, { id: 7 }] }), 'src / 7'],
         [ui.pager({ page: 2, total: 312, size: 50 }), 'Page 2 of 7'],
         [ui.pager({ page: 3 }), 'Page 3'],
         [
            ui.empty({ title: 'No jobs', text: 'Start one' }, ui.button('New')),
            'No jobs\nStart one\n[New]',
         ],
         [ui.skeleton({ shape: 'rows' }), ''],
         [
            ui.calendar({ range: true, value: ['2026-10-19', '2026-10-30'] }),
            '2026-10-19 – 2026-10-30',
         ],
         [ui.date({ range: true, placeholder: 'Pick dates' }), 'Pick dates'],
         [ui.calendar({ range: true, value: ['2026-10-19', null] }), '2026-10-19'],
         [ui.menu({ text: 'Sort', items: ['name'] }), '[Sort]'],
         [
            ui.menubar({
               menus: [
                  { id: 'file', label: 'File', items: [] },
                  { id: 'edit', items: [] },
               ],
            }),
            'File  edit',
         ],
         [
            ui.nav({
               active: 'b',
               items: [
                  { id: 'a', label: 'Home' },
                  { id: 'more', items: [{ id: 'b' }] },
               ],
            }),
            'Home | [more]',
         ],
         [ui.split(ui.text('left'), ui.text('right')), 'left\nright'],
         [ui.scroll({ stick: 'end' }, ui.text('log')), 'log'],
         [ui.carousel({ index: 1 }, ui.text('one'), ui.text('two')), 'two'],
         [
            ui.message(
               {
                  author: 'Ana',
                  time: '14:02',
                  avatar: ui.avatar({ text: 'Ana' }),
                  reactions: [{ text: '👍', count: 2 }, { text: '🎉' }],
                  status: 'Read 14:03',
               },
               'Ship it'
            ),
            'Ana 14:02\n  Ship it\n  👍 2  🎉\n  Read 14:03',
         ],
         [ui.attachment({ name: 'report.pdf', state: 'uploading' }), '[attachment: report.pdf]'],
         [
            ui.trace(
               {
                  title: 'Fixing',
                  status: 'running',
                  lines: [
                     { id: 'diff', title: 'Ran', target: 'git diff', status: 'done' },
                     { title: 'Searching', status: 'running' },
                  ],
               },
               ui.text('Lockfile out of date'),
               ui.text({ key: 'diff' }, '2 files changed')
            ),
            [
               'Fixing (running)',
               '- Ran git diff (done)',
               '    2 files changed',
               '- Searching (running)',
               'Lockfile out of date',
            ].join('\n'),
         ],
         [
            ui.stacktrace({
               error: { type: 'TypeError', message: 'x is undefined' },
               frames: [{ fn: 'run', file: 'src/a.ts', line: 3, col: 7 }, { app: false }],
            }),
            'TypeError: x is undefined\n  at run (src/a.ts:3:7)\n  at <anonymous>',
         ],
         [ui.suggestions({ items: [{ text: 'Explain' }, { text: 'Fix' }] }), '- Explain\n- Fix'],
         [
            ui.questions({
               items: [
                  {
                     id: 'size',
                     prompt: 'Which size?',
                     choices: ['S', { value: 'l', label: 'Large' }],
                  },
               ],
            }),
            'Which size?\n  - S\n  - Large',
         ],
         [ui.input({ text: 'hunter2', secret: true }), '•••••••'],
         [
            ui.rule({ label: 'Checkpoint', time: '14:02' }),
            '---------- Checkpoint · 14:02 ----------',
         ],
         [ui.chart({ kind: 'line', series: [0, null, 7] }), '▁█'],
         [
            ui.chart({ kind: 'donut', unit: '%', series: [{ value: 60, label: 'Hit' }, 40] }),
            'Hit 60%  40%',
         ],
      ];
      for (const [view, text] of cases) expect(plain(view)).toBe(text);
   });
});
