/**
 * The session layer: detection and the `hello` handshake, raw mode,
 * surfaces, blobs, the input loop with event routing, recording and a clean
 * exit.
 */
import { appendFileSync } from 'node:fs';
import { basename, extname } from 'node:path';
import { InputParser, type InputItem } from './input.js';
import { KeyDecoder, type Key } from './keys.js';
import { Surface, type OpenOptions } from './surface.js';
import {
   blobId,
   DEFAULT_APC,
   DEFAULT_CREDITS,
   encodeBlob,
   encodeJson,
   KINDS,
   PROTOCOL_VERSION,
   type CellSize,
   type HelloQuery,
   type HelloReply,
   type OutMessage,
   type ProgramFeature,
   type TerminalFeature,
   type TspEvent,
} from './wire.js';

/** The tty's input side as the session reads it (`process.stdin` fits). */
export interface TermInput {
   on(event: 'data', listener: (chunk: Uint8Array | string) => void): unknown;
   off(event: 'data', listener: (chunk: Uint8Array | string) => void): unknown;
   readonly isTTY?: boolean;
   readonly isRaw?: boolean;
   setRawMode?(mode: boolean): unknown;
   resume?(): unknown;
   pause?(): unknown;
}

/** The tty's output side as the session writes it (`process.stdout` fits). */
export interface TermOutput {
   write(data: Uint8Array | string): unknown;
   readonly isTTY?: boolean;
   readonly columns?: number;
}

/** What `connect` takes. */
export interface ConnectOptions {
   /** The program's name; the executable's name by default. */
   readonly app?: string;
   /** The program's version. */
   readonly version?: string;
   /** What the program handles beyond v1. */
   readonly features?: readonly (ProgramFeature | (string & {}))[];
   /** How long to wait for the reply, in ms. Default 1000. */
   readonly timeout?: number;
   /** The tty's input; `process.stdin` by default. */
   readonly input?: TermInput;
   /** The tty's output; `process.stdout` by default. */
   readonly output?: TermOutput;
   /** The environment read by detection; `process.env` by default. */
   readonly env?: Readonly<Record<string, string | undefined>>;
   /** Enable bracketed paste while connected. Default `true`. */
   readonly bracketedPaste?: boolean;
   /** Push kitty keyboard flag 1 while connected. Default `true`. */
   readonly kittyKeyboard?: boolean;
   /** Record every message as JSONL here; `TERN_TSP_RECORD` by default. */
   readonly record?: string;
   /** Restore the tty on process exit and SIGINT/SIGTERM. Default: when using the process's own streams. */
   readonly exitHooks?: boolean;
}

/** What the terminal supports, from the `hello` reply, kept current by events. */
export interface Capabilities {
   readonly term: string;
   readonly ver: string;
   readonly kinds: readonly string[];
   readonly features: readonly TerminalFeature[];
   readonly apc: number;
   readonly credits: number;
   readonly cols: number;
   readonly cell: CellSize;
   readonly dark: boolean;
   readonly reduceMotion: boolean;
   /** The user's system reads a 12-hour clock (`3:05 PM`); false (24-hour) on terminals that don't say. */
   readonly hour12: boolean;
}

/** One thing read from the session: a key, or an event no handler took. */
export type SessionInput =
   | { readonly type: 'key'; readonly key: Key }
   | { readonly type: 'event'; readonly event: TspEvent };

const encoder = new TextEncoder();
const DA1_QUERY = '\x1b[c';
const PASTE_ON = '\x1b[?2004h';
const PASTE_OFF = '\x1b[?2004l';
const KITTY_PUSH = '\x1b[>1u';
const KITTY_POP = '\x1b[<u';
/** Input idle this long releases undecided prefixes. */
const FLUSH_MS = 30;
/** Input read and dropped after close, so late replies never reach the shell. */
const DRAIN_MS = 50;

/** Raw-mode ownership is exclusive for each input stream. */
const ownedInputs = new WeakSet<TermInput>();

/**
 * A connection to Tern over the pane's tty. Read keys and unhandled events
 * with `for await (const input of session)`; node handlers run before
 * anything is yielded. `await using session = await connect(…)` closes it.
 */
export class Session implements AsyncIterable<SessionInput>, AsyncDisposable {
   #input: TermInput;
   #output: TermOutput;
   #options: ConnectOptions;
   #parser = new InputParser();
   #keys = new KeyDecoder();
   #caps: Capabilities;
   #wasRaw = false;
   #ownsInput = false;
   #modesArmed = false;
   #attached = false;
   #handshake: ((ok: boolean) => void) | undefined;
   #flushTimer: NodeJS.Timeout | undefined;
   #surfaces = new Map<string, Surface>();
   #retired = new Map<string, Surface>();
   #surfaceCount = 0;
   #chunkCount = 0;
   #blobsSent = new Set<string>();
   #blobWaiters: ((have: string[]) => void)[] = [];
   #queue: SessionInput[] = [];
   #waiters: { resolve: (input: SessionInput | null) => void; reject: (error: unknown) => void }[] =
      [];
   #failure: { error: unknown } | undefined;
   #chain: Promise<void> = Promise.resolve();
   #draining = false;
   #closed = false;
   #closing: Promise<void> | undefined;
   #record: string | undefined;
   #hooks: { exit: () => void; signal: (signal: NodeJS.Signals) => void } | undefined;
   #listener = (chunk: Uint8Array | string): void => this.#onData(chunk);

   /** Use `connect`, which detects Tern and shakes hands first. */
   constructor(input: TermInput, output: TermOutput, options: ConnectOptions) {
      this.#input = input;
      this.#output = output;
      this.#options = options;
      const record = options.record ?? (options.env ?? process.env)['TERN_TSP_RECORD'];
      this.#record = record === '' ? undefined : record;
      this.#caps = {
         term: '',
         ver: '',
         kinds: [...KINDS],
         features: [],
         apc: DEFAULT_APC,
         credits: DEFAULT_CREDITS,
         cols: output.columns ?? 80,
         cell: { w: 8, h: 16 },
         dark: true,
         reduceMotion: false,
         hour12: false,
      };
   }

   /** What the terminal supports, kept current by `resize`, `theme` and `motion`. */
   get caps(): Capabilities {
      return this.#caps;
   }

   /** Whether the session is closed. */
   get closed(): boolean {
      return this.#closed;
   }

   /** The open surfaces. */
   get surfaces(): readonly Surface[] {
      return [...this.#surfaces.values()];
   }

   /**
    * Switches the tty to raw mode, sends `hello` and a DA1 request, and
    * waits for the reply; restores the tty when there is none.
    */
   async handshake(): Promise<boolean> {
      const options = this.#options;
      if (ownedInputs.has(this.#input)) throw new Error('the tty already has a session');
      ownedInputs.add(this.#input);
      this.#ownsInput = true;
      this.#wasRaw = this.#input.isRaw === true;
      const { promise, resolve } = Promise.withResolvers<boolean>();
      const timer = setTimeout(() => resolve(false), options.timeout ?? 1000);
      this.#handshake = resolve;
      let connected = false;
      try {
         if (options.exitHooks ?? (options.input === undefined && options.output === undefined))
            this.#installHooks();
         this.#input.setRawMode?.(true);
         this.#attach();
         const script = process.argv[1] ?? process.argv0;
         const hello: HelloQuery = {
            q: 'hello',
            v: [PROTOCOL_VERSION],
            app: options.app ?? basename(script, extname(script)),
            ...(options.version === undefined ? {} : { ver: options.version }),
            ...(options.features === undefined ? {} : { features: options.features }),
         };
         this.#send({ verb: 'q', body: hello });
         this.#output.write(DA1_QUERY);
         if (!(await promise) || this.#draining || this.#closed) return false;
         this.#modesArmed = true;
         let modes = '';
         if (options.bracketedPaste !== false) modes += PASTE_ON;
         if (options.kittyKeyboard !== false) modes += KITTY_PUSH;
         if (modes !== '') this.#output.write(modes);
         connected = true;
         return true;
      } finally {
         clearTimeout(timer);
         this.#handshake = undefined;
         if (!connected) await this.close();
      }
   }

   /** Opens a surface (`o`); ids default to `s1`, `s2`, … */
   open(options: OpenOptions = {}): Surface {
      if (this.#closed) throw new Error('the session is closed');
      this.#surfaceCount += 1;
      const id = options.id ?? `s${this.#surfaceCount}`;
      const surface = new Surface(
         {
            send: message => this.#send(message),
            credits: () => this.#caps.credits,
            dropped: closed => {
               if (this.#surfaces.get(closed.id) === closed) {
                  this.#surfaces.delete(closed.id);
                  this.#retired.set(closed.id, closed);
               }
            },
         },
         id,
         options,
         options.adopt ? this.#retired.get(id) : undefined
      );
      this.#send({
         verb: 'o',
         body: {
            id,
            mode: surface.mode,
            ...(options.title === undefined ? {} : { title: options.title }),
            ...(options.role === undefined ? {} : { role: options.role }),
            ...(surface.listen ? {} : { listen: false }),
            ...(options.adopt ? { adopt: true } : {}),
         },
      });
      this.#surfaces.set(id, surface);
      this.#retired.delete(id);
      return surface;
   }

   /** The open surface with id `id`. */
   surface(id: string): Surface | undefined {
      return this.#surfaces.get(id);
   }

   /** Sends a blob once per session (`b`) and returns its id for `image` nodes. */
   blob(bytes: Uint8Array, mime?: string): string {
      const id = blobId(bytes);
      if (this.#blobsSent.has(id) || this.#closed) return id;
      const encoded = encodeBlob(bytes, mime, {
         limit: this.#caps.apc,
         chunk: () => this.#chunkId(),
      });
      const params: Record<string, string> = {};
      for (const [key, value] of encoded.params) params[key] = value;
      this.#recordLine('out', 'b', params, encoded.body);
      this.#output.write(encoded.bytes);
      this.#blobsSent.add(id);
      return id;
   }

   /** Asks which of these blobs Tern still holds. */
   blobs(ids: readonly string[]): Promise<string[]> {
      if (this.#closed) return Promise.resolve([]);
      const { promise, resolve } = Promise.withResolvers<string[]>();
      this.#blobWaiters.push(resolve);
      this.#send({ verb: 'q', body: { q: 'blobs', ids } });
      return promise;
   }

   /** The next key or unhandled event; `null` once the session is closed. */
   next(): Promise<SessionInput | null> {
      const input = this.#queue.shift();
      if (input) return Promise.resolve(input);
      if (this.#failure) {
         const { error } = this.#failure;
         this.#failure = undefined;
         return Promise.reject(error);
      }
      if (this.#closed || this.#closing) return Promise.resolve(null);
      const { promise, resolve, reject } = Promise.withResolvers<SessionInput | null>();
      this.#waiters.push({ resolve, reject });
      return promise;
   }

   /** Keys and unhandled events until the session closes. */
   async *[Symbol.asyncIterator](): AsyncGenerator<SessionInput, void, undefined> {
      for (;;) {
         const input = await this.next();
         if (input === null) return;
         yield input;
      }
   }

   /**
    * Closes the open surfaces (each with `keep`), undoes the modes, drains
    * input for 50 ms and restores the tty.
    */
   close(): Promise<void> {
      this.#closing ??= this.#close();
      return this.#closing;
   }

   /** Closes the session, for `await using`. */
   [Symbol.asyncDispose](): Promise<void> {
      return this.close();
   }

   async #close(): Promise<void> {
      if (this.#closed) return;
      this.#handshake?.(false);
      this.#draining = true;
      for (const waiter of this.#waiters.splice(0)) waiter.resolve(null);
      try {
         this.#leave();
      } finally {
         const drained = Promise.withResolvers<void>();
         setTimeout(drained.resolve, DRAIN_MS);
         try {
            await drained.promise;
         } finally {
            this.#finish();
         }
      }
   }

   /** Attempts every close and mode reset, even after an output failure. */
   #leave(): void {
      let failure: { error: unknown } | undefined;
      const attempt = (fn: () => void): void => {
         try {
            fn();
         } catch (error) {
            failure ??= { error };
         }
      };
      for (const surface of this.#surfaces.values())
         attempt(() => {
            void surface.close();
         });
      if (this.#modesArmed) {
         if (this.#options.kittyKeyboard !== false)
            attempt(() => {
               this.#output.write(KITTY_POP);
            });
         if (this.#options.bracketedPaste !== false)
            attempt(() => {
               this.#output.write(PASTE_OFF);
            });
         this.#modesArmed = false;
      }
      if (failure) throw failure.error;
   }

   /** Stops reading, restores the tty and ends the input loop. */
   #finish(): void {
      try {
         this.#detach();
      } finally {
         try {
            if (this.#ownsInput) this.#input.setRawMode?.(this.#wasRaw);
         } finally {
            if (this.#ownsInput) ownedInputs.delete(this.#input);
            this.#ownsInput = false;
            this.#closed = true;
            this.#removeHooks();
            for (const waiter of this.#waiters.splice(0)) waiter.resolve(null);
            for (const waiter of this.#blobWaiters.splice(0)) waiter([]);
         }
      }
   }

   #installHooks(): void {
      const exit = (): void => {
         if (this.#closed) return;
         try {
            this.#leave();
         } finally {
            this.#finish();
         }
      };
      const signal = (name: NodeJS.Signals): void => {
         const reraise = (): void => {
            process.kill(process.pid, name);
         };
         void this.close().then(reraise, reraise);
      };
      this.#hooks = { exit, signal };
      process.once('exit', exit);
      process.once('SIGINT', signal);
      process.once('SIGTERM', signal);
      process.once('SIGHUP', signal);
   }

   #removeHooks(): void {
      const hooks = this.#hooks;
      if (!hooks) return;
      this.#hooks = undefined;
      process.off('exit', hooks.exit);
      process.off('SIGINT', hooks.signal);
      process.off('SIGTERM', hooks.signal);
      process.off('SIGHUP', hooks.signal);
   }

   #attach(): void {
      if (this.#attached) return;
      this.#attached = true;
      this.#input.on('data', this.#listener);
      this.#input.resume?.();
   }

   #detach(): void {
      if (!this.#attached) return;
      this.#attached = false;
      clearTimeout(this.#flushTimer);
      this.#input.off('data', this.#listener);
      this.#input.pause?.();
   }

   #chunkId(): string {
      this.#chunkCount += 1;
      return this.#chunkCount.toString(36);
   }

   /** Writes (and records) one message. */
   #send(message: OutMessage): void {
      this.#recordLine('out', message.verb, {}, message.body);
      this.#output.write(
         encodeJson(message, { limit: this.#caps.apc, chunk: () => this.#chunkId() })
      );
   }

   /** Appends one JSONL line to the recording, when recording. */
   #recordLine(
      dir: 'in' | 'out',
      verb: string,
      params: Record<string, string>,
      body: unknown
   ): void {
      const path = this.#record;
      if (path === undefined) return;
      try {
         appendFileSync(path, `${JSON.stringify({ t: Date.now(), dir, verb, params, body })}\n`);
      } catch {
         this.#record = undefined;
      }
   }

   #onData(chunk: Uint8Array | string): void {
      const bytes = typeof chunk === 'string' ? encoder.encode(chunk) : chunk;
      clearTimeout(this.#flushTimer);
      if (this.#draining) return;
      this.#items(this.#parser.feed(bytes));
      if (this.#parser.pending || this.#keys.pending) {
         this.#flushTimer = setTimeout(() => this.#flush(), FLUSH_MS);
      }
   }

   /** Releases undecided input after a pause. */
   #flush(): void {
      if (this.#draining) return;
      this.#items(this.#parser.flush());
      for (const key of this.#keys.flush()) this.#deliver({ type: 'key', key });
   }

   #items(items: readonly InputItem[]): void {
      for (const item of items) {
         switch (item.type) {
            case 'keys':
               for (const key of this.#keys.feed(item.bytes)) this.#deliver({ type: 'key', key });
               break;
            case 'da1':
               this.#handshake?.(false);
               this.#handshake = undefined;
               break;
            case 'reply':
               this.#recordLine('in', 'r', {}, item.reply.raw);
               if (item.reply.r === 'hello') this.#hello(item.reply);
               else if (item.reply.r === 'blobs') this.#blobWaiters.shift()?.([...item.reply.have]);
               break;
            case 'event':
               this.#recordLine('in', 'e', {}, item.event.raw);
               this.#event(item.event);
               break;
         }
      }
   }

   #hello(reply: HelloReply): void {
      this.#caps = {
         term: reply.term,
         ver: reply.ver,
         kinds: reply.kinds,
         features: reply.features,
         apc: reply.apc,
         credits: reply.credits,
         cols: reply.cols,
         cell: reply.cell,
         dark: reply.dark,
         reduceMotion: reply.reduceMotion,
         hour12: reply.hour12,
      };
      this.#handshake?.(true);
      this.#handshake = undefined;
   }

   /** Updates state from an event, then routes it to a handler or the program. */
   #event(event: TspEvent): void {
      switch (event.ev) {
         case 'ack':
            try {
               this.#surfaces.get(event.sf)?.acknowledge(event.s);
            } catch (error) {
               this.#chain = this.#chain.then(() => this.#fail(error));
            }
            return;
         case 'resize':
            this.#caps = { ...this.#caps, cols: event.cols, cell: event.cell ?? this.#caps.cell };
            break;
         case 'theme':
            this.#caps = { ...this.#caps, dark: event.dark };
            break;
         case 'motion':
            this.#caps = { ...this.#caps, reduceMotion: event.reduce };
            break;
         case 'gone':
            for (const id of event.ids) this.#surfaces.get(id)?.gone();
            break;
         default: {
            const sf = 'sf' in event ? event.sf : undefined;
            const surface = sf === undefined ? undefined : this.#surfaces.get(sf);
            const dispatch = surface?.dispatch(event);
            if (dispatch) {
               this.#chain = this.#chain
                  .then(dispatch)
                  .catch((error: unknown) => this.#fail(error));
               if (event.ev !== 'action' || !surface?.forwardActions.includes(event.act)) return;
            }
         }
      }
      this.#deliver({ type: 'event', event });
   }

   /** Hands an input to the program, after every handler queued before it. */
   #deliver(input: SessionInput): void {
      this.#chain = this.#chain.then(() => {
         const waiter = this.#waiters.shift();
         if (waiter) waiter.resolve(input);
         else this.#queue.push(input);
      });
   }

   /** A handler threw: the program's next read rejects with the error. */
   #fail(error: unknown): void {
      const waiter = this.#waiters.shift();
      if (waiter) waiter.reject(error);
      else this.#failure = { error };
   }
}

/**
 * Connects to Tern: the session, or `null` when TSP isn't available
 * (`TERN_TSP=0`, not a tty, inside tmux/screen/zellij, or no reply before
 * DA1 or the timeout).
 */
export async function connect(options: ConnectOptions = {}): Promise<Session | null> {
   const env = options.env ?? process.env;
   if (env['TERN_TSP'] === '0' || env['TMUX'] || env['STY'] || env['ZELLIJ']) return null;
   const input = options.input ?? process.stdin;
   const output = options.output ?? process.stdout;
   if (!input.isTTY || !output.isTTY) return null;
   const session = new Session(input, output, options);
   return (await session.handshake()) ? session : null;
}
