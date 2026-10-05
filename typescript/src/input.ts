/**
 * The input layer's stream parser: splits TSP replies, events and DA1
 * answers out of the pty's input and passes every other byte through as key
 * bytes.
 */
import {
   concatBytes,
   decodeEvent,
   decodeReply,
   isJsonObject,
   splitMessage,
   type Reply,
   type TspEvent,
} from './wire.js';

/** One thing the input parser found, in input order. */
export type InputItem =
   | { readonly type: 'keys'; readonly bytes: Uint8Array }
   | { readonly type: 'reply'; readonly reply: Reply }
   | { readonly type: 'event'; readonly event: TspEvent }
   | { readonly type: 'da1' };

/** A TSP message longer than this is dropped. */
export const MAX_INPUT_MESSAGE = 32 * 1024 * 1024;

const ESC = 0x1b;
const BEL = 0x07;
const APC_TSP = [0x74, 0x73, 0x70, 0x3b];
const OSC_TSP = [0x38, 0x37, 0x37, 0x3b, 0x74, 0x73, 0x70, 0x3b];
const PASTE_START = [ESC, 0x5b, 0x32, 0x30, 0x30, 0x7e];
const PASTE_END = [ESC, 0x5b, 0x32, 0x30, 0x31, 0x7e];
const EMPTY = new Uint8Array();
const decoder = new TextDecoder();

/** A TSP message being collected up to its terminator. */
interface Collecting {
   parts: Uint8Array[];
   size: number;
   /** The last byte seen was an ESC that may start ST. */
   esc: boolean;
}

/** Where `needle` starts in `bytes` at or after `from`, or -1. */
export function indexOfSeq(bytes: Uint8Array, needle: readonly number[], from: number): number {
   const first = needle[0];
   for (let i = bytes.indexOf(first ?? 0, from); i >= 0; i = bytes.indexOf(first ?? 0, i + 1)) {
      if (i + needle.length > bytes.length) return -1;
      let match = true;
      for (let k = 1; k < needle.length; k++) {
         if (bytes[i + k] !== needle[k]) {
            match = false;
            break;
         }
      }
      if (match) return i;
   }
   return -1;
}

/** The length of the longest end of `bytes` (from `from`) that starts `needle`. */
export function partialSuffix(bytes: Uint8Array, needle: readonly number[], from: number): number {
   const max = Math.min(needle.length - 1, bytes.length - from);
   for (let len = max; len > 0; len--) {
      let match = true;
      for (let k = 0; k < len; k++) {
         if (bytes[bytes.length - len + k] !== needle[k]) {
            match = false;
            break;
         }
      }
      if (match) return len;
   }
   return 0;
}

/** What an ESC at some position starts. */
type Escape =
   | { readonly kind: 'need' }
   | { readonly kind: 'pass'; readonly end: number }
   | { readonly kind: 'da1'; readonly end: number }
   | { readonly kind: 'paste'; readonly end: number }
   | { readonly kind: 'tsp'; readonly start: number };

/** Matches `prefix` at `at`: the bytes after it, `need` when cut short, or -1 on a mismatch. */
function matchPrefix(bytes: Uint8Array, at: number, prefix: readonly number[]): number | 'need' {
   for (let k = 0; k < prefix.length; k++) {
      const b = bytes[at + k];
      if (b === undefined) return 'need';
      if (b !== prefix[k]) return -1;
   }
   return at + prefix.length;
}

/** Classifies the escape sequence starting at `i` (an ESC). */
function escapeAt(bytes: Uint8Array, i: number): Escape {
   const next = bytes[i + 1];
   if (next === undefined) return { kind: 'need' };
   if (next === 0x5f || next === 0x5d) {
      const end = matchPrefix(bytes, i + 2, next === 0x5f ? APC_TSP : OSC_TSP);
      if (end === 'need') return { kind: 'need' };
      return end < 0 ? { kind: 'pass', end: i + 2 } : { kind: 'tsp', start: end };
   }
   if (next !== 0x5b) return { kind: 'pass', end: i + 1 };
   let j = i + 2;
   while (j < bytes.length && (bytes[j] ?? 0) >= 0x30 && (bytes[j] ?? 0) <= 0x3f) j++;
   while (j < bytes.length && (bytes[j] ?? 0) >= 0x20 && (bytes[j] ?? 0) <= 0x2f) j++;
   const final = bytes[j];
   if (final === undefined) return j - i > 64 ? { kind: 'pass', end: j } : { kind: 'need' };
   if (final < 0x40 || final > 0x7e) return { kind: 'pass', end: j };
   const end = j + 1;
   if (final === 0x63 && bytes[i + 2] === 0x3f) {
      let da1 = true;
      for (let k = i + 3; k < j; k++) {
         const b = bytes[k] ?? 0;
         if (!((b >= 0x30 && b <= 0x39) || b === 0x3b)) da1 = false;
      }
      if (da1) return { kind: 'da1', end };
   }
   if (matchPrefix(bytes, i, PASTE_START) === end) return { kind: 'paste', end };
   return { kind: 'pass', end };
}

/**
 * A streaming parser of the pty's input. Feed it bytes as they arrive; it
 * returns, in order, key bytes (passed through exactly), replies, events
 * and DA1 answers. Undecided prefixes wait for more bytes or `flush()`.
 */
export class InputParser {
   /** Bytes held until more input decides them. */
   #held: Uint8Array = EMPTY;
   /** A TSP message being collected, and whether it came as OSC 877. */
   #message: Collecting | null = null;
   /** Inside a bracketed paste: bytes pass through until its end. */
   #paste = false;

   /** Whether bytes are held back (a flush would release some, or a message is open). */
   get pending(): boolean {
      return this.#held.length > 0 || this.#message !== null;
   }

   /** Parses the next bytes of input. */
   feed(data: Uint8Array): InputItem[] {
      const bytes = this.#held.length > 0 ? concatBytes([this.#held, data]) : data;
      this.#held = EMPTY;
      const out: InputItem[] = [];
      let keys = 0;
      let i = 0;
      const pass = (end: number): void => {
         if (end > keys) out.push({ type: 'keys', bytes: bytes.slice(keys, end) });
      };
      while (i < bytes.length) {
         if (this.#message) {
            const end = this.#collect(bytes, i, out);
            i = end;
            keys = end;
            continue;
         }
         if (this.#paste) {
            const at = indexOfSeq(bytes, PASTE_END, i);
            if (at >= 0) {
               i = at + PASTE_END.length;
               this.#paste = false;
               continue;
            }
            const hold = partialSuffix(bytes, PASTE_END, i);
            pass(bytes.length - hold);
            this.#held = bytes.slice(bytes.length - hold);
            return out;
         }
         const esc = bytes.indexOf(ESC, i);
         if (esc < 0) break;
         const found = escapeAt(bytes, esc);
         switch (found.kind) {
            case 'need':
               pass(esc);
               this.#held = bytes.slice(esc);
               return out;
            case 'pass':
               i = found.end;
               break;
            case 'paste':
               i = found.end;
               this.#paste = true;
               break;
            case 'da1':
               pass(esc);
               out.push({ type: 'da1' });
               i = found.end;
               keys = i;
               break;
            case 'tsp':
               pass(esc);
               this.#message = { parts: [], size: 0, esc: false };
               i = found.start;
               keys = i;
               break;
         }
      }
      pass(bytes.length);
      return out;
   }

   /** Releases held undecided bytes as keys; a TSP message already recognized stays. */
   flush(): InputItem[] {
      if (this.#paste || this.#held.length === 0) return [];
      const bytes = this.#held;
      this.#held = EMPTY;
      return [{ type: 'keys', bytes }];
   }

   /** Collects message bytes from `i`; returns where scanning resumes. */
   #collect(bytes: Uint8Array, i: number, out: InputItem[]): number {
      const msg = this.#message;
      if (!msg) return i;
      if (msg.esc) {
         msg.esc = false;
         if (bytes[i] === 0x5c) {
            this.#finish(out);
            return i + 1;
         }
         this.#message = null;
         return i;
      }
      for (let j = i; j < bytes.length; j++) {
         const b = bytes[j];
         if (b !== BEL && b !== ESC) continue;
         this.#append(bytes.subarray(i, j));
         if (b === BEL) {
            this.#finish(out);
            return j + 1;
         }
         const after = bytes[j + 1];
         if (after === undefined) {
            msg.esc = true;
            return j + 1;
         }
         if (after === 0x5c) {
            this.#finish(out);
            return j + 2;
         }
         this.#message = null;
         return j;
      }
      this.#append(bytes.subarray(i));
      return bytes.length;
   }

   /** Adds body bytes to the open message, dropping it past the limit. */
   #append(part: Uint8Array): void {
      const msg = this.#message;
      if (!msg || part.length === 0) return;
      msg.size += part.length;
      if (msg.size > MAX_INPUT_MESSAGE) {
         msg.parts = [];
         return;
      }
      msg.parts.push(part.slice());
   }

   /** Ends the open message and delivers it when it is an `r` or `e` with a JSON object. */
   #finish(out: InputItem[]): void {
      const msg = this.#message;
      this.#message = null;
      if (!msg || msg.size > MAX_INPUT_MESSAGE) return;
      const split = splitMessage(concatBytes(msg.parts));
      if (!split || (split.verb !== 'r' && split.verb !== 'e')) return;
      let body: unknown;
      try {
         body = JSON.parse(decoder.decode(split.body));
      } catch {
         return;
      }
      if (!isJsonObject(body)) return;
      out.push(
         split.verb === 'r'
            ? { type: 'reply', reply: decodeReply(body) }
            : { type: 'event', event: decodeEvent(body) }
      );
   }
}
