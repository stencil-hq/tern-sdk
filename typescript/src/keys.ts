/**
 * The input layer's key decoder: turns key bytes (legacy xterm, kitty
 * `CSI u`, bracketed paste) into keys named as in Tern's plugin API.
 */
import { indexOfSeq, partialSuffix } from './input.js';
import { concatBytes } from './wire.js';

/** A named key (`Key` in the plugin API), or a newer name. */
export type KeyName =
   | 'enter'
   | 'tab'
   | 'backspace'
   | 'escape'
   | 'space'
   | 'up'
   | 'down'
   | 'left'
   | 'right'
   | 'home'
   | 'end'
   | 'begin'
   | 'page_up'
   | 'page_down'
   | 'insert'
   | 'delete'
   | 'menu'
   | 'caps_lock'
   | 'scroll_lock'
   | 'num_lock'
   | 'print_screen'
   | 'pause'
   | 'paste'
   | `f${number}`
   | (string & {});

/** One key press: its name, what it types, and the modifiers held. */
export interface Key {
   /** A lowercase character (`a`, `+`, `é`), `space`, or a named key. */
   readonly name: KeyName;
   /** What the key types, when it types something; a paste's text. */
   readonly text?: string;
   readonly ctrl: boolean;
   readonly alt: boolean;
   readonly shift: boolean;
   /** Cmd on macOS, Super elsewhere. */
   readonly meta: boolean;
}

const ESC = 0x1b;
const PASTE_END = [ESC, 0x5b, 0x32, 0x30, 0x31, 0x7e];
const EMPTY = new Uint8Array();
const decoder = new TextDecoder();

/** Modifier flags, as `CSI …;<1 + bits>` carries them. */
interface Mods {
   ctrl: boolean;
   alt: boolean;
   shift: boolean;
   meta: boolean;
}

const NO_MODS: Mods = { ctrl: false, alt: false, shift: false, meta: false };

/** The modifiers of an xterm/kitty modifier parameter (`1 + bits`). */
function modsOf(param: number | undefined): Mods {
   const bits = Math.max(0, (param ?? 1) - 1);
   return {
      shift: (bits & 1) !== 0,
      alt: (bits & 2) !== 0,
      ctrl: (bits & 4) !== 0,
      meta: (bits & 8) !== 0,
   };
}

/** A key with the given name, text and modifiers. */
function key(name: string, mods: Mods, text?: string): Key {
   return text === undefined ? { name, ...mods } : { name, text, ...mods };
}

/** Legacy CSI and SS3 finals. */
const LETTER_KEYS: Readonly<Record<string, string>> = {
   A: 'up',
   B: 'down',
   C: 'right',
   D: 'left',
   H: 'home',
   F: 'end',
   E: 'begin',
   P: 'f1',
   Q: 'f2',
   R: 'f3',
   S: 'f4',
};

/** `CSI <n> ~` keys. */
const TILDE_KEYS: Readonly<Record<number, string>> = {
   1: 'home',
   2: 'insert',
   3: 'delete',
   4: 'end',
   5: 'page_up',
   6: 'page_down',
   7: 'home',
   8: 'end',
   11: 'f1',
   12: 'f2',
   13: 'f3',
   14: 'f4',
   15: 'f5',
   17: 'f6',
   18: 'f7',
   19: 'f8',
   20: 'f9',
   21: 'f10',
   23: 'f11',
   24: 'f12',
   25: 'f13',
   26: 'f14',
   28: 'f15',
   29: 'menu',
   31: 'f17',
   32: 'f18',
   33: 'f19',
   34: 'f20',
};

/** Kitty's functional key codes (Private Use Area) that are keys of the API. */
const KITTY_KEYS: Readonly<Record<number, string>> = {
   57358: 'caps_lock',
   57359: 'scroll_lock',
   57360: 'num_lock',
   57361: 'print_screen',
   57362: 'pause',
   57363: 'menu',
   57414: 'enter',
   57417: 'left',
   57418: 'right',
   57419: 'up',
   57420: 'down',
   57421: 'page_up',
   57422: 'page_down',
   57423: 'home',
   57424: 'end',
   57425: 'insert',
   57426: 'delete',
   57427: 'begin',
};

/** Kitty keypad codes that type a character. */
const KEYPAD_CHARS: Readonly<Record<number, string>> = {
   57409: '.',
   57410: '/',
   57411: '*',
   57412: '-',
   57413: '+',
   57415: '=',
   57416: ',',
};

/** The key a code point names (kitty `CSI u`, control bytes, typed characters), with `mods`. */
function codeKey(code: number, mods: Mods, functional = true): Key | null {
   if (!Number.isInteger(code) || code < 0 || code > 0x10ffff || (code >= 0xd800 && code <= 0xdfff))
      return null;
   const typing = !mods.ctrl && !mods.alt && !mods.meta;
   switch (code) {
      case 13:
         return key('enter', mods);
      case 9:
         return key('tab', mods);
      case 8:
      case 127:
         return key('backspace', mods);
      case 27:
         return key('escape', mods);
      case 32:
         return key('space', mods, typing ? ' ' : undefined);
   }
   if (functional) {
      const named = KITTY_KEYS[code];
      if (named !== undefined) return key(named, mods);
      if (code >= 57376 && code <= 57398) return key(`f${code - 57376 + 13}`, mods);
      if (code >= 57399 && code <= 57408) {
         const digit = String(code - 57399);
         return key(digit, mods, typing ? digit : undefined);
      }
      const pad = KEYPAD_CHARS[code];
      if (pad !== undefined) return key(pad, mods, typing ? pad : undefined);
      if (code >= 57344 && code <= 63743) return null;
   }
   if (code < 32 || (code >= 0x7f && code < 0xa0)) return null;
   const char = String.fromCodePoint(code);
   const lower = char.toLowerCase();
   const shift = mods.shift || lower !== char;
   const typed = shift && lower === char ? char.toUpperCase() : char;
   return key(lower, { ...mods, shift }, typing ? typed : undefined);
}

/** The key of a C0 control byte (Ctrl+letter and friends), with `alt`. */
function controlKey(b: number, alt: boolean): Key | null {
   const mods = { ...NO_MODS, alt };
   switch (b) {
      case 0x0d:
      case 0x0a:
         return key('enter', mods);
      case 0x09:
         return key('tab', mods);
      case 0x08:
         return key('backspace', { ...mods, ctrl: true });
      case 0x7f:
         return key('backspace', mods);
      case 0x00:
         return key('space', { ...mods, ctrl: true });
      case 0x1b:
         return key('escape', mods);
   }
   if (b >= 0x01 && b <= 0x1a) return key(String.fromCharCode(b + 0x60), { ...mods, ctrl: true });
   if (b >= 0x1c && b <= 0x1f) return key('\\]^_'.charAt(b - 0x1c), { ...mods, ctrl: true });
   return null;
}

/** The byte length of the UTF-8 sequence led by `b` (1 for an invalid lead). */
function utf8Length(b: number): number {
   if (b >= 0xf0 && b <= 0xf7) return 4;
   if (b >= 0xe0) return 3;
   if (b >= 0xc0) return 2;
   return 1;
}

/** The numbers of a CSI parameter string (`1;5`, `97:65;6`): each first sub-parameter. */
function csiParams(params: string): (number | undefined)[] {
   if (params === '') return [];
   return params.split(';').map(part => {
      const head = part.split(':')[0] ?? '';
      return head === '' ? undefined : Number(head);
   });
}

/** Decodes a CSI sequence's parameters and final into a key, or `null` to drop it. */
function csiKey(params: string, final: string): Key | null {
   if (!/^[0-9:;]*$/.test(params)) return null;
   const nums = csiParams(params);
   const mods = modsOf(nums[1]);
   if (final === 'u') {
      const code = nums[0];
      return code === undefined ? null : codeKey(code, mods);
   }
   if (final === '~') {
      if (nums[0] === 27 && nums[2] !== undefined) return codeKey(nums[2], mods);
      const name = TILDE_KEYS[nums[0] ?? 0];
      return name === undefined ? null : key(name, mods);
   }
   if (final === 'Z') return key('tab', { ...mods, shift: true });
   const name = LETTER_KEYS[final];
   return name === undefined ? null : key(name, mods);
}

/**
 * Decodes key bytes into keys. Feed it the input parser's key bytes; a
 * prefix that can't be decided yet (a lone ESC, a cut sequence) waits for
 * more bytes or `flush()`.
 */
export class KeyDecoder {
   #held: Uint8Array = EMPTY;
   /** The text of a bracketed paste so far, when inside one. */
   #paste: Uint8Array[] | null = null;

   /** Whether bytes wait for more input or a flush. */
   get pending(): boolean {
      return this.#held.length > 0 || this.#paste !== null;
   }

   /** Decodes the next key bytes. */
   feed(data: Uint8Array): Key[] {
      const bytes = this.#held.length > 0 ? concatBytes([this.#held, data]) : data;
      this.#held = EMPTY;
      const out: Key[] = [];
      const end = this.#decode(bytes, out, false);
      this.#held = bytes.slice(end);
      return out;
   }

   /** Decodes what is held as if no more bytes were coming (a lone ESC is Escape). */
   flush(): Key[] {
      if (this.#paste) return [];
      const out: Key[] = [];
      const bytes = this.#held;
      this.#held = EMPTY;
      this.#decode(bytes, out, true);
      return out;
   }

   /** Decodes keys from `bytes` into `out`; returns how many bytes it used. */
   #decode(bytes: Uint8Array, out: Key[], final: boolean): number {
      let i = 0;
      while (i < bytes.length) {
         if (this.#paste) {
            const at = indexOfSeq(bytes, PASTE_END, i);
            if (at < 0) {
               const hold = partialSuffix(bytes, PASTE_END, i);
               this.#paste.push(bytes.slice(i, bytes.length - hold));
               return bytes.length - hold;
            }
            this.#paste.push(bytes.slice(i, at));
            out.push({ ...key('paste', NO_MODS), text: decoder.decode(concatBytes(this.#paste)) });
            this.#paste = null;
            i = at + PASTE_END.length;
            continue;
         }
         const used = this.#one(bytes, i, out, final);
         if (used === 0) return i;
         i += used;
      }
      return i;
   }

   /** Decodes one key at `i`; returns the bytes it took, 0 when it needs more. */
   #one(bytes: Uint8Array, i: number, out: Key[], final: boolean): number {
      const b = bytes[i] ?? 0;
      if (b !== ESC) return this.#plain(bytes, i, out, false, final);
      const next = bytes[i + 1];
      if (next === undefined) {
         if (!final) return 0;
         out.push(key('escape', NO_MODS));
         return 1;
      }
      if (next === 0x5b || next === 0x4f) {
         return this.#sequence(bytes, i, i, out, false, final);
      }
      if (next === ESC) {
         const after = bytes[i + 2];
         if (after === undefined && !final) return 0;
         if (after === 0x5b || after === 0x4f) {
            const used = this.#sequence(bytes, i + 1, i, out, true, final);
            return used;
         }
         out.push(key('escape', { ...NO_MODS, alt: true }));
         return 2;
      }
      if (next === 0x5d || next === 0x5f) {
         for (let j = i + 2; j < bytes.length; j++) {
            if (bytes[j] === 0x07) return j + 1 - i;
            if (bytes[j] === ESC && bytes[j + 1] === 0x5c) return j + 2 - i;
         }
         return final ? bytes.length - i : 0;
      }
      const used = this.#plain(bytes, i + 1, out, true, final);
      return used === 0 ? 0 : used + 1;
   }

   /** Decodes a CSI or SS3 sequence whose ESC is at `at` (`start` is where the key began). */
   #sequence(
      bytes: Uint8Array,
      at: number,
      start: number,
      out: Key[],
      alt: boolean,
      final: boolean
   ): number {
      const intro = bytes[at + 1];
      let j = at + 2;
      while (j < bytes.length && (bytes[j] ?? 0) >= 0x30 && (bytes[j] ?? 0) <= 0x3f) j++;
      while (j < bytes.length && (bytes[j] ?? 0) >= 0x20 && (bytes[j] ?? 0) <= 0x2f) j++;
      const f = bytes[j];
      if (f === undefined) {
         if (!final) return 0;
         return bytes.length - start;
      }
      if (f < 0x40 || f > 0x7e) return j - start;
      const params = decoder.decode(bytes.subarray(at + 2, j));
      if (intro === 0x5b && params === '200' && f === 0x7e) {
         this.#paste = [];
         return j + 1 - start;
      }
      const found = csiKey(params, String.fromCharCode(f));
      if (found && alt) {
         out.push(
            key(found.name, { ctrl: found.ctrl, alt: true, shift: found.shift, meta: found.meta })
         );
      } else if (found) out.push(found);
      return j + 1 - start;
   }

   /** Decodes one typed character or control byte at `i`. */
   #plain(bytes: Uint8Array, i: number, out: Key[], alt: boolean, final: boolean): number {
      const b = bytes[i] ?? 0;
      if (b < 0x20 || b === 0x7f) {
         const found = controlKey(b, alt);
         if (found) out.push(found);
         return 1;
      }
      const len = utf8Length(b);
      if (i + len > bytes.length) return final ? bytes.length - i : 0;
      const char = decoder.decode(bytes.subarray(i, i + len));
      const code = char.codePointAt(0) ?? 0xfffd;
      const found = codeKey(code, { ...NO_MODS, alt }, false);
      if (found) out.push(found);
      return len;
   }
}
