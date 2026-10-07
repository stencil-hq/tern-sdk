/**
 * The wire layer: TSP constants, the typed messages in both directions, the
 * program → terminal encoder (framing, chunking, blobs) and the decoders of
 * terminal → program bodies.
 */
import { createHash } from 'node:crypto';

/** The protocol version this SDK speaks. */
export const PROTOCOL_VERSION = 1;
/** The largest body sent in one APC string unless the `hello` reply says otherwise. */
export const DEFAULT_APC = 65536;
/** How many frames may be unacknowledged unless the `hello` reply says otherwise. */
export const DEFAULT_CREDITS = 2;
/** The largest blob Tern takes, in decoded bytes. */
export const MAX_BLOB = 16 * 1024 * 1024;

/** Every node kind of the TSP v1 vocabulary, in vocabulary order. */
export const KINDS = [
   'col',
   'row',
   'card',
   'section',
   'rule',
   'spacer',
   'text',
   'md',
   'code',
   'diff',
   'ansi',
   'math',
   'image',
   'kv',
   'table',
   'tree',
   'badge',
   'kbd',
   'icon',
   'spinner',
   'shimmer',
   'elapsed',
   'progress',
   'rate',
   'list',
   'item',
   'tabs',
   'editor',
   'input',
   'status',
   'seg',
   'overlay',
   'toast',
   'rows',
   'picker',
   'prefs',
   'tool',
   'checklist',
   'agent',
   'chart',
   'meter',
   'block',
   'effort',
   'el',
] as const;

/** A node kind of the vocabulary. */
export type Kind = (typeof KINDS)[number];

/** The kinds whose primary text is the `text` prop, addressed by the `text` and `splice` ops. */
export const TEXT_KINDS = [
   'text',
   'md',
   'code',
   'ansi',
   'math',
   'editor',
   'input',
   'shimmer',
   'el',
] as const;

/** A kind whose primary text is the `text` prop. */
export type TextKind = (typeof TEXT_KINDS)[number];

const textKinds: Readonly<Record<string, true>> = Object.fromEntries(
   TEXT_KINDS.map(kind => [kind, true])
);

/** Whether `kind` keeps its primary text in the `text` prop. */
export function isTextKind(kind: string): boolean {
   return textKinds[kind] === true;
}

/** The features a program announces in its `hello` query. */
export const PROGRAM_FEATURES = ['edit', 'undo', 'send'] as const;

/** A feature a program announces in its `hello` query. */
export type ProgramFeature = (typeof PROGRAM_FEATURES)[number];

/** The terminal features a `hello` reply lists. */
export const TERMINAL_FEATURES = [
   'blobs',
   'settle',
   'adopt',
   'dock',
   'program-palette',
   'reduce-motion',
   'aside',
   'scroll',
   'styles',
   'flow',
] as const;

/** A terminal feature of the `hello` reply, or a newer one. */
export type TerminalFeature = (typeof TERMINAL_FEATURES)[number] | (string & {});

/** A JSON value. */
export type Json = null | boolean | number | string | readonly Json[] | JsonObject;

/** A JSON object. */
export interface JsonObject {
   readonly [key: string]: Json;
}

/** A node as it travels in an `add` op: id, kind, props and children. */
export interface WireNode {
   readonly id: string;
   readonly k: string;
   readonly p?: JsonObject;
   readonly c?: readonly WireNode[];
}

/** Where `reveal` puts a node. */
export type RevealAt = 'start' | 'end' | 'nearest';

/** How far `scroll` moves. */
export type ScrollTo = 'line-up' | 'line-down' | 'page-up' | 'page-down' | 'start' | 'end';

/** One frame op (Documents and Frames, frame ops). */
export type Op =
   | readonly ['add', string, string, string | null, WireNode]
   | readonly ['set', string, JsonObject]
   | readonly ['text', string, 'append' | 'replace', string]
   | readonly ['splice', string, number, number, string]
   | readonly ['move', string, string, string | null]
   | readonly ['del', string]
   | readonly ['settle', string]
   | readonly ['focus', string | null]
   | readonly ['reveal', string, RevealAt]
   | readonly ['scroll', string, ScrollTo]
   | readonly ['suspend']
   | readonly ['resume'];

/** A surface's mode. */
export type SurfaceMode = 'inline' | 'screen' | 'flow';

/** The `hello` query (`q`). */
export interface HelloQuery {
   readonly q: 'hello';
   readonly v: readonly number[];
   readonly app?: string;
   readonly ver?: string;
   readonly features?: readonly string[];
}

/** The `blobs` query (`q`): which of these blobs Tern still holds. */
export interface BlobsQuery {
   readonly q: 'blobs';
   readonly ids: readonly string[];
}

/** A query (`q`). */
export type Query = HelloQuery | BlobsQuery;

/** Opening a surface (`o`). */
export interface OpenMessage {
   readonly id: string;
   readonly mode?: SurfaceMode;
   readonly title?: string;
   readonly role?: string;
   readonly listen?: boolean;
   readonly adopt?: boolean;
}

/** A frame (`f`): an atomic batch of ops for one surface. */
export interface FrameMessage {
   readonly sf: string;
   readonly s: number;
   readonly ops: readonly Op[];
}

/** One palette variant: token name to `#rrggbb`. */
export type PaletteColors = Readonly<Record<string, string>>;

/** The program palette (`t`). */
export interface PaletteMessage {
   readonly sf?: string;
   readonly dark?: PaletteColors;
   readonly light?: PaletteColors;
   readonly name?: { readonly dark?: string; readonly light?: string };
}

/** A stylesheet (`s`); an absent `css` removes the sheet. */
export interface StylesheetMessage {
   readonly sf?: string;
   readonly name: string;
   readonly css?: string;
}

/** Closing a surface (`x`). */
export interface CloseMessage {
   readonly id: string;
   readonly keep?: boolean;
}

/** A program → terminal message with a JSON body, by verb. */
export type OutMessage =
   | { readonly verb: 'q'; readonly body: Query }
   | { readonly verb: 'o'; readonly body: OpenMessage }
   | { readonly verb: 'f'; readonly body: FrameMessage }
   | { readonly verb: 't'; readonly body: PaletteMessage }
   | { readonly verb: 's'; readonly body: StylesheetMessage }
   | { readonly verb: 'x'; readonly body: CloseMessage };

/** A message parameter: key and value. */
export type Param = readonly [string, string];

/** How to frame one message. */
export interface EncodeOptions {
   /** Parameters written before the body (a blob's `mime`). */
   readonly params?: readonly Param[];
   /** The APC limit; bodies over it are chunked. Default {@link DEFAULT_APC}. */
   readonly limit?: number;
   /** The chunk id, asked for only when the body has to be chunked. */
   readonly chunk?: () => string;
}

const encoder = new TextEncoder();
const APC_START = encoder.encode('\x1b_tsp;');
const ST = encoder.encode('\x1b\\');

/** Whether `b` may appear in a parameter key. */
function keyByte(b: number): boolean {
   return (
      (b >= 0x30 && b <= 0x39) ||
      (b >= 0x41 && b <= 0x5a) ||
      (b >= 0x61 && b <= 0x7a) ||
      b === 0x5f ||
      b === 0x2d
   );
}

/** Whether `b` may appear in a parameter value: printable ASCII other than `;`. */
function valueByte(b: number): boolean {
   return b >= 0x21 && b <= 0x7e && b !== 0x3b;
}

/** Whether `bytes` from `at` leads with a parameter-shaped segment (`key=value;`). */
export function leadsWithParameter(bytes: Uint8Array, at: number): boolean {
   let i = at;
   while (i < bytes.length && keyByte(bytes[i] ?? 0)) i++;
   if (i === at || bytes[i] !== 0x3d) return false;
   i++;
   while (i < bytes.length && valueByte(bytes[i] ?? 0)) i++;
   return bytes[i] === 0x3b;
}

/** Where the chunk of `body` starting at `start` ends, as Tern's `chunk_end`. */
function chunkEnd(body: Uint8Array, start: number, limit: number): number {
   const target = start + Math.max(1, limit);
   if (target >= body.length) return body.length;
   const safe = (at: number): boolean =>
      ((body[at] ?? 0) & 0xc0) !== 0x80 && !leadsWithParameter(body, at);
   for (let at = target; at > start; at--) if (safe(at)) return at;
   for (let at = target + 1; at < body.length; at++) if (safe(at)) return at;
   return body.length;
}

/** Splits a body into chunk bodies Tern's parser reads back whole (Tern's `tsp_chunks`). */
export function chunks(body: Uint8Array, limit: number): Uint8Array[] {
   const out: Uint8Array[] = [];
   let start = 0;
   while (start < body.length) {
      const end = chunkEnd(body, start, limit);
      out.push(body.subarray(start, end));
      start = end;
   }
   return out;
}

/** Concatenates byte arrays. */
export function concatBytes(parts: readonly Uint8Array[]): Uint8Array {
   let size = 0;
   for (const part of parts) size += part.length;
   const out = new Uint8Array(size);
   let at = 0;
   for (const part of parts) {
      out.set(part, at);
      at += part.length;
   }
   return out;
}

/** One APC string: `ESC _ tsp;<verb>;<params>;<body> ESC \`. */
function apc(verb: string, params: readonly Param[], body: Uint8Array): Uint8Array {
   let head = `${verb};`;
   for (const [key, value] of params) head += `${key}=${value};`;
   return concatBytes([APC_START, encoder.encode(head), body, ST]);
}

/** Frames `body` as one message of `verb`, chunked when it is over the limit. */
export function encodeMessage(
   verb: string,
   body: string | Uint8Array,
   options: EncodeOptions = {}
): Uint8Array {
   const bytes = typeof body === 'string' ? encoder.encode(body) : body;
   const params = options.params ?? [];
   const limit = options.limit ?? DEFAULT_APC;
   if (bytes.length <= limit) return apc(verb, params, bytes);
   const id = options.chunk ? options.chunk() : '0';
   const pieces = chunks(bytes, limit);
   const out: Uint8Array[] = [];
   pieces.forEach((piece, i) => {
      const last = i === pieces.length - 1;
      const head: Param[] = i === 0 ? [...params, ['c', id]] : [['c', id]];
      if (!last) head.push(['m', '1']);
      out.push(apc(verb, head, piece));
   });
   return concatBytes(out);
}

/** Frames a message with a JSON body, written compact. */
export function encodeJson(message: OutMessage, options: EncodeOptions = {}): Uint8Array {
   return encodeMessage(message.verb, JSON.stringify(message.body), options);
}

/** A blob ready to send: its id and the framed `b` message. */
export interface EncodedBlob {
   /** The lowercase hex SHA-256 of the bytes; not sent, Tern hashes them itself. */
   readonly id: string;
   /** The parameters of the message (`mime` when given). */
   readonly params: readonly Param[];
   /** The base64 body. */
   readonly body: string;
   /** The framed message, chunked when needed. */
   readonly bytes: Uint8Array;
}

/** The lowercase hex SHA-256 of `bytes`, the id a blob is named by. */
export function blobId(bytes: Uint8Array): string {
   return createHash('sha256').update(bytes).digest('hex');
}

/** Encodes a blob (`b`); throws a `RangeError` for more than 16 MiB. */
export function encodeBlob(
   bytes: Uint8Array,
   mime?: string,
   options: Omit<EncodeOptions, 'params'> = {}
): EncodedBlob {
   if (bytes.length > MAX_BLOB) {
      throw new RangeError(`a blob is at most ${MAX_BLOB} bytes, got ${bytes.length}`);
   }
   const id = blobId(bytes);
   const params: Param[] = mime === undefined ? [] : [['mime', mime]];
   const body = Buffer.from(bytes.buffer, bytes.byteOffset, bytes.length).toString('base64');
   return { id, params, body, bytes: encodeMessage('b', body, { ...options, params }) };
}

/** The `hello` reply: what the terminal draws and supports. */
export interface HelloReply {
   readonly r: 'hello';
   readonly v: number;
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
   readonly hour12: boolean;
   readonly raw: JsonObject;
}

/** The `blobs` reply: the asked ids Tern holds, in asking order. */
export interface BlobsReply {
   readonly r: 'blobs';
   readonly have: readonly string[];
   readonly raw: JsonObject;
}

/** A reply this SDK doesn't know, kept raw. */
export interface UnknownReply {
   readonly r: 'unknown';
   readonly raw: JsonObject;
}

/** A terminal → program reply (`r`). */
export type Reply = HelloReply | BlobsReply | UnknownReply;

/** A cell's size in pixels. */
export interface CellSize {
   readonly w: number;
   readonly h: number;
}

/** A form control's value in `values`: a radio's pick, a lone checkbox, or checked values. */
export type FormValue = string | boolean | readonly string[] | null;

/** The named controls of a form, as they are at the event. */
export type FormValues = Readonly<Record<string, FormValue>>;

/** A modifier held during a pointer action. */
export type Mod = 'shift' | 'ctrl' | 'alt' | 'meta';

/** Frame `s` drawn: credit returns for every frame up to it. */
export interface AckEvent {
   readonly ev: 'ack';
   readonly sf: string;
   readonly s: number;
   readonly raw: JsonObject;
}

/** The pane's width or cell size changed, or the surface was first drawn. */
export interface ResizeEvent {
   readonly ev: 'resize';
   readonly sf?: string;
   readonly cols: number;
   readonly cell?: CellSize;
   readonly visible?: boolean;
   readonly raw: JsonObject;
}

/** The appearance switched. */
export interface ThemeEvent {
   readonly ev: 'theme';
   readonly dark: boolean;
   readonly raw: JsonObject;
}

/** Reduce Motion was toggled. */
export interface MotionEvent {
   readonly ev: 'motion';
   readonly reduce: boolean;
   readonly raw: JsonObject;
}

/** The pane was hidden or shown. */
export interface VisibleEvent {
   readonly ev: 'visible';
   readonly sf?: string;
   readonly visible: boolean;
   readonly raw: JsonObject;
}

/** A node was folded or unfolded. */
export interface ToggleEvent {
   readonly ev: 'toggle';
   readonly sf: string;
   readonly id: string;
   readonly collapsed: boolean;
   readonly key?: string;
   readonly raw: JsonObject;
}

/** An item was selected (`id` the list, `item` the item). */
export interface SelectEvent {
   readonly ev: 'select';
   readonly sf: string;
   readonly id: string;
   readonly item: string;
   readonly values?: FormValues;
   readonly raw: JsonObject;
}

/** An item was activated (`id` the list, `item` the item). */
export interface ActivateEvent {
   readonly ev: 'activate';
   readonly sf: string;
   readonly id: string;
   readonly item: string;
   readonly values?: FormValues;
   readonly raw: JsonObject;
}

/** A named pointer action ran on a node. */
export interface ActionEvent {
   readonly ev: 'action';
   readonly sf: string;
   readonly id: string;
   readonly act: string;
   readonly value?: string;
   readonly mods?: readonly Mod[];
   readonly values?: FormValues;
   readonly raw: JsonObject;
}

/** An `el` checkbox or radio flipped, or a `prefs` row changed. */
export interface ChangeEvent {
   readonly ev: 'change';
   readonly sf: string;
   readonly id: string;
   readonly value?: Json;
   readonly checked?: boolean;
   readonly name?: string;
   readonly item?: string;
   readonly values?: FormValues;
   readonly raw: JsonObject;
}

/** A click asked for the keys in a field. */
export interface FocusEvent {
   readonly ev: 'focus';
   readonly sf: string;
   readonly id: string;
   readonly raw: JsonObject;
}

/** Native editing: replace UTF-16 `[from, to)` with `text`, caret at `cursor`. */
export interface EditEvent {
   readonly ev: 'edit';
   readonly sf: string;
   readonly id: string;
   readonly from: number;
   readonly to: number;
   readonly text: string;
   readonly cursor: number;
   readonly len: number;
   readonly raw: JsonObject;
}

/** Undo the last change to a field. */
export interface UndoEvent {
   readonly ev: 'undo';
   readonly sf: string;
   readonly id: string;
   readonly raw: JsonObject;
}

/** Submit `text` through a composer's normal path. */
export interface SendEvent {
   readonly ev: 'send';
   readonly sf: string;
   readonly id: string;
   readonly text: string;
   readonly raw: JsonObject;
}

/** Something sent was rejected. */
export interface ErrorEvent {
   readonly ev: 'error';
   readonly sf?: string;
   readonly s?: number;
   readonly op?: number;
   readonly sheet?: string;
   readonly id?: string;
   readonly msg: string;
   readonly raw: JsonObject;
}

/** Tern dropped these nodes or surfaces. */
export interface GoneEvent {
   readonly ev: 'gone';
   readonly sf?: string;
   readonly ids: readonly string[];
   readonly raw: JsonObject;
}

/** An event this SDK doesn't know (or one missing its fields), kept raw. */
export interface UnknownEvent {
   readonly ev: 'unknown';
   readonly sf?: string;
   readonly raw: JsonObject;
}

/** A terminal → program event (`e`). */
export type TspEvent =
   | AckEvent
   | ResizeEvent
   | ThemeEvent
   | MotionEvent
   | VisibleEvent
   | ToggleEvent
   | SelectEvent
   | ActivateEvent
   | ActionEvent
   | ChangeEvent
   | FocusEvent
   | EditEvent
   | UndoEvent
   | SendEvent
   | ErrorEvent
   | GoneEvent
   | UnknownEvent;

/** Whether `value` is a JSON object (not an array). */
export function isJsonObject(value: unknown): value is JsonObject {
   return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function str(raw: JsonObject, key: string): string | undefined {
   const v = raw[key];
   return typeof v === 'string' ? v : undefined;
}

function num(raw: JsonObject, key: string): number | undefined {
   const v = raw[key];
   return typeof v === 'number' ? v : undefined;
}

function bool(raw: JsonObject, key: string): boolean | undefined {
   const v = raw[key];
   return typeof v === 'boolean' ? v : undefined;
}

function strings(raw: JsonObject, key: string): string[] | undefined {
   const v = raw[key];
   if (!Array.isArray(v)) return undefined;
   const out: string[] = [];
   for (const item of v) if (typeof item === 'string') out.push(item);
   return out;
}

function cell(raw: JsonObject): CellSize | undefined {
   const v = raw['cell'];
   if (!isJsonObject(v)) return undefined;
   const w = num(v, 'w');
   const h = num(v, 'h');
   return w !== undefined && h !== undefined ? { w, h } : undefined;
}

function formValues(raw: JsonObject): FormValues | undefined {
   const v = raw['values'];
   if (!isJsonObject(v)) return undefined;
   const out: Record<string, FormValue> = {};
   for (const key in v) {
      const value = v[key];
      if (value === undefined) continue;
      if (typeof value === 'string' || typeof value === 'boolean' || value === null) {
         out[key] = value;
      } else if (Array.isArray(value)) {
         out[key] = value.filter(x => typeof x === 'string');
      }
   }
   return out;
}

function mods(raw: JsonObject): Mod[] | undefined {
   const list = strings(raw, 'mods');
   if (!list) return undefined;
   return list.filter(m => m === 'shift' || m === 'ctrl' || m === 'alt' || m === 'meta');
}

/** Copies `fields` onto `target`, leaving out the undefined ones. */
function defined<T extends object>(target: T, fields: Record<string, unknown>): T {
   for (const key in fields) {
      const value = fields[key];
      if (value !== undefined) Reflect.set(target, key, value);
   }
   return target;
}

/** Decodes an event body; unknown events, and events missing a field they need, stay raw. */
export function decodeEvent(raw: JsonObject): TspEvent {
   const ev = str(raw, 'ev');
   const sf = str(raw, 'sf');
   const id = str(raw, 'id');
   const unknown = (): UnknownEvent => defined({ ev: 'unknown', raw }, { sf });
   switch (ev) {
      case 'ack': {
         const s = num(raw, 's');
         return sf !== undefined && s !== undefined ? { ev, sf, s, raw } : unknown();
      }
      case 'resize': {
         const cols = num(raw, 'cols');
         if (cols === undefined) return unknown();
         return defined<ResizeEvent>(
            { ev, cols, raw },
            { sf, cell: cell(raw), visible: bool(raw, 'visible') }
         );
      }
      case 'theme': {
         const dark = bool(raw, 'dark');
         return dark !== undefined ? { ev, dark, raw } : unknown();
      }
      case 'motion': {
         const reduce = bool(raw, 'reduce');
         return reduce !== undefined ? { ev, reduce, raw } : unknown();
      }
      case 'visible': {
         const visible = bool(raw, 'visible');
         return visible !== undefined
            ? defined<VisibleEvent>({ ev, visible, raw }, { sf })
            : unknown();
      }
      case 'toggle': {
         const collapsed = bool(raw, 'collapsed');
         if (sf === undefined || id === undefined || collapsed === undefined) return unknown();
         return defined<ToggleEvent>({ ev, sf, id, collapsed, raw }, { key: str(raw, 'key') });
      }
      case 'select':
      case 'activate': {
         const item = str(raw, 'item') ?? id;
         if (sf === undefined || id === undefined || item === undefined) return unknown();
         return defined<SelectEvent | ActivateEvent>(
            { ev, sf, id, item, raw },
            { values: formValues(raw) }
         );
      }
      case 'action': {
         const act = str(raw, 'act');
         if (sf === undefined || id === undefined || act === undefined) return unknown();
         return defined<ActionEvent>(
            { ev, sf, id, act, raw },
            { value: str(raw, 'value'), mods: mods(raw), values: formValues(raw) }
         );
      }
      case 'change': {
         if (sf === undefined || id === undefined) return unknown();
         return defined<ChangeEvent>(
            { ev, sf, id, raw },
            {
               value: raw['value'],
               checked: bool(raw, 'checked'),
               name: str(raw, 'name'),
               item: str(raw, 'item'),
               values: formValues(raw),
            }
         );
      }
      case 'focus':
      case 'undo':
         return sf !== undefined && id !== undefined ? { ev, sf, id, raw } : unknown();
      case 'edit': {
         const from = num(raw, 'from');
         const to = num(raw, 'to');
         const text = str(raw, 'text');
         const cursor = num(raw, 'cursor');
         const len = num(raw, 'len');
         if (
            sf === undefined ||
            id === undefined ||
            from === undefined ||
            to === undefined ||
            text === undefined ||
            cursor === undefined ||
            len === undefined
         ) {
            return unknown();
         }
         return { ev, sf, id, from, to, text, cursor, len, raw };
      }
      case 'send': {
         const text = str(raw, 'text');
         return sf !== undefined && id !== undefined && text !== undefined
            ? { ev, sf, id, text, raw }
            : unknown();
      }
      case 'error':
         return defined<ErrorEvent>(
            { ev, msg: str(raw, 'msg') ?? '', raw },
            { sf, s: num(raw, 's'), op: num(raw, 'op'), sheet: str(raw, 'sheet'), id }
         );
      case 'gone':
         return defined<GoneEvent>({ ev, ids: strings(raw, 'ids') ?? [], raw }, { sf });
      default:
         return unknown();
   }
}

/** Decodes a reply body; unknown replies stay raw. */
export function decodeReply(raw: JsonObject): Reply {
   switch (str(raw, 'r')) {
      case 'hello':
         return {
            r: 'hello',
            v: num(raw, 'v') ?? PROTOCOL_VERSION,
            term: str(raw, 'term') ?? '',
            ver: str(raw, 'ver') ?? '',
            kinds: strings(raw, 'kinds') ?? [...KINDS],
            features: strings(raw, 'features') ?? [],
            apc: num(raw, 'apc') ?? DEFAULT_APC,
            credits: num(raw, 'credits') ?? DEFAULT_CREDITS,
            cols: num(raw, 'cols') ?? 80,
            cell: cell(raw) ?? { w: 8, h: 16 },
            dark: bool(raw, 'dark') ?? true,
            reduceMotion: bool(raw, 'reduceMotion') ?? false,
            hour12: bool(raw, 'hour12') ?? false,
            raw,
         };
      case 'blobs':
         return { r: 'blobs', have: strings(raw, 'have') ?? [], raw };
      default:
         return { r: 'unknown', raw };
   }
}

/** One TSP message split into verb, parameters and body (Tern's `split`). */
export interface SplitMessage {
   readonly verb: string;
   readonly params: readonly Param[];
   readonly body: Uint8Array;
}

const decoder = new TextDecoder();

/** Splits the data after `tsp;` into verb, parameters and body; `null` when it has no verb. */
export function splitMessage(inner: Uint8Array): SplitMessage | null {
   const semi = inner.indexOf(0x3b);
   if (semi < 0) {
      return inner.length > 0
         ? { verb: decoder.decode(inner), params: [], body: new Uint8Array() }
         : null;
   }
   if (semi === 0) return null;
   const verb = decoder.decode(inner.subarray(0, semi));
   const params: Param[] = [];
   let pos = semi + 1;
   for (;;) {
      const len = inner.subarray(pos).indexOf(0x3b);
      if (len < 0) break;
      const segment = inner.subarray(pos, pos + len);
      const eq = segment.indexOf(0x3d);
      if (eq <= 0) break;
      const key = segment.subarray(0, eq);
      const value = segment.subarray(eq + 1);
      if (!key.every(keyByte) || !value.every(valueByte)) break;
      params.push([decoder.decode(key), decoder.decode(value)]);
      pos += len + 1;
   }
   return { verb, params, body: inner.subarray(pos) };
}
