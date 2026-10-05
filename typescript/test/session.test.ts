import { describe, expect, spyOn, test } from 'bun:test';
import { mkdtempSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import {
   ask,
   connect,
   html,
   print,
   ui,
   type ActionEvent,
   type ConnectOptions,
   type Session,
   type SessionInput,
} from '../src/index.js';
import { HELLO, Terminal, tsp, type Script } from './terminal.js';

/** Connects over a scripted terminal. */
async function open(
   script: Script = 'tern',
   options: ConnectOptions = {},
   hello: object = HELLO
): Promise<{ term: Terminal; session: Session | null }> {
   const term = new Terminal(script, hello);
   const session = await connect({
      app: 'test',
      env: {},
      input: term.input,
      output: term.output,
      ...options,
   });
   return { term, session };
}

/** Connects and expects a session. */
async function connected(options: ConnectOptions = {}, hello: object = HELLO) {
   const { term, session } = await open('tern', options, hello);
   if (!session) throw new Error('no session');
   return { term, session };
}

describe('connect', () => {
   test('handshake writes and partial raw/mode setup always roll back', async () => {
      for (const stage of ['raw', 'hello', 'da1', 'modes']) {
         const term = new Terminal();
         const originalWrite = term.output.write.bind(term.output);
         const originalRaw = term.input.setRawMode.bind(term.input);
         const before = process.listenerCount('SIGTERM');
         term.input.setRawMode = mode => {
            if (mode) expect(process.listenerCount('SIGTERM')).toBe(before + 1);
            originalRaw(mode);
            if (mode && stage === 'raw') throw new Error(stage);
         };
         term.output.write = data => {
            const text = typeof data === 'string' ? data : new TextDecoder().decode(data);
            if (
               (stage === 'hello' && text.includes('tsp;q;')) ||
               (stage === 'da1' && text === '\x1b[c') ||
               (stage === 'modes' && text.includes('2004h'))
            )
               throw new Error(stage);
            return originalWrite(data);
         };
         await expect(
            connect({ env: {}, input: term.input, output: term.output, exitHooks: true })
         ).rejects.toThrow(stage);
         expect(term.input.raw).toEqual([true, false]);
         expect(term.input.listenerCount('data')).toBe(0);
         expect(process.listenerCount('SIGTERM')).toBe(before);
         if (stage === 'modes') expect(term.output.text()).toContain('\x1b[<u\x1b[?2004l');
      }
   });

   test('a second session cannot acquire an owned tty', async () => {
      const { term, session } = await connected();
      await expect(connect({ env: {}, input: term.input, output: term.output })).rejects.toThrow(
         'already has a session'
      );
      expect(term.input.raw).toEqual([true]);
      await session.close();
      const again = await connect({ env: {}, input: term.input, output: term.output });
      expect(again).not.toBeNull();
      await again?.close();
   });

   test('handshake decides in arrival order and preserves trailing input', async () => {
      for (const helloFirst of [true, false]) {
         const term = new Terminal('silent');
         term.output.onWrite = text => {
            if (text !== '\x1b[c') return;
            term.input.type(
               helloFirst
                  ? tsp('r', HELLO) + '\x1b[?62;c' + tsp('e', { ev: 'theme', dark: false }) + '😀'
                  : '\x1b[?62;c' + tsp('r', HELLO)
            );
         };
         const session = await connect({ env: {}, input: term.input, output: term.output });
         if (!helloFirst) {
            expect(session).toBeNull();
            continue;
         }
         expect(session?.caps.dark).toBe(false);
         expect((await session?.next())?.type).toBe('event');
         const input = await session?.next();
         expect(input?.type === 'key' && input.key.text).toBe('😀');
         await session?.close();
      }
   });
   test('a hello reply before DA1 connects, with its capabilities', async () => {
      const { term, session } = await connected({ version: '1.2.3', features: ['edit'] });
      expect(session.caps.cols).toBe(120);
      expect(session.caps.features).toEqual(['flow', 'styles']);
      const [hello] = term.output.bodies('q');
      expect(hello).toEqual({ q: 'hello', v: [1], app: 'test', ver: '1.2.3', features: ['edit'] });
      expect(term.input.raw).toEqual([true]);
      const text = term.output.text();
      expect(text.indexOf('\x1b_tsp;q;')).toBeLessThan(text.indexOf('\x1b[c'));
      expect(text.endsWith('\x1b[?2004h\x1b[>1u')).toBe(true);
      await session.close();
   });

   test('DA1 answering first means no TSP, and the tty is restored', async () => {
      const { term, session } = await open('da1');
      expect(session).toBeNull();
      expect(term.input.raw).toEqual([true, false]);
      expect(term.input.listenerCount('data')).toBe(0);
      expect(term.output.text()).not.toContain('2004h');
   });

   test('no answer within the timeout means no TSP', async () => {
      const started = Date.now();
      const { term, session } = await open('silent', { timeout: 30 });
      expect(session).toBeNull();
      expect(Date.now() - started).toBeGreaterThanOrEqual(25);
      expect(term.input.raw).toEqual([true, false]);
   });

   test('TERN_TSP=0 and multiplexers skip the probe', async () => {
      for (const env of [{ TERN_TSP: '0' }, { TMUX: '/tmp/tmux' }, { STY: 's' }, { ZELLIJ: '0' }]) {
         const { term, session } = await open('tern', { env });
         expect(session).toBeNull();
         expect(term.output.chunks).toHaveLength(0);
         expect(term.input.raw).toEqual([]);
      }
   });

   test('modes can be left off', async () => {
      const { term, session } = await connected({ bracketedPaste: false, kittyKeyboard: false });
      await session.close();
      expect(term.output.text()).not.toContain('2004');
      expect(term.output.text()).not.toContain('>1u');
   });
});

describe('surfaces', () => {
   test('adopt retains main and sequence but drops dock and layer', async () => {
      const { term, session } = await connected();
      const first = session.open({ id: 'saved' });
      first.render({ main: [ui.md('a')], dock: [ui.md('dock')], layer: [ui.md('layer')] });
      await first.close();
      const adopted = session.open({ id: 'saved', adopt: true });
      adopted.render({ main: [ui.md('ab')], dock: [ui.md('new')] });
      const frame = term.output.bodies('f')[1];
      expect(frame?.['s']).toBe(2);
      expect(frame?.['ops']).toEqual([
         ['text', 'main.0', 'append', 'b'],
         [
            'add',
            'dock',
            'saved',
            null,
            { id: 'dock', k: 'col', c: [{ id: 'dock.0', k: 'md', p: { text: 'new' } }] },
         ],
      ]);
      const unknown = session.open({ id: 'unknown', adopt: true });
      unknown.render([ui.md('fresh')]);
      const unknownOps = term.output.bodies('f')[2]?.['ops'];
      expect(Array.isArray(unknownOps) && unknownOps[0]).toEqual(['del', 'main']);
      await session.close();
   });

   test('failed frames retain sequence, sent view and queued ops for retry', async () => {
      const { term, session } = await connected({}, { ...HELLO, credits: 1 });
      const surface = session.open();
      surface.render([ui.md('a')]);
      surface.render([ui.md('ab')]);
      surface.focus('main.0');
      const write = term.output.write.bind(term.output);
      term.output.write = () => {
         throw new Error('frame failed');
      };
      term.event({ ev: 'ack', sf: 's1', s: 1 });
      await expect(session.next()).rejects.toThrow('frame failed');
      expect(surface.seq).toBe(1);
      expect(surface.blocked).toBe(false);
      term.output.write = write;
      surface.render([ui.md('abc')]);
      expect(term.output.bodies('f')[1]).toEqual({
         sf: 's1',
         s: 2,
         ops: [
            ['text', 'main.0', 'append', 'bc'],
            ['focus', 'main.0'],
         ],
      });
      term.event({ ev: 'ack', sf: 's1', s: 2 });
      term.output.write = () => {
         throw new Error('direct failed');
      };
      expect(() => surface.render([ui.md('abcd')])).toThrow('direct failed');
      expect(surface.seq).toBe(2);
      term.output.write = write;
      surface.render([ui.md('abcde')]);
      expect(term.output.bodies('f')[2]).toEqual({
         sf: 's1',
         s: 3,
         ops: [['text', 'main.0', 'append', 'de']],
      });
      await session.close();
   });
   test('frames number from 1 and the first view adds main', async () => {
      const { term, session } = await connected();
      const surface = session.open({ mode: 'flow', title: 'Build' });
      surface.render([ui.text('a')]);
      expect(term.output.bodies('o')).toEqual([{ id: 's1', mode: 'flow', title: 'Build' }]);
      expect(term.output.bodies('f')).toEqual([
         {
            sf: 's1',
            s: 1,
            ops: [
               [
                  'add',
                  'main',
                  's1',
                  null,
                  { id: 'main', k: 'col', c: [{ id: 'main.0', k: 'text', p: { text: 'a' } }] },
               ],
            ],
         },
      ]);
      await session.close();
   });

   test('blocked renders coalesce into one frame when an ack returns credit', async () => {
      const { term, session } = await connected();
      const surface = session.open();
      const view = (text: string) => [ui.md(text)];
      surface.render(view('a'));
      surface.render(view('ab'));
      surface.render(view('abc'));
      surface.render(view('abcd'));
      surface.focus('main.0');
      expect(term.output.bodies('f').map(f => f['s'])).toEqual([1, 2]);
      expect(surface.blocked).toBe(true);
      term.event({ ev: 'ack', sf: 's1', s: 1 });
      await term.settle();
      const frames = term.output.bodies('f');
      expect(frames.map(f => f['s'])).toEqual([1, 2, 3]);
      expect(frames[2]?.['ops']).toEqual([
         ['text', 'main.0', 'append', 'cd'],
         ['focus', 'main.0'],
      ]);
      term.event({ ev: 'ack', sf: 's1', s: 3 });
      await term.settle();
      expect(term.output.bodies('f')).toHaveLength(3);
      surface.render(view('abcd'));
      expect(term.output.bodies('f')).toHaveLength(3);
      await session.close();
   });

   test('an ack covers every frame up to it', async () => {
      const { term, session } = await connected({}, { ...HELLO, credits: 1 });
      const surface = session.open();
      surface.render([ui.md('1')]);
      surface.render([ui.md('12')]);
      expect(term.output.bodies('f')).toHaveLength(1);
      term.event({ ev: 'ack', sf: 's1', s: 1 });
      await term.settle();
      expect(term.output.bodies('f')).toHaveLength(2);
      await session.close();
   });

   test('listen:false surfaces send at once', async () => {
      const { term, session } = await connected();
      const surface = session.open({ mode: 'flow', listen: false });
      for (const text of ['a', 'ab', 'abc', 'abcd']) surface.render([ui.md(text)]);
      expect(term.output.bodies('o')).toEqual([{ id: 's1', mode: 'flow', listen: false }]);
      expect(term.output.bodies('f').map(f => f['s'])).toEqual([1, 2, 3, 4]);
      await session.close();
   });

   test('a view with two siblings of one id is rejected whole', async () => {
      const { term, session } = await connected();
      const surface = session.open();
      expect(() => surface.render([ui.md({ key: 'a' }, 'x'), ui.md({ key: 'a' }, 'y')])).toThrow();
      expect(term.output.bodies('f')).toHaveLength(0);
      await session.close();
   });

   test('stylesheets and palettes go to the surface', async () => {
      const { term, session } = await connected();
      const surface = session.open({ id: 'pick', mode: 'flow' });
      surface.stylesheet('main', '.a{color:red}');
      surface.stylesheet('main', null);
      surface.palette({ dark: { accent: '#00b4ff' } });
      expect(term.output.bodies('s')).toEqual([
         { sf: 'pick', name: 'main', css: '.a{color:red}' },
         { sf: 'pick', name: 'main' },
      ]);
      expect(term.output.bodies('t')).toEqual([{ sf: 'pick', dark: { accent: '#00b4ff' } }]);
      await session.close();
   });

   test('a gone for the surface closes it', async () => {
      const { term, session } = await connected();
      const surface = session.open();
      term.event({ ev: 'gone', sf: 's1', ids: ['s1'] });
      await term.settle();
      expect(surface.closed).toBe(true);
      surface.render([ui.md('x')]);
      expect(term.output.bodies('f')).toHaveLength(0);
      const input = await session.next();
      expect(input?.type === 'event' && input.event.ev).toBe('gone');
      await session.close();
      expect(term.output.bodies('x')).toHaveLength(0);
   });

   test('resize, theme and motion keep the capabilities current', async () => {
      const { term, session } = await connected();
      term.event({ ev: 'resize', sf: 's1', cols: 90, cell: { w: 9, h: 18 } });
      term.event({ ev: 'theme', dark: false });
      term.event({ ev: 'motion', reduce: true });
      await term.settle();
      expect(session.caps.cols).toBe(90);
      expect(session.caps.cell).toEqual({ w: 9, h: 18 });
      expect(session.caps.dark).toBe(false);
      expect(session.caps.reduceMotion).toBe(true);
      await session.close();
   });

   test('hour12 follows the hello reply and defaults to 24-hour', async () => {
      const twelve = await connected({}, { ...HELLO, hour12: true });
      expect(twelve.session.caps.hour12).toBe(true);
      await twelve.session.close();
      const older = await connected();
      expect(older.session.caps.hour12).toBe(false);
      await older.session.close();
   });
});

describe('input', () => {
   test('named click and double-click match act plus value', async () => {
      const { term, session } = await connected();
      const calls: string[] = [];
      session.open().render([
         ui.text(
            {
               actions: { click: 'sort=name', dblclick: 'open=detail' },
               onClick: () => {
                  calls.push('click');
               },
               onDblClick: () => {
                  calls.push('double');
               },
            },
            'item'
         ),
      ]);
      term.event({ ev: 'action', sf: 's1', id: 'main.0', act: 'sort', value: 'name' });
      term.event({ ev: 'action', sf: 's1', id: 'main.0', act: 'open', value: 'detail' });
      term.input.type('z');
      const input = await session.next();
      expect(input?.type === 'key' && input.key.name).toBe('z');
      expect(calls).toEqual(['click', 'double']);
      await session.close();
   });

   test('idle flushing waits for silence and preserves paste terminators', async () => {
      const { term, session } = await connected();
      term.input.type('\x1b');
      await term.settle(15);
      term.input.type('[1;');
      await term.settle(20);
      term.input.type('5A');
      const input = await session.next();
      expect(input?.type === 'key' && input.key.name).toBe('up');
      expect(input?.type === 'key' && input.key.ctrl).toBe(true);
      term.input.type('\x1b[200~😀\x1b[20');
      await term.settle(40);
      term.input.type('1~z');
      const paste = await session.next();
      expect(paste?.type === 'key' && paste.key.text).toBe('😀');
      const after = await session.next();
      expect(after?.type === 'key' && after.key.name).toBe('z');
      await session.close();
   });
   test('handlers take their events; keys and other events reach the loop in order', async () => {
      const { term, session } = await connected();
      const surface = session.open({ mode: 'flow' });
      const clicks: ActionEvent[] = [];
      const menus: string[] = [];
      surface.render([
         html.button(
            {
               key: 'go',
               onClick: event => {
                  clicks.push(event);
               },
               onMenu: {
                  rerun: () => {
                     menus.push('rerun');
                  },
               },
            },
            'Go'
         ),
         html.button({ key: 'other', actions: { click: 'other' } }, 'Other'),
      ]);
      term.input.type('a');
      term.event({ ev: 'action', sf: 's1', id: 'main.go', act: 'click', values: { size: 'l' } });
      term.event({ ev: 'action', sf: 's1', id: 'main.go', act: 'rerun' });
      term.event({ ev: 'action', sf: 's1', id: 'main.other', act: 'other' });
      term.input.type('b');
      const seen: SessionInput[] = [];
      for await (const input of session) {
         seen.push(input);
         if (seen.length === 3) break;
      }
      expect(clicks.map(c => c.values)).toEqual([{ size: 'l' }]);
      expect(menus).toEqual(['rerun']);
      expect(seen.map(i => (i.type === 'key' ? i.key.name : i.event.ev))).toEqual([
         'a',
         'action',
         'b',
      ]);
      await session.close();
   });

   test('an event for another surface or an unknown id is not taken', async () => {
      const { term, session } = await connected();
      const surface = session.open();
      surface.render([ui.text({ onToggle: () => {} }, 'x')]);
      term.event({ ev: 'toggle', sf: 's9', id: 'main.0', collapsed: true });
      term.event({ ev: 'toggle', sf: 's1', id: 'main.7', collapsed: true });
      const first = await session.next();
      const second = await session.next();
      expect([first?.type, second?.type]).toEqual(['event', 'event']);
      await session.close();
   });

   test('a handler error rejects the next read', async () => {
      const { term, session } = await connected();
      const surface = session.open();
      surface.render([
         html.button(
            {
               onClick: () => {
                  throw new Error('boom');
               },
            },
            'Go'
         ),
      ]);
      term.event({ ev: 'action', sf: 's1', id: 'main.0', act: 'click' });
      await term.settle();
      await expect(session.next()).rejects.toThrow('boom');
      await session.close();
   });

   test('a lone ESC becomes Escape after the idle flush', async () => {
      const { term, session } = await connected();
      term.input.type('\x1b');
      const input = await session.next();
      expect(input?.type === 'key' && input.key.name).toBe('escape');
      await session.close();
   });

   test('the DA1 answer after the reply is swallowed', async () => {
      const { term, session } = await connected();
      await term.settle();
      term.input.type('z');
      const input = await session.next();
      expect(input?.type === 'key' && input.key.name).toBe('z');
      await session.close();
   });
});

describe('close', () => {
   test('close tries x and every reset after output failures, drains and restores', async () => {
      const { term, session } = await connected({}, { ...HELLO, credits: 1 });
      const surface = session.open();
      surface.render([ui.md('a')]);
      surface.render([ui.md('pending')]);
      const attempts: string[] = [];
      term.output.write = data => {
         attempts.push(typeof data === 'string' ? data : new TextDecoder().decode(data));
         throw new Error('write failed');
      };
      const start = Date.now();
      await expect(session.close()).rejects.toThrow('write failed');
      expect(Date.now() - start).toBeGreaterThanOrEqual(45);
      expect(attempts).toHaveLength(4);
      expect(attempts[0]).toContain('tsp;f;');
      expect(attempts[1]).toContain('tsp;x;');
      expect(attempts.slice(2)).toEqual(['\x1b[<u', '\x1b[?2004l']);
      expect(term.input.isRaw).toBe(false);
      expect(term.input.listenerCount('data')).toBe(0);
      expect(session.closed).toBe(true);
   });

   test('termination signals use normal close and drain, including during handshake', async () => {
      for (const signal of ['SIGINT', 'SIGTERM', 'SIGHUP'] as const) {
         for (const handshake of [false, true]) {
            const term = new Terminal(handshake ? 'silent' : 'tern');
            let reraised = false;
            const kill = spyOn(process, 'kill').mockImplementation((_pid, name) => {
               expect(name).toBe(signal);
               expect(term.input.isRaw).toBe(false);
               expect(term.input.listenerCount('data')).toBe(0);
               reraised = true;
               return true;
            });
            try {
               const pending = connect({
                  env: {},
                  input: term.input,
                  output: term.output,
                  exitHooks: true,
               });
               const session = handshake ? null : await pending;
               session?.open();
               process.emit(signal, signal);
               expect(term.input.isRaw).toBe(true);
               await term.settle(20);
               expect(reraised).toBe(false);
               await term.settle(45);
               expect(reraised).toBe(true);
               if (handshake) expect(await pending).toBeNull();
               else expect(term.output.bodies('x')).toEqual([{ id: 's1', keep: true }]);
            } finally {
               kill.mockRestore();
            }
         }
      }
   });
   test('closes surfaces with keep, undoes the modes, drains, restores the tty', async () => {
      const { term, session } = await connected();
      session.open({ mode: 'flow' });
      const other = session.open({ mode: 'inline' });
      await other.close({ keep: false });
      const pending = session.next();
      const closing = session.close();
      term.input.type(tsp('e', { ev: 'ack', sf: 's1', s: 1 }) + 'late');
      await closing;
      expect(await pending).toBeNull();
      expect(await session.next()).toBeNull();
      expect(term.output.bodies('x')).toEqual([
         { id: 's2', keep: false },
         { id: 's1', keep: true },
      ]);
      const text = term.output.text();
      expect(text.endsWith('\x1b[<u\x1b[?2004l')).toBe(true);
      expect(term.input.raw).toEqual([true, false]);
      expect(term.input.listenerCount('data')).toBe(0);
      expect(session.closed).toBe(true);
   });

   test('a pending view goes as a last frame before x', async () => {
      const { term, session } = await connected({}, { ...HELLO, credits: 1 });
      const surface = session.open({ mode: 'flow' });
      surface.render([ui.md('a')]);
      surface.render([ui.md('done')]);
      await surface.close();
      expect(term.output.messages().map(m => m.verb)).toEqual(['q', 'o', 'f', 'f', 'x']);
      await session.close();
   });

   test('await using closes the session', async () => {
      const term = new Terminal();
      {
         await using session = await connect({ env: {}, input: term.input, output: term.output });
         session?.open();
      }
      expect(term.output.bodies('x')).toEqual([{ id: 's1', keep: true }]);
      expect(term.input.raw).toEqual([true, false]);
   });
});

describe('blobs and recording', () => {
   test('a blob is sent once per session; blobs asks what Tern holds', async () => {
      const dir = mkdtempSync(join(tmpdir(), 'tern-sdk-'));
      const record = join(dir, 'rec.jsonl');
      const { term, session } = await connected({ record });
      const bytes = new TextEncoder().encode('hello');
      const id = session.blob(bytes, 'text/plain');
      expect(session.blob(bytes, 'text/plain')).toBe(id);
      const blobs = term.output.messages().filter(m => m.verb === 'b');
      expect(blobs).toHaveLength(1);
      expect(blobs[0]?.params).toEqual([
         ['id', id],
         ['mime', 'text/plain'],
      ]);
      const have = session.blobs([id, 'ff']);
      term.input.type(tsp('r', { r: 'blobs', have: [id] }));
      expect(await have).toEqual([id]);
      await session.close();
      const lines = readFileSync(record, 'utf8')
         .trim()
         .split('\n')
         .map(line => JSON.parse(line));
      expect(lines.map(l => `${l.dir} ${l.verb}`)).toEqual([
         'out q',
         'in r',
         'out b',
         'out q',
         'in r',
      ]);
      expect(lines[2].params).toEqual({ id, mime: 'text/plain' });
      expect(typeof lines[0].t).toBe('number');
   });
});

describe('helpers', () => {
   test('ask returns the first handled submit after its handler runs', async () => {
      for (const actionHandler of [false, true]) {
         const term = new Terminal();
         const calls: string[] = [];
         const handler = async (): Promise<void> => {
            await term.settle();
            calls.push('handled');
         };
         const answer = ask(
            [
               html.button(
                  {
                     actions: { click: 'submit' },
                     ...(actionHandler ? { onAction: { submit: handler } } : { onClick: handler }),
                  },
                  'Submit'
               ),
            ],
            { env: {}, input: term.input, output: term.output }
         );
         await term.settle(20);
         term.event({ ev: 'action', sf: 's1', id: 'main.0', act: 'submit', values: { size: 'm' } });
         expect(await answer).toEqual({
            type: 'answer',
            id: 'main.0',
            act: 'submit',
            values: { size: 'm' },
         });
         expect(calls).toEqual(['handled']);
      }
   });
   test('print sends a listen:false flow surface, its sheet, one frame and x', async () => {
      const term = new Terminal();
      await print([ui.md('**hi**')], {
         css: '.x{}',
         env: {},
         input: term.input,
         output: term.output,
      });
      const messages = term.output.messages();
      expect(messages.map(m => m.verb)).toEqual(['q', 'o', 's', 'f', 'x']);
      expect(messages[1]?.json).toEqual({ id: 's1', mode: 'flow', listen: false });
      expect(messages[4]?.json).toEqual({ id: 's1', keep: true });
      expect(term.input.raw).toEqual([true, false]);
   });

   test('print outside Tern writes the plain text, or the fallback', async () => {
      const term = new Terminal();
      await print([ui.card({ head: 'Deploy' }, ui.md('ok'))], {
         env: { TERN_TSP: '0' },
         output: term.output,
      });
      await print([ui.md('x')], { env: { TERN_TSP: '0' }, output: term.output, fallback: 'plain' });
      expect(term.output.text()).toBe('Deploy\n  ok\nplain\n');
   });

   test('ask returns the submit action with its values', async () => {
      const term = new Terminal();
      const answer = ask(
         [html.form(html.button({ key: 'go', actions: { click: 'submit' } }, 'Create'))],
         {
            env: {},
            input: term.input,
            output: term.output,
         }
      );
      await term.settle(20);
      term.event({ ev: 'change', sf: 's1', id: 'main.0.l', value: 'l', checked: true });
      term.event({
         ev: 'action',
         sf: 's1',
         id: 'main.0.go',
         act: 'submit',
         values: { size: 'l', tags: ['a'] },
      });
      expect(await answer).toEqual({
         type: 'answer',
         id: 'main.0.go',
         act: 'submit',
         values: { size: 'l', tags: ['a'] },
      });
      expect(term.output.bodies('x')).toEqual([{ id: 's1', keep: true }]);
   });

   test('ask returns null on Escape and Ctrl+C', async () => {
      for (const key of ['\x1b[27u', '\x03']) {
         const term = new Terminal();
         const answer = ask([ui.md('?')], { env: {}, input: term.input, output: term.output });
         await term.settle(20);
         term.input.type(key);
         expect(await answer).toBeNull();
      }
   });

   test('ask without TSP is unsupported', async () => {
      expect(await ask([ui.md('?')], { env: { TERN_TSP: '0' } })).toEqual({ type: 'unsupported' });
   });
});
