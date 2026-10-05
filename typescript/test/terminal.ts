/** An in-memory terminal that answers the handshake as Tern (or not) and records output. */
import { EventEmitter } from 'node:events';
import {
   concatBytes,
   isJsonObject,
   splitMessage,
   type JsonObject,
   type Param,
   type TermInput,
   type TermOutput,
} from '../src/index.js';

const encoder = new TextEncoder();
const decoder = new TextDecoder();

/** How the scripted terminal answers `hello`. */
export type Script = 'tern' | 'da1' | 'silent';

/** One program → terminal message, chunks joined. */
export interface Sent {
   readonly verb: string;
   readonly params: readonly Param[];
   readonly body: string;
   readonly json: JsonObject | undefined;
}

/** The tty's input side: the test types into it. */
export class FakeInput extends EventEmitter implements TermInput {
   readonly isTTY = true;
   isRaw = false;
   readonly raw: boolean[] = [];

   setRawMode(mode: boolean): void {
      this.isRaw = mode;
      this.raw.push(mode);
   }

   /** Sends bytes as Tern or the keyboard would. */
   type(text: string): void {
      this.emit('data', encoder.encode(text));
   }
}

/** The tty's output side: keeps every byte written. */
export class FakeOutput implements TermOutput {
   readonly isTTY = true;
   readonly columns = 100;
   readonly chunks: Uint8Array[] = [];
   onWrite: ((text: string) => void) | undefined;

   write(data: Uint8Array | string): boolean {
      const bytes = typeof data === 'string' ? encoder.encode(data) : data;
      this.chunks.push(bytes);
      this.onWrite?.(decoder.decode(bytes));
      return true;
   }

   /** Everything written, as text. */
   text(): string {
      return decoder.decode(concatBytes(this.chunks));
   }

   /** The TSP messages written, chunked ones joined. */
   messages(): Sent[] {
      const bytes = concatBytes(this.chunks);
      const out: Sent[] = [];
      let joining: { verb: string; params: Param[]; body: Uint8Array[] } | undefined;
      let at = 0;
      for (;;) {
         const start = bytes.indexOf(0x1b, at);
         if (start < 0) break;
         if (bytes[start + 1] !== 0x5f) {
            at = start + 1;
            continue;
         }
         let end = start + 2;
         while (end < bytes.length && !(bytes[end] === 0x1b && bytes[end + 1] === 0x5c)) end++;
         at = end + 2;
         const inner = bytes.subarray(start + 2, end);
         if (decoder.decode(inner.subarray(0, 4)) !== 'tsp;') continue;
         const split = splitMessage(inner.subarray(4));
         if (!split) continue;
         const chunk = split.params.find(([key]) => key === 'c');
         const more = split.params.some(([key, value]) => key === 'm' && value === '1');
         const params = split.params.filter(([key]) => key !== 'c' && key !== 'm');
         if (chunk) {
            joining ??= { verb: split.verb, params, body: [] };
            joining.body.push(split.body);
            if (more) continue;
            out.push(sent(joining.verb, joining.params, concatBytes(joining.body)));
            joining = undefined;
            continue;
         }
         out.push(sent(split.verb, params, split.body));
      }
      return out;
   }

   /** The JSON bodies of the messages of `verb`. */
   bodies(verb: string): JsonObject[] {
      return this.messages()
         .filter(message => message.verb === verb)
         .flatMap(message => (message.json ? [message.json] : []));
   }
}

function sent(verb: string, params: readonly Param[], body: Uint8Array): Sent {
   const text = decoder.decode(body);
   let json: JsonObject | undefined;
   try {
      const parsed: unknown = JSON.parse(text);
      json = isJsonObject(parsed) ? parsed : undefined;
   } catch {
      json = undefined;
   }
   return { verb, params, body: text, json };
}

/** The `hello` reply Tern sends. */
export const HELLO = {
   r: 'hello',
   v: 1,
   term: 'tern',
   ver: '0.4.3',
   kinds: ['col', 'text', 'el'],
   features: ['flow', 'styles'],
   apc: 65536,
   credits: 2,
   cols: 120,
   cell: { w: 8, h: 17 },
   dark: true,
   reduceMotion: false,
};

/** A TSP message from the terminal. */
export function tsp(verb: 'r' | 'e', body: unknown): string {
   return `\x1b_tsp;${verb};${JSON.stringify(body)}\x1b\\`;
}

/** A pty with a scripted terminal on the other end. */
export class Terminal {
   readonly input = new FakeInput();
   readonly output = new FakeOutput();

   constructor(script: Script = 'tern', hello: object = HELLO) {
      this.output.onWrite = text => {
         if (!text.includes('\x1b[c')) return;
         setTimeout(() => {
            if (script === 'tern') this.input.type(tsp('r', hello) + '\x1b[?62;52;c');
            if (script === 'da1') this.input.type('\x1b[?62;c');
         }, 1);
      };
   }

   /** Sends an event. */
   event(body: object): void {
      this.input.type(tsp('e', body));
   }

   /** Lets timers and promise jobs run. */
   async settle(ms = 5): Promise<void> {
      const { promise, resolve } = Promise.withResolvers<void>();
      setTimeout(resolve, ms);
      await promise;
   }
}
