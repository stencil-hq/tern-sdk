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
