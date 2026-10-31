/**
 * Typed props of every kind (except Tern's own `block`), the props every
 * node takes, and the enumerations they use. Every enumeration keeps a
 * `(string & {})` escape hatch for values newer than the SDK.
 */

import type { Node, NodeJson, Span } from './nodes.js';
import type { Json } from './wire.js';

/** A node (any TSX element) drawn in a panel's slot, sent as the child keyed by the slot. */
export type SlotNode = Node | Span;

/** The semantic color of a node's chrome. */
export type Tone =
   | 'neutral'
   | 'accent'
   | 'info'
   | 'success'
   | 'warning'
   | 'error'
   | 'pending'
   | 'muted'
   | 'user'
   | (string & {});

/** The state of a card or tool. */
export type Status = 'pending' | 'running' | 'done' | 'error' | 'cancelled' | (string & {});

/** The state of an agent. */
export type AgentStatus =
   | 'pending'
   | 'running'
   | 'done'
   | 'failed'
   | 'aborted'
   | 'idle'
   | 'parked'
   | (string & {});

/** One span style token (a style string is space-separated tokens). */
export type SpanToken =
   | 'muted'
   | 'dim'
   | 'strong'
   | 'em'
   | 'accent'
   | 'success'
   | 'warning'
   | 'error'
   | 'info'
   | 'pending'
   | 'code'
   | 'mono'
   | 'path'
   | 'key'
   | 'link'
   | 'num'
   | 'ins'
   | 'del'
   | 'mark'
   | 'typo'
   | 'icon'
   | 'hide'
   | 'title'
   | (string & {});

/** A span effect. */
export type SpanFx = 'shimmer' | 'pulse' | 'none' | (string & {});

/** One styled run of text. */
export interface SpanData {
   /** The text. */
   readonly t: string;
   /** Space-separated style tokens. */
   readonly s?: SpanToken;
   /** An effect. */
   readonly fx?: SpanFx;
   /** Makes the span a link. */
   readonly href?: string;
   /** The sources the run cites: a marker follows it, drawn as the `text`'s `cite` look says. */
   readonly cite?: Cite | readonly Cite[];
}

/** A source a span cites. */
export interface Cite {
   /** Where the source is; opened on click. A cite without one is ignored. */
   readonly href: string;
   /** The source's title (hover card, sources list, accessible name). */
   readonly title?: string;
   /** The number shown (an integer ≥ 1); by default the next free one, in order of first appearance. */
   readonly n?: number;
   /** The name a pill shows (default the `href`'s host, `www.` dropped). */
   readonly site?: string;
   /** A quote shown in the hover card, two lines at most. */
   readonly snippet?: string;
}

/** Text given as a string or as a list of strings and styled spans. */
export type Spans = string | readonly (string | SpanData)[];

/** Tern's named icons and their aliases. The app icons (`app-vim`, `app-python`, …)
    follow Tern's catalog of programs, so they aren't listed: any string passes. */
export type IconName =
   | 'logo'
   | 'tern'
   | 'pi-mark'
   | 'arrow-up'
   | 'arrow-down'
   | 'arrow-left'
   | 'arrow-right'
   | 'back'
   | 'forward'
   | 'chev'
   | 'chev-r'
   | 'chev-up'
   | 'corner-down-right'
   | 'expand'
   | 'shrink'
   | 'minimize'
   | 'fit'
   | 'zoom-in'
   | 'zoom-out'
   | 'more'
   | 'grip'
   | 'compass'
   | 'check'
   | 'x'
   | 'x-circle'
   | 'warn'
   | 'info'
   | 'help'
   | 'bell'
   | 'shield'
   | 'shield-alert'
   | 'slash'
   | 'slash-circle'
   | 'lock'
   | 'key'
   | 'key-round'
   | 'plus'
   | 'minus'
   | 'clock'
   | 'timer'
   | 'doc'
   | 'file'
   | 'file-code'
   | 'file-image'
   | 'file-pdf'
   | 'file-plus'
   | 'files'
   | 'folder'
   | 'folder-go'
   | 'folder-minus'
   | 'folder-open'
   | 'folder-plus'
   | 'markdown'
   | 'note'
   | 'clipboard'
   | 'save'
   | 'inbox'
   | 'image'
   | 'newspaper'
   | 'trash'
   | 'branch'
   | 'commit'
   | 'merge'
   | 'rebase'
   | 'pull'
   | 'push'
   | 'fetch'
   | 'stash'
   | 'cherry'
   | 'diff'
   | 'unified'
   | 'split'
   | 'split-down'
   | 'revert'
   | 'history'
   | 'play'
   | 'pause'
   | 'stop'
   | 'rewind'
   | 'fast-forward'
   | 'frame-next'
   | 'frame-prev'
   | 'repeat'
   | 'redo'
   | 'undo'
   | 'run'
   | 'reel'
   | 'mic'
   | 'wave'
   | 'vibrate'
   | 'power'
   | 'send'
   | 'share'
   | 'open'
   | 'download'
   | 'copy'
   | 'scissors'
   | 'bold'
   | 'italic'
   | 'underline'
   | 'type'
   | 'align-left'
   | 'align-center'
   | 'align-right'
   | 'align-justify'
   | 'pen'
   | 'eraser'
   | 'wand'
   | 'cursor'
   | 'selection'
   | 'path-insert'
   | 'prompt'
   | 'vector'
   | 'blur'
   | 'ink'
   | 'palette'
   | 'columns'
   | 'grid'
   | 'sidebar'
   | 'tabs-h'
   | 'tabs-v'
   | 'dock-up'
   | 'dock-down'
   | 'dock-left'
   | 'dock-right'
   | 'pip'
   | 'pip-tl'
   | 'pip-tr'
   | 'pip-bl'
   | 'pip-exit'
   | 'layers'
   | 'stack'
   | 'kanban'
   | 'canvas'
   | 'diagram'
   | 'flow'
   | 'peek'
   | 'tree'
   | 'box'
   | 'eye'
   | 'eye-off'
   | 'terminal'
   | 'code'
   | 'braces'
   | 'binary'
   | 'bug'
   | 'cpu'
   | 'database'
   | 'server'
   | 'laptop'
   | 'monitor'
   | 'gauge'
   | 'activity'
   | 'chart'
   | 'plug'
   | 'puzzle'
   | 'wrench'
   | 'gear'
   | 'sliders'
   | 'stethoscope'
   | 'hash'
   | 'tag'
   | 'link'
   | 'globe'
   | 'broadcast'
   | 'search'
   | 'funnel'
   | 'funnel-x'
   | 'sort-x'
   | 'swap'
   | 'list'
   | 'list-checks'
   | 'pin'
   | 'keyboard'
   | 'user'
   | 'users'
   | 'message'
   | 'brain'
   | 'sparkle'
   | 'lightbulb'
   | 'bolt'
   | 'flame'
   | 'rocket'
   | 'moon'
   | 'sun'
   | 'cart'
   | 'footprints'
   | 'scale'
   | 'log-in'
   | 'log-out'
   | 'success'
   | 'done'
   | 'error'
   | 'failed'
   | 'warning'
   | 'pending'
   | 'running'
   | 'aborted'
   | 'cancelled'
   | 'disabled'
   | 'model'
   | 'plan'
   | 'goal'
   | 'loop'
   | 'git'
   | 'pr'
   | 'tokens'
   | 'speculation'
   | 'compaction'
   | 'context'
   | 'cost'
   | 'time'
   | 'omp'
   | 'agents'
   | 'agent'
   | 'job'
   | 'cache'
   | 'input'
   | 'output'
   | 'host'
   | 'session'
   | 'worktree'
   | 'bash'
   | 'shell'
   | 'python'
   | 'edit'
   | 'write'
   | 'read'
   | 'web'
   | 'url'
   | 'todo'
   | 'task'
   | 'thinking'
   | 'settings'
   | 'appearance'
   | 'interaction'
   | 'memory'
   | 'tools'
   | 'tasks'
   | 'providers'
   | 'plugins'
   | 'advisor'
   | 'git-branch'
   | 'sparkles'
   | 'action'
   | 'extension'
   | 'computer'
   | 'stats'
   | 'news'
   | 'export'
   | 'restart'
   | 'compress'
   | 'handoff'
   | 'question'
   | 'pencil'
   | 'folderMove'
   | 'folderPlus'
   | 'folderMinus'
   | 'hammer'
   | 'prewalk'
   | 'jobs'
   | 'signIn'
   | 'signOut'
   | 'package'
   | 'fast'
   | 'voice'
   | 'rule'
   | 'skill'
   | 'mcp'
   | 'path'
   | 'token_total'
   | 'time_spent'
   | (string & {});

/** A size bound: `"<n>ch"` cells, `"<n>lines"` text lines, or a fraction of the parent. */
export type Extent = `${number}ch` | `${number}lines` | number;

/** Width and height bounds. */
export interface Bounds {
   readonly w?: Extent;
   readonly h?: Extent;
}

/** A row of a menu: a `menu`'s, a `menubar`'s or a right-click `actions.menu`. */
export interface MenuItem {
   /** What the `action` event sends as `act`. */
   readonly id: string;
   /** The row's text (default from `id`: `open-file` reads "Open file"). */
   readonly label?: string;
   readonly icon?: IconName;
   /** Shortcut keys drawn at the right (`['cmd', 'shift', 'C']` reads ⌘⇧C), display only. */
   readonly keys?: string | readonly string[];
   /** A second line under the label (a rich row); a right-click menu shows it as the tooltip. */
   readonly detail?: string;
   /** A check row, checked while `true`; choosing it sends the new state as `checked`. */
   readonly checked?: boolean;
   /** A radio row of this group: a dot while `checked`; choosing it turns the group's others off. */
   readonly radio?: string;
   /** Dimmed, not choosable; a string is the tooltip reason. */
   readonly disabled?: boolean | string;
   /** A destructive action, drawn in the error color. */
   readonly danger?: boolean;
   /** A submenu, opening beside the row; the row itself sends nothing. */
   readonly items?: readonly MenuEntry[];
}

/** A menu entry: `"-"` a separator, `{ header }` a section title, an action id, or an item. */
export type MenuEntry = string | { readonly header: string } | MenuItem;

/** What a pointer does on a node. */
export interface Actions {
   readonly click?: string;
   readonly dblclick?: string;
   /** The right-click menu: action ids, or rows with icons, keys, checks and submenus. */
   readonly menu?: readonly MenuEntry[];
}

/** The props every node takes. */
export interface CommonProps {
   /** Identity among siblings; the node's id derives from it. */
   readonly key?: string | number;
   /** A name of yours, `data-role` for stylesheets. */
   readonly role?: string;
   readonly tone?: Tone;
   /** Mounted but not laid out. */
   readonly hidden?: boolean;
   /** A transient selection mark. */
   readonly mark?: 'pick' | 'drop' | (string & {});
   readonly actions?: Actions;
   /** A tooltip. */
   readonly title?: string;
   /**
    * A floating card shown when the pointer rests on the node, in place of
    * `title` while open: an id-less node tree (a `Node` is sent as its JSON;
    * its handlers, ids and keys are ignored). `null` removes it. An older
    * Tern ignores it.
    */
   readonly hovercard?: Node | NodeJson | null;
   /** The accessible name of a node that shows no text. */
   readonly aria?: string;
   /** A link for the node. */
   readonly href?: string;
   readonly grow?: number;
   readonly shrink?: number;
   /** A fraction of the parent, or `content` for the node's own size. */
   readonly basis?: number | 'content';
   readonly min?: Bounds;
   readonly max?: Bounds;
}

/** Space between a stack's children. */
export type Gap = 'none' | 'xs' | 'sm' | 'md' | 'lg' | (string & {});

/** Cross-axis alignment. */
export type Align = 'start' | 'center' | 'end' | 'baseline' | 'stretch' | (string & {});

/** Props of `col` and `row`. */
export interface StackProps {
   readonly gap?: Gap;
   readonly align?: Align;
   readonly justify?: 'between' | 'end' | (string & {});
   readonly wrap?: boolean;
}

/** Props of `card`. */
export interface CardProps {
   /** The head text, or a node drawn as the head (sent as the child keyed `head`). */
   readonly head?: Spans | SlotNode;
   /** The head icon; wins over the one the role implies. */
   readonly icon?: IconName;
   /** Muted text after the head text. */
   readonly meta?: Spans;
   /**
    * A node drawn at the head's end instead of the body (buttons, segmented
    * tabs, badges); sent as the child keyed `end`, or a child's id.
    */
   readonly end?: SlotNode | string;
   readonly status?: Status;
   readonly collapsible?: boolean;
   readonly collapsed?: boolean;
   readonly preview?: 'auto' | { readonly lines: number };
   readonly selected?: boolean;
   readonly inset?: boolean;
   readonly variant?: 'bare' | (string & {});
}

/** Props of `section`. */
export interface SectionProps {
   /** The head text, or a node drawn as the head (sent as the child keyed `head`). */
   readonly head?: Spans | SlotNode;
   readonly collapsible?: boolean;
   readonly collapsed?: boolean;
}

/** Props of `rule`. */
export interface RuleProps {
   /** `line` (default), `note` a framed row, `pin` a pill on the line. */
   readonly variant?: 'line' | 'note' | 'pin' | (string & {});
   readonly label?: Spans;
   /** An icon before the label. */
   readonly icon?: IconName;
   /** A time or short stamp, in mono. */
   readonly time?: string;
   /**
    * A node drawn at the end (after the label, at a note's right, beside a
    * pin); sent as the child keyed `end`, or a child's id. Other children
    * are not drawn.
    */
   readonly end?: SlotNode | string;
}

/** Props of `spacer`. */
export interface SpacerProps {
   readonly size?: Gap;
}

/** Where overflowing text is cut. */
export type Truncate = 'end' | 'start' | 'middle' | (string & {});

/** Props of `text`. */
export interface TextProps {
   /** Plain text, one unstyled span; used only when `spans` is absent. */
   readonly text?: string;
   /** The styled runs; wins over `text`. */
   readonly spans?: readonly (string | SpanData)[];
   /** `word` (default) breaks between words, `char` anywhere, `none` keeps each line whole. */
   readonly wrap?: 'word' | 'char' | 'none' | (string & {});
   /** Where overflowing text is cut (default `end`); alone it keeps the text on one line. */
   readonly truncate?: Truncate;
   /** Clamp to this many visual lines, the last ending in `…`; `0` is no clamp. */
   readonly lines?: number;
   /** `prose` caps the line length at 80 cells for readable paragraphs. */
   readonly measure?: 'prose' | (string & {});
   /** How cited spans look: `chip` (default) numbered chips, `pill` the source's site, `note` dotted runs with superscripts. */
   readonly cite?: 'chip' | 'pill' | 'note' | (string & {});
   /** Draws the numbered sources under the text (default on for `note`); not drawn when the text is fitted. */
   readonly sources?: boolean;
}

/** Props of `md`. */
export interface MdProps {
   readonly text?: string;
   /** The source is still arriving. */
   readonly stream?: boolean;
   readonly marks?: readonly (string | SpanData)[];
}

/** A marked line of a `code` node. */
export interface CodeMark {
   readonly line: number;
   readonly tone?: Tone;
   /** A score 0–1 on the heat ramp: the overview's bar and the fold sparkline, and the row's wash without `wash`; without `tone`, no bar. */
   readonly heat?: number;
   /** Byte ranges `[start, end)` in the line. */
   readonly ranges?: readonly (readonly [number, number])[];
   /** Scores 0–1 washing the row on the heat ramp: one flat, several a gradient across it, left to right; in place of `heat`'s wash; without `tone`, no bar. */
   readonly wash?: readonly number[];
}

/** A run of lines a `code` node draws as one row until it is opened. */
export interface CodeFold {
   /** First and last folded line, in the gutter's numbering. */
   readonly from: number;
   readonly to: number;
   /** The folded row's text, in the code's face (default `… [lines {from}-{to}]`, `… [line {from}]` for one line). */
   readonly label?: string;
   /** What the fold hides, after a sparkline of its lines' mark `heat`. */
   readonly note?: Spans;
   /** `error` tints the row; any tone colors the note. */
   readonly tone?: Tone;
   /** Drawn open; a click on the folded row (or an open fold's head) toggles it in place. */
   readonly open?: boolean;
   /** A card shown while the pointer rests on the folded row, as the common `hovercard`. */
   readonly hovercard?: Node | NodeJson;
}

/** Props of `code`. */
export interface CodeProps {
   readonly text?: string;
   readonly lang?: string;
   readonly path?: string;
   readonly numbers?: boolean;
   readonly start?: number;
   /** Each line's number, strictly ascending, in place of counting from `start`; a `…` row where they jump. */
   readonly nums?: readonly number[];
   readonly marks?: readonly CodeMark[];
   readonly wrap?: boolean;
   /** Runs of lines drawn folded: sorted, not overlapping; marks inside apply once a fold is open. */
   readonly folds?: readonly CodeFold[];
   /**
    * An overview strip above the lines: one bar per line (its height the
    * mark's `heat`, its color the mark's tone), a band over the lines outside
    * folds and a bracket over the lines in view. A click or drag scrolls there.
    */
   readonly overview?: boolean;
}

/** One hunk of a `diff`. */
export interface DiffHunk {
   readonly oldStart?: number;
   readonly newStart?: number;
   readonly lines: readonly string[];
}

/** Props of `diff`. */
export interface DiffProps {
   readonly text?: string;
   readonly hunks?: readonly DiffHunk[];
   readonly path?: string;
   readonly lang?: string;
   readonly mode?: 'unified' | 'split' | 'auto' | (string & {});
}

/** Props of `ansi`. */
export interface AnsiProps {
   readonly text?: string;
   readonly cols?: number;
   readonly preview?: { readonly lines: number };
   readonly follow?: boolean;
}

/** Props of `rows`. */
export interface RowsProps {
   readonly lines?: readonly string[];
   readonly cols?: number;
}

/** Props of `math`. */
export interface MathProps {
   readonly text?: string;
   readonly display?: boolean;
}

/** Props of `image`. */
export interface ImageProps {
   /** A blob id. */
   readonly blob?: string;
   readonly builtin?: 'omp' | (string & {});
   readonly alt?: string;
   readonly w?: number;
   readonly h?: number;
   readonly path?: string;
}

/** Props of `media`: a file Tern loads by path and shows in its media viewer. */
export interface MediaProps {
   /** The file: absolute, or relative to the pane's working directory. */
   readonly path: string;
   /** Read the file from git instead of disk (`HEAD`, a sha, `:0` for the index). */
   readonly rev?: string;
   /** `media` nodes sharing this link pan and zoom together. */
   readonly link?: string;
}

/** One pair of a `kv`. */
export interface KvItem {
   readonly k: Spans;
   readonly v: Spans;
}

/** Props of `kv`. */
export interface KvProps {
   readonly items?: readonly KvItem[];
   readonly layout?: 'grid' | 'inline' | (string & {});
}

/** One stacked part of a meter. */
export interface MeterPart {
   readonly value: number;
   readonly token?: string;
   readonly label?: string;
   readonly hatch?: boolean;
}

/** Warning and danger levels of a meter. */
export interface Thresholds {
   readonly warn?: number;
   readonly bad?: number;
}

/** A table cell drawn as a meter. */
export interface MeterCell {
   readonly meter: {
      readonly value?: number;
      readonly parts?: readonly MeterPart[];
      readonly thresholds?: Thresholds;
      readonly tone?: Tone;
      readonly title?: string;
   };
}

/** A table cell of text with a leading status dot or icon. */
export interface TextCell {
   readonly text: Spans;
   readonly icon?: IconName;
   /** A status dot in this tone's color. */
   readonly dot?: Tone;
   /** The icon's color. */
   readonly tone?: Tone;
   readonly title?: string;
}

/** A table cell drawn as a badge. */
export interface BadgeCell {
   readonly badge: { readonly text: string; readonly tone?: Tone; readonly title?: string };
}

/** A table cell drawn as a small button; a click sends `act` with the row as `item`. */
export interface ButtonCell {
   readonly button: {
      readonly text: string;
      readonly act: string;
      readonly icon?: IconName;
      readonly keys?: readonly string[];
      readonly variant?: ButtonVariant;
      readonly disabled?: boolean | string;
   };
}

/** A table cell drawn as a sparkline. */
export interface SparkCell {
   readonly spark: {
      readonly values: readonly number[];
      /** Default `accent`. */
      readonly tone?: Tone;
      readonly min?: number;
      readonly max?: number;
      readonly title?: string;
   };
}

/** A cell of a `table` row. */
export type TableCell = Spans | MeterCell | TextCell | BadgeCell | ButtonCell | SparkCell;

/** A column of a `table`. */
export interface TableCol {
   readonly id: string;
   readonly head?: Spans;
   /** `end` columns get tabular numbers. */
   readonly align?: 'start' | 'center' | 'end' | (string & {});
   /** Cells in the mono face, for machine strings (ids, paths, numbers). */
   readonly mono?: boolean;
   readonly truncate?: Truncate;
   readonly priority?: number;
   readonly grow?: number;
   /** Its header takes clicks and sends `action` `sort` with `{ col, dir }` as `value`. */
   readonly sortable?: boolean;
}

/** A button of a `table`'s selection bar: a small `button`. */
export interface TableBarAction {
   /** What a click sends, with the picked ids as `value` (default `click`). */
   readonly act?: string;
   readonly text?: string;
   readonly icon?: IconName;
   readonly keys?: readonly string[];
   readonly variant?: ButtonVariant;
   /** Dimmed and sends nothing; a string is the tooltip reason. */
   readonly disabled?: boolean | string;
   readonly title?: string;
}

/** A row of a `table`: one cell per column id. */
export interface TableRow {
   readonly id: string;
   readonly cells: Readonly<Record<string, TableCell>>;
   /** Dimmed and never selected or activated; a string is the tooltip reason. */
   readonly disabled?: boolean | string;
   /** Right-click actions; each sends `action` with the row as `item`. */
   readonly menu?: readonly string[];
   /** The row's tooltip. */
   readonly title?: string;
}

/** Props of `table`. */
export interface TableProps {
   readonly cols?: readonly TableCol[];
   readonly rows?: readonly TableRow[];
   /** Rows take clicks: click sends `select`, double-click `activate`, with the row as `item`. */
   readonly selectable?: boolean;
   /** The id of the row drawn selected. */
   readonly selected?: string;
   /** Shown when there are no rows. */
   readonly empty?: Spans;
   /** The column the rows are sorted by, drawn with its arrow; a change wins over the user's last header click. */
   readonly sort?: { readonly col: string; readonly dir: 'asc' | 'desc' | (string & {}) };
   /** Rows are picked several at a time: a click toggles one, a shift-click a range; each change sends `change`. */
   readonly multi?: boolean;
   /** The ids of the picked rows (with `multi`); a change wins over the user's clicks. */
   readonly picked?: readonly string[];
   /** How picks and the sort look: `check` (default) a checkbox column, `bar` an accent edge. */
   readonly variant?: 'check' | 'bar' | (string & {});
   /** Buttons of the selection bar, left to right. */
   readonly bar?: readonly TableBarAction[];
   /** Muted text after the count in the selection bar, such as the keys the program takes. */
   readonly hint?: Spans;
}

/** An item of a `tree`. */
export interface TreeItem {
   readonly id: string;
   readonly label?: Spans;
   readonly icon?: IconName;
   readonly open?: boolean;
   readonly children?: readonly TreeItem[];
}

/** Props of `tree`. */
export interface TreeProps {
   readonly nodes?: readonly TreeItem[];
}

/** Props of `json`. */
export interface JsonProps {
   /** The JSON value to draw; absent reads as null. */
   readonly value?: Json;
   /** How many levels of containers start expanded (default 0). */
   readonly open?: number;
}

/** A button's chrome. */
export type ButtonVariant = 'default' | 'primary' | 'quiet' | 'danger' | (string & {});

/** A button's size: `md` 28px for bars, heads and forms; `sm` 22px inline and in rows. */
export type ButtonSize = 'sm' | 'md' | (string & {});

/** Props of `button`: a click sends `action` with `actions.click`, else `"click"`. */
export interface ButtonProps {
   /** The label; string children become it. */
   readonly text?: string;
   /** A leading icon; with no `text`, an icon-only button (give it `title` or `aria`). */
   readonly icon?: IconName;
   /** Keycaps drawn after the label, display only. */
   readonly keys?: readonly string[];
   /** Default `default`; one `primary` per view. */
   readonly variant?: ButtonVariant;
   /** Default `md`. */
   readonly size?: ButtonSize;
   /** A pressed toggle. */
   readonly on?: boolean;
   /** Dimmed and sends nothing; a string is the tooltip reason. */
   readonly disabled?: boolean | string;
   /** A spinner in place of the icon; sends nothing. */
   readonly busy?: boolean;
}

/** One tile of a `stats` well. */
export interface StatItem {
   readonly label: Spans;
   readonly value: Spans;
   readonly sub?: Spans;
   /** Colors the value. */
   readonly tone?: Tone;
   readonly title?: string;
}

/** Props of `stats`: a well of stat tiles side by side. */
export interface StatsProps {
   readonly items?: readonly StatItem[];
}

/** Props of `badge`. */
export interface BadgeProps {
   readonly text?: string;
}

/** Props of `kbd`. */
export interface KbdProps {
   readonly keys?: readonly string[];
}

/** Props of `icon`. */
export interface IconProps {
   readonly name?: IconName;
}

/** Props of `list`. */
export interface ListProps {
   /** The node id of the selected item. */
   readonly selected?: string;
   readonly filter?: string;
   readonly empty?: Spans;
   readonly max?: { readonly lines: number } | number;
   readonly virtual?: boolean;
}

/** Props of `item`. */
export interface ItemProps {
   readonly label?: Spans;
   readonly detail?: Spans;
   readonly value?: Spans;
   readonly icon?: IconName;
   readonly hint?: readonly string[];
   readonly disabled?: boolean;
}

/** A tab of a `tabs` strip. */
export interface Tab {
   readonly id: string;
   readonly label: Spans;
}

/** Props of `tabs`. */
export interface TabsProps {
   readonly items?: readonly Tab[];
   readonly active?: string;
   /** `underline` (default) or `segmented`, the inline well for view switches. */
   readonly variant?: 'underline' | 'segmented' | (string & {});
}

/** A row of a `picker`'s catalog. */
export interface PickerItem {
   readonly id: string;
   readonly label?: Spans;
   readonly detail?: Spans;
   readonly icon?: IconName;
   readonly role?: string;
   readonly mark?: { readonly text: string; readonly seed?: string };
   readonly dot?: Tone;
   readonly mono?: boolean;
   readonly facts?: Readonly<Record<string, number | Spans>>;
   readonly badges?: readonly {
      readonly text: string;
      readonly tone?: Tone;
      readonly title?: string;
   }[];
   readonly chips?: readonly {
      readonly text: string;
      readonly on?: boolean;
      readonly auto?: boolean;
      readonly dot?: string;
   }[];
   readonly tone?: Tone;
   readonly disabled?: true | string;
   readonly hits?: readonly (readonly [number, number])[];
   readonly node?: string;
   readonly depth?: number;
   readonly open?: boolean;
   readonly title?: string;
}

/** A group header in a picker's `order`. */
export interface PickerGroup {
   readonly group: string;
   readonly label: string;
   readonly count?: number;
}

/** A fact column of a `picker`. */
export interface PickerColumn {
   readonly id: string;
   readonly head?: string;
   readonly format?: 'text' | 'num' | 'price' | 'bar' | 'time' | 'elapsed' | 'dim' | (string & {});
   readonly priority?: number;
   readonly min?: number;
}

/** A scope of a `picker`'s scope column. */
export interface PickerScope {
   readonly id: string;
   readonly label: Spans;
   readonly group?: string;
   readonly mark?: { readonly text: string; readonly seed?: string };
   readonly icon?: IconName;
   readonly dot?: Tone;
   readonly count?: number;
   readonly disabled?: true | string;
}

/** A button of a `picker`'s action bar. */
export interface PickerAction {
   readonly id: string;
   readonly label?: string;
   readonly keys?: readonly string[];
   readonly primary?: boolean;
   readonly end?: boolean;
   readonly danger?: boolean;
   readonly on?: boolean;
   readonly disabled?: true | string;
}

/** Props of `picker`. */
export interface PickerProps {
   readonly size?: 'md' | 'lg' | 'screen' | (string & {});
   readonly layout?: 'rows' | 'cards' | 'timeline' | 'tree' | (string & {});
   readonly preview?: 'side' | 'below' | 'none' | (string & {});
   readonly title?: Spans;
   readonly subtitle?: Spans;
   readonly icon?: IconName;
   readonly query?: string;
   readonly cursor?: number;
   readonly placeholder?: string;
   readonly noun?: string;
   readonly focus?: 'list' | 'scopes' | 'tabs' | 'strip' | 'preview' | (string & {});
   readonly state?: 'ready' | 'loading' | 'error' | (string & {});
   readonly message?: Spans;
   readonly empty?: Spans;
   readonly total?: number;
   readonly items?: readonly PickerItem[];
   readonly itemsAdd?: readonly PickerItem[];
   readonly itemsDel?: readonly string[];
   readonly order?: readonly (string | PickerGroup)[];
   readonly selected?: string;
   readonly current?: readonly string[];
   readonly hits?: Readonly<Record<string, readonly (readonly [number, number])[]>>;
   readonly columns?: readonly PickerColumn[];
   readonly confirm?: { readonly text: string; readonly act?: string; readonly label?: string };
   readonly scopes?: readonly PickerScope[];
   readonly scope?: string;
   readonly tabs?: readonly {
      readonly id: string;
      readonly label: Spans;
      readonly count?: number;
   }[];
   readonly tab?: string;
   readonly strip?: {
      readonly label?: string;
      readonly items: readonly {
         readonly id: string;
         readonly label: Spans;
         readonly on?: boolean;
         readonly dot?: string;
      }[];
      readonly selected?: string;
   };
   readonly actions?: readonly PickerAction[];
}

/** Props of `spinner`. */
export interface SpinnerProps {
   readonly style?: 'braille' | 'dots' | 'starburst' | 'orbit' | (string & {});
   readonly label?: Spans;
}

/** Props of `shimmer`. */
export interface ShimmerProps {
   readonly text?: string;
   readonly spans?: readonly (string | SpanData)[];
   readonly mode?: 'classic' | 'kitt' | (string & {});
   readonly palette?: {
      readonly low?: SpanToken;
      readonly mid?: SpanToken;
      readonly high?: SpanToken;
   };
}

/** Props of `elapsed`. */
export interface ElapsedProps {
   /** Milliseconds already elapsed when the node was built; negative counts down. */
   readonly age?: number;
   /** Freezes the display at this many ms. */
   readonly stopped?: number;
   readonly format?: 'short' | 'clock' | (string & {});
}

/** Props of `rate`. */
export interface RateProps {
   readonly value?: number;
   readonly unit?: string;
}

/** Props of `progress`. */
export interface ProgressProps {
   /** The fraction done, 0–1; absent is indeterminate. */
   readonly value?: number;
   readonly label?: Spans;
}

/** A tick on a meter. */
export interface MeterMark {
   readonly at: number;
   readonly tone?: Tone;
   readonly title?: string;
   readonly icon?: IconName;
}

/** Props of `meter`. */
export interface MeterProps {
   readonly value?: number;
   readonly parts?: readonly MeterPart[];
   readonly thresholds?: Thresholds;
   readonly style?: 'bar' | 'ring' | 'blocks' | (string & {});
   readonly size?: 'sm' | 'md' | 'lg' | (string & {});
   readonly steps?: number;
   readonly marks?: readonly MeterMark[];
   readonly label?: Spans;
   readonly total?: Spans;
}

/** A point of a chart's one series: a bar, a line point or a donut segment. */
export interface ChartPoint {
   /** Absent or `null` on a `line`/`area`: a gap, the line breaks there. */
   readonly value?: number | null;
   /** The point's x label (`line`/`area` without `labels`), a bar's or a segment's label. */
   readonly label?: string;
   /** Replaces the point's tooltip. */
   readonly title?: string;
   /** This bar's fill instead of `token`. */
   readonly tone?: Tone;
   /** `donut`: this segment's color. */
   readonly token?: string;
}

/** One series of a `line` or `area` chart with several. */
export interface ChartSeries {
   /** Names its legend chip and its part of the tooltip. */
   readonly label?: string;
   /** Its color (default the next of the chart palette). */
   readonly token?: string;
   /** One entry per point, sharing x positions by index; `null` is a gap. */
   readonly values: readonly (
      | number
      | null
      | { readonly value: number; readonly title?: string }
   )[];
}

/** Props of `chart`. */
export interface ChartProps {
   readonly kind?: 'heatmap' | 'bars' | 'spark' | 'line' | 'area' | 'donut' | (string & {});
   readonly size?: 'sm' | 'md' | 'lg' | (string & {});
   /** The first series' color; further series without a `token` take the chart palette. */
   readonly token?: string;
   readonly summary?: Spans;
   /** One series of points, or (`line`, `area`) several series, one line each. */
   readonly series?: readonly (number | null | ChartPoint)[] | readonly ChartSeries[];
   /** `line`/`area`: the x labels by point index; `""` leaves a point unlabelled. */
   readonly labels?: readonly string[];
   /** `line`/`area`/`donut`: written with every value Tern prints: `$`, `€`, `£`, `¥` lead it, `%`, `×`, `°` attach after it, words follow a space. */
   readonly unit?: string;
   /** `line`/`area` with one series: the label beside the last point (`true`, the default, its value; `false` none). */
   readonly last?: boolean | Spans;
   /** `donut`: what the ring's middle says (default the sum of the values, with `unit`). */
   readonly total?: Spans;
   /** `donut`: a quiet line under the ring. */
   readonly caption?: Spans;
   /** `bars`/`spark`/`line`/`area`: the value domain's low end; values are clamped into it. */
   readonly min?: number;
   /** `bars`/`spark`/`line`/`area`: the value domain's high end. */
   readonly max?: number;
   /** `bars`/`spark`/`line`/`area`: dashed reference lines in value units; `title` is the line's tooltip. */
   readonly marks?: readonly {
      readonly value: number;
      /** The line's color (default muted foreground). */
      readonly tone?: Tone;
      readonly title?: string;
   }[];
   readonly cells?: readonly (readonly (number | null)[])[];
   readonly tips?: readonly (readonly (string | null)[])[];
   readonly rows?: readonly string[];
   readonly cols?: readonly { readonly at: number; readonly label: string }[];
}

/** Props of `effort`. */
export interface EffortProps {
   readonly level?: 'off' | 'minimal' | 'low' | 'medium' | 'high' | 'xhigh' | 'max' | (string & {});
}

/** A styled range of an editor's text (UTF-16 offsets). */
export interface Decor {
   readonly from: number;
   readonly to: number;
   readonly s: SpanToken;
   readonly fx?: SpanFx;
}

/** A button attached to an `input` or `editor`. */
export interface InputButton {
   /** What a click sends (`copy` copies the text; `x=y` sends `act` `x` with `value` `y`). */
   readonly act: string;
   /** The label; without one the button is icon-only. */
   readonly text?: string;
   readonly icon?: IconName;
   /** `default`: an attached segment; the others: an inset small button. */
   readonly variant?: ButtonVariant;
   /** Keycaps after the label (inset buttons). */
   readonly keys?: readonly string[];
   /** The tooltip, and an icon-only button's accessible name. */
   readonly title?: string;
   /** Faded and inert; a string is the tooltip reason. */
   readonly disabled?: boolean | string;
   /** A spinner in the icon's place (inset buttons), inert. */
   readonly busy?: boolean;
}

/** Props of `input` and `editor`. */
export interface FieldTextProps {
   readonly text?: string;
   /** The caret, in UTF-16 units. */
   readonly cursor?: number;
   /** The selection anchor, in UTF-16 units. */
   readonly anchor?: number;
   readonly decor?: readonly Decor[];
   readonly ghost?: string;
   readonly placeholder?: Spans;
   readonly prompt?: Spans;
   /** A vim mode label; turns native editing off. */
   readonly mode?: string;
   readonly lang?: string;
   readonly readonly?: boolean;
   /** The owner accepts an atomic `send`. */
   readonly sendable?: boolean;
   /** The program cannot read input yet: Tern keeps what is typed and sends it once this clears. */
   readonly typeahead?: boolean;
   /** `field` (the default with any addon) frames it as a form field; `well` the bare well. */
   readonly variant?: 'well' | 'field' | (string & {});
   /** Attached buttons, in order; a click sends `action` with the button's `act`. */
   readonly buttons?: readonly InputButton[];
}

/** Props of `input`. */
export interface InputProps extends FieldTextProps {
   /** A leading icon. */
   readonly icon?: IconName;
   /** One keycap for a chord at the end while the field is idle and empty (`['cmd', 'f']` draws ⌘F). */
   readonly keys?: string | readonly string[];
   /** Keeps the mono face for the text in a sans region. */
   readonly mono?: boolean;
   /** A circled × while there is text; sends `action` `clear`. */
   readonly clear?: boolean;
   /** A mono well before the text. */
   readonly prefix?: string;
   /** A mono well after the text. */
   readonly suffix?: string;
   /** A mono unit after the text, inside the field. */
   readonly unit?: string;
   /** An attached up/down segment; sends `action` `inc` / `dec`. */
   readonly stepper?: boolean;
   /** Draws a dot per UTF-16 unit of `text`, with an eye that reveals it locally. */
   readonly secret?: boolean;
   /** A secret's shown state; a change wins over the eye. */
   readonly reveal?: boolean;
}

/** Props of `editor`. */
export interface EditorProps extends FieldTextProps {
   readonly maxLines?: number;
}

/** Props of `status`. */
export interface StatusProps {
   readonly transparent?: boolean;
}

/** Props of `seg`. */
export interface SegProps {
   readonly text?: string;
   readonly spans?: readonly (string | SpanData)[];
   readonly icon?: IconName;
   readonly side?: 'right' | (string & {});
   readonly priority?: number;
}

/** Props of `toast`. */
export interface ToastProps {
   readonly text?: string;
   readonly sub?: string;
   readonly ttl?: number;
}

/** Where an overlay sits. */
export type OverlayAnchor =
   | 'center'
   | 'top'
   | 'bottom'
   | { readonly node: string; readonly side?: 'below' | 'above' }
   | { readonly caret: string };

/** Props of `overlay`. */
export interface OverlayProps {
   readonly anchor?: OverlayAnchor;
   /**
    * The card's width (`sm` anchored, else `md`); with `side`, the sheet's
    * width or the drawer's height, or a fraction (0, 1] of the layer.
    */
   readonly size?: 'sm' | 'md' | 'lg' | 'full' | number | (string & {});
   /** Docks the card to an edge, winning over `anchor`: a full-height sheet (`left`, `right`) or a drawer (`bottom`). */
   readonly side?: 'right' | 'left' | 'bottom' | (string & {});
   /** With `side`: `edge` (default) flush with the edge, `inset` floating off it with rounded corners. */
   readonly variant?: 'edge' | 'inset' | (string & {});
   readonly modal?: boolean;
   readonly head?: Spans;
   /** The head icon. */
   readonly icon?: IconName;
   /** Muted text after the head text. */
   readonly meta?: Spans;
   /** A node drawn at the head's end; sent as the child keyed `end`, or a child's id. */
   readonly end?: SlotNode | string;
   /** A node drawn in a footer bar under the body; sent as the child keyed `foot`, or an id. */
   readonly foot?: SlotNode | string;
}

/** A chip of a tool or agent head. */
export interface Chip {
   readonly text: string;
   readonly tone?: Tone;
   readonly title?: string;
}

/** Props of `tool`. */
export interface ToolProps {
   readonly name?: string;
   readonly title?: Spans;
   readonly target?: Spans;
   readonly targetKind?: 'command' | 'path' | 'pattern' | 'query' | 'text' | (string & {});
   readonly meta?: readonly Spans[];
   readonly badges?: readonly Chip[];
   readonly note?: Spans;
   readonly exit?: number;
   /** `ask` waits for approval and draws the approval panel. */
   readonly status?: Status | 'ask';
   readonly age?: number;
   readonly took?: number;
   readonly intent?: string;
   readonly frame?: 'card' | 'inline' | (string & {});
   readonly collapsible?: boolean;
   readonly collapsed?: boolean;
   readonly preview?: { readonly lines: number } | { readonly tail: number };
   readonly tools?: readonly {
      readonly id: string;
      readonly label?: string;
      readonly keys?: readonly string[];
   }[];
   /** The approval look while `status` is `ask`: `card` (default), `strip` or `scopes`. */
   readonly confirm?: 'card' | 'strip' | 'scopes' | (string & {});
   /** The approval panel's contents. */
   readonly ask?: ToolAsk;
   /** The resolved answer, shown as a head chip (any status). */
   readonly approved?: 'once' | 'session' | 'always' | 'denied' | (string & {});
}

/** The approval panel of a `tool` waiting with `status: 'ask'`. */
export interface ToolAsk {
   /** The question. */
   readonly text?: string;
   /** The Allow button's label (default `Allow`). */
   readonly allow?: string;
   /** The Deny button's label (default `Deny`). */
   readonly deny?: string;
   /** The Always button's label (`card` only; default `Always`); `false` hides it. */
   readonly always?: string | false;
   /** The radios of the `scopes` look (default once, session, always); `label` defaults to `id`. */
   readonly scopes?: readonly { readonly id: string; readonly label?: string }[];
   /** The scope selected, sent as the actions' `value`; a change wins over the user's pick. */
   readonly scope?: string;
}

/** The stats of an `agent` row. */
export interface AgentStats {
   readonly tools?: number;
   readonly requests?: number;
   readonly done?: number;
   readonly context?: number;
   readonly contextLabel?: string;
   readonly tokens?: number;
   readonly cost?: number;
   readonly age?: number;
   readonly took?: number;
}

/** Props of `agent`. */
export interface AgentProps {
   readonly name?: string;
   readonly agent?: string;
   readonly badges?: readonly Chip[];
   readonly task?: Spans;
   readonly status?: AgentStatus;
   readonly model?: string;
   /** A palette token coloring the model chip's dot. */
   readonly thinking?: string;
   readonly stats?: AgentStats;
   readonly tool?: { readonly name: string; readonly intent?: string; readonly age?: number };
   readonly retry?: {
      readonly attempt?: number;
      readonly max?: number;
      readonly delay?: number;
      readonly age?: number;
      readonly error?: string;
   };
   readonly depth?: number;
   readonly collapsible?: boolean;
   readonly collapsed?: boolean;
}

/** The state of a checklist item. */
export type ChecklistStatus = 'pending' | 'active' | 'done' | 'dropped' | 'blocked' | (string & {});

/** One phase of a `checklist`. */
export interface ChecklistPhase {
   readonly id?: string;
   readonly title?: Spans;
   readonly items?: readonly {
      readonly id: string;
      readonly text?: Spans;
      readonly status?: ChecklistStatus;
      readonly note?: Spans;
   }[];
   readonly collapsed?: boolean;
}

/** Props of `checklist`. */
export interface ChecklistProps {
   readonly phases?: readonly ChecklistPhase[];
   readonly mode?: 'full' | 'hud' | 'reminder' | (string & {});
   readonly note?: Spans;
}

/** An option of a `prefs` choice or multi control. */
export interface PrefsOption {
   readonly value: string;
   readonly label?: string;
   readonly detail?: string;
}

/** The control of a `prefs` row, by `k`. */
export type PrefsControl =
   | { readonly k: 'switch'; readonly on?: boolean }
   | {
        readonly k: 'choice';
        readonly value?: string;
        readonly options?: readonly PrefsOption[];
        readonly style?: 'auto' | 'segmented' | 'menu';
        readonly mono?: boolean;
     }
   | {
        readonly k: 'number';
        readonly value?: number;
        readonly min?: number;
        readonly max?: number;
        readonly step?: number;
        readonly unit?: string;
        readonly labels?: Readonly<Record<string, string>>;
     }
   | {
        readonly k: 'text';
        readonly value?: string;
        readonly placeholder?: string;
        readonly secret?: boolean;
        readonly mono?: boolean;
     }
   | { readonly k: 'keys'; readonly keys?: readonly (readonly string[])[] }
   | {
        readonly k: 'multi';
        readonly values?: readonly string[];
        readonly options?: readonly PrefsOption[];
        readonly ordered?: boolean;
     }
   | { readonly k: 'action'; readonly act?: string; readonly label?: string };

/** A row of a `prefs` section. */
export interface PrefsRow {
   readonly id: string;
   readonly label?: string;
   readonly hint?: string;
   readonly warning?: string;
   readonly disabled?: string;
   readonly changed?: boolean;
   readonly defaultLabel?: string;
   readonly control?: PrefsControl;
}

/** Props of `prefs`. */
export interface PrefsProps {
   readonly title?: string;
   readonly pages?: readonly {
      readonly id: string;
      readonly label?: string;
      readonly group?: string;
      readonly icon?: IconName;
      readonly disabled?: string;
      readonly changed?: number;
   }[];
   readonly page?: string;
   readonly lead?: string;
   readonly sections?: readonly {
      readonly id?: string;
      readonly title?: string;
      readonly page?: string;
      readonly rows: readonly PrefsRow[];
   }[];
   readonly query?: string;
   readonly cursor?: number;
   readonly focus?: string;
   readonly editing?: {
      readonly row: string;
      readonly option?: string;
      readonly draft?: string;
      readonly cursor?: number;
   };
}

/** An option of a `select`. */
export interface SelectOption {
   /** What `value`, `active` and `change` carry. */
   readonly value: string;
   /** What the option and, when current, the trigger show (default `value`). */
   readonly label?: string;
   /** Muted mono text at the option's right. */
   readonly detail?: string;
   /** A leading icon (`provider:<id>` for a provider's logo); the current option's leads the trigger. */
   readonly icon?: IconName;
   /** A section header over the first option of each run with the same group. */
   readonly group?: string;
   /** Drawn faint and never picked. */
   readonly disabled?: boolean;
}

/** Props of `select`: a pick sends `change` with the option's `value`. */
export interface SelectProps {
   /** The current option's `value`; one no option has shows as itself. */
   readonly value?: string;
   /** The options in order; a string is an option whose value and label are that string. */
   readonly options?: readonly (string | SelectOption)[];
   /** `field` (default) a form field, `quiet` a borderless trigger for heads and bars. */
   readonly variant?: 'field' | 'quiet' | (string & {});
   /** A muted label before the value on the trigger. */
   readonly label?: string;
   /** The trigger's text without a `value`, and the search line's without a `query`. */
   readonly placeholder?: string;
   /** Default `md`. */
   readonly size?: 'sm' | 'md' | (string & {});
   /** Dimmed and inert; a string is the tooltip reason. */
   readonly disabled?: boolean | string;
   /** The menu shows; a change wins over the pointer's open or close. */
   readonly open?: boolean;
   /** The highlighted option's `value` (your keyboard cursor; default the current `value`). */
   readonly active?: string;
   /** A combobox: a search line your program filters `options` by. */
   readonly search?: boolean;
   /** The search text; its hits are marked in the labels. */
   readonly query?: string;
   /** The caret in `query`, in UTF-16 units. */
   readonly cursor?: number;
   /** The size of the whole set, counted beside a `query`. */
   readonly total?: number;
   /** The menu's text when `options` is empty. */
   readonly empty?: string;
}

/** Props of `switch`: a flip sends `change` with the new state as `value`. */
export interface SwitchProps {
   /** A change wins over the user's local flip. */
   readonly on?: boolean;
   /** `default`, `compact` or `segment` (two labeled sides). */
   readonly variant?: 'default' | 'compact' | 'segment' | (string & {});
   /** The setting's name, and the accessible name; a labeled switch fills its row. */
   readonly label?: string;
   /** A muted line under the label. */
   readonly hint?: string;
   /** A `segment`'s two words, off first (default `['Off', 'On']`). */
   readonly labels?: readonly [string, string];
   /** Dimmed and inert; a string is the tooltip reason. */
   readonly disabled?: boolean | string;
}

/** Props of `slider`: a drag sends `change` with the value (a pair for a range). */
export interface SliderProps {
   /** The value, or a `[lo, hi]` range with two thumbs; clamped and moved to the nearest step. */
   readonly value?: number | readonly [number, number];
   /** The low end (default 0). */
   readonly min?: number;
   /** The high end (default 100). */
   readonly max?: number;
   /** The distance between values, counted from `min`. */
   readonly step?: number;
   /** The name at the left; the thumbs speak it. */
   readonly label?: string;
   /** Written after the number (`format: 'number'`); `%`, `°` and `×` attach. */
   readonly unit?: string;
   /** `number` (default), `percent` (a fraction as a percentage) or `duration` (milliseconds). */
   readonly format?: 'number' | 'percent' | 'duration' | (string & {});
   /** Tick marks: `true` one per step, a number that many equal parts, a list one at each value. */
   readonly ticks?: boolean | number | readonly number[];
   /** `default`, or `accent` with the label over the track. */
   readonly variant?: 'default' | 'accent' | (string & {});
   /** Dimmed and inert; a string is the tooltip reason. */
   readonly disabled?: boolean | string;
}

/** Props of `field`: a labelled form row around its control children. */
export interface FieldProps {
   readonly label?: Spans;
   /** The help line, hidden while an `error` or `warning` shows. */
   readonly hint?: Spans;
   /** Something is wrong: the note in red and a red ring on the control; `true` the state without a line. */
   readonly error?: Spans | boolean;
   /** Worth a look but not wrong, as `error` in amber; ignored while `error` is set. */
   readonly warning?: Spans | boolean;
   /** A `*` after the label, display only. */
   readonly required?: boolean;
   /** `grid` only: the label column's width (default `12ch`). */
   readonly labelWidth?: Extent;
   /** `stacked` (default), `row` or `grid`. */
   readonly variant?: 'stacked' | 'row' | 'grid' | (string & {});
}

/** Props of `otp`: a one-time code, one cell per character. */
export interface OtpProps {
   /** The characters typed so far. */
   readonly text?: string;
   /** The number of cells, 1–12 (default 6). */
   readonly length?: number;
   /** Cells per group, a dash between groups (default 3; `0` one group). */
   readonly group?: number;
   /** The active cell, 0-based (default the length of `text`); `false` none. */
   readonly cursor?: number | false;
   /** `cells` (default), `joined` or `underline`. */
   readonly variant?: 'cells' | 'joined' | 'underline' | (string & {});
   /** Hides what was typed: `true` a dot per filled cell, a string its first character. */
   readonly mask?: boolean | string;
   /** `error` rings the cells red and shakes them once; `success` rings them green. */
   readonly state?: 'error' | 'success' | (string & {});
   /** A quiet line under the cells, in the state's color. */
   readonly hint?: string;
   /** Dimmed, no active cell; a string is the tooltip reason. */
   readonly disabled?: boolean | string;
}

/** Props of `buttons`: its `button` children drawn as one group. */
export interface ButtonsProps {
   /** `joined` (default), `split` or `toolbar`. */
   readonly variant?: 'joined' | 'split' | 'toolbar' | (string & {});
   /** The children's size, winning over their own (default `md`). */
   readonly size?: ButtonSize;
   /** The group's accessible name. */
   readonly label?: string;
}

/** An avatar's size: 16, 20, 28 or 36px. */
export type AvatarSize = 'xs' | 'sm' | 'md' | 'lg' | (string & {});

/** One avatar: initials, an icon or a picture. */
export interface AvatarItem {
   /** `circle` (default) a person, `square` a bot, an org or a repo. */
   readonly variant?: 'circle' | 'square' | (string & {});
   /** The name the mark stands for: its first two characters are the initials. */
   readonly text?: string;
   /** Picks the colors (default `text`): a stable id keeps them across renames. */
   readonly seed?: string;
   /** An icon name or `provider:<id>`, drawn instead of the initials. */
   readonly icon?: IconName;
   /** A picture's blob id, cropped to the face. */
   readonly blob?: string;
   /** A dot in the tone's color at the bottom right (`success` online, `warning` away). */
   readonly status?: Tone;
   /** The tooltip and accessible name (default `text`). */
   readonly title?: string;
}

/** Props of `avatar`. */
export interface AvatarProps extends AvatarItem {
   /** Default `md`. */
   readonly size?: AvatarSize;
}

/** Props of `avatars`: a stack of avatars with a `+N` overflow chip. */
export interface AvatarsProps {
   /** The avatars, front first; their `status` dots are not drawn. */
   readonly items?: readonly AvatarItem[];
   /** How many to show before the `+N` chip (default all). */
   readonly max?: number;
   /** Every avatar's size (default `md`). */
   readonly size?: AvatarSize;
   /** Muted text after the stack, and the group's accessible name. */
   readonly label?: string;
}

/** A place of a `breadcrumb`. */
export interface Crumb {
   readonly id: string | number;
   /** Default the `id`. */
   readonly label?: string;
   readonly icon?: IconName;
   /** A URL or path a ⌘-click opens. */
   readonly href?: string;
}

/** Props of `breadcrumb`: a click on a crumb but the last sends `select` with its id. */
export interface BreadcrumbProps {
   /** The trail, root first; the last is the current place. */
   readonly items?: readonly Crumb[];
   /** `chevron` (default) or `path`. */
   readonly variant?: 'chevron' | 'path' | (string & {});
   /** Most crumbs shown; the middle ones fold into a "…" menu. */
   readonly max?: number;
   /** The fold's menu shows; a change wins over the user's open or close. */
   readonly open?: boolean;
   /** The crumb under your keyboard cursor. */
   readonly active?: string;
}

/** Props of `pager`: a page change sends `change` with the page, the more row `action` `more`. */
export interface PagerProps {
   /** `range` (default) a range with arrows, `pages` page numbers, `more` a show-more row. */
   readonly variant?: 'range' | 'pages' | 'more' | (string & {});
   /** The current page, 1-based (default 1); for `more` the pages loaded so far. */
   readonly page?: number;
   /** The page count (default from `total` / `size`). */
   readonly pages?: number;
   /** The item count. */
   readonly total?: number;
   /** Items per page. */
   readonly size?: number;
   /** A turning ring; the more row sends nothing. */
   readonly loading?: boolean;
   /** What the items are, plural, after the counts. */
   readonly label?: string;
}

/** Props of `empty`: an empty state over its action children. */
export interface EmptyProps {
   /** `center` (default), `inline` or `drop` (a drop zone). */
   readonly variant?: 'center' | 'inline' | 'drop' | (string & {});
   /** The icon (default by look); `""` for none. */
   readonly icon?: IconName;
   /** What is missing, in a few words. */
   readonly title?: Spans;
   /** One muted line: why, or what to do. */
   readonly text?: Spans;
   /** `inline` only: draws the row in a hairline card. */
   readonly frame?: boolean;
   /** `drop` only: a drag is over the zone. */
   readonly over?: boolean;
}

/** Props of `skeleton`: placeholder shapes while content loads. */
export interface SkeletonProps {
   /** `shimmer` (default), `pulse` or `wire`. */
   readonly variant?: 'shimmer' | 'pulse' | 'wire' | (string & {});
   /** What it stands for (default `text`). */
   readonly shape?: 'text' | 'avatar' | 'rows' | 'card' | 'block' | (string & {});
   /** `text` only: how many lines, 1–24 (default 3). */
   readonly lines?: number;
   /** `rows` only: how many rows, 1–50 (default 4). */
   readonly rows?: number;
   /** `card` only: how many stat tiles, 1–8 (default 3). */
   readonly tiles?: number;
}

/** A selected day (`YYYY-MM-DD`), or with `range` its ends (`[from, null]` waits for the end). */
export type DateValue = string | readonly [string, string | null];

/** Props of `calendar`: a pick sends `change` with the day, or a range's sorted ends. */
export interface CalendarProps {
   /** `month` (default) a month grid, `week` presets beside a week strip. */
   readonly variant?: 'month' | 'week' | (string & {});
   /** The selection; a change wins over the user's pick. */
   readonly value?: DateValue;
   /** Picks a range (two clicks) instead of a day. */
   readonly range?: boolean;
   /** The month shown, `YYYY-MM` (default the month of `value`, else of `today`). */
   readonly month?: string;
   /** The day ringed as today (default the system's local date). */
   readonly today?: string;
   /** The first day that can be picked. */
   readonly min?: string;
   /** The last day that can be picked. */
   readonly max?: string;
   /** The first column of the month grid (default `mon`). */
   readonly weekStart?: 'mon' | 'sun' | (string & {});
   /** Days carrying a dot, in the accent or the tone's color. */
   readonly marks?: readonly (string | { readonly date: string; readonly tone?: Tone })[];
   /** The day your keys are on, drawn with a focus ring. */
   readonly cursor?: string;
   /** The `week` variant's presets column; a choice sends `select` with its `id`. */
   readonly presets?: readonly { readonly id: string; readonly label?: string }[];
   /** The active preset's `id`. */
   readonly preset?: string;
   /** A muted line under the days. */
   readonly note?: string;
}

/** Props of `date`: a date field opening a calendar. */
export interface DateProps extends Pick<
   CalendarProps,
   'month' | 'today' | 'min' | 'max' | 'weekStart' | 'marks' | 'cursor' | 'note'
> {
   /** As `calendar`'s; the field shows `Oct 30, 2026`. */
   readonly value?: DateValue;
   /** Picks a range; the popover stays open until the second end. */
   readonly range?: boolean;
   /** The popover shows; a change wins over the user's click. */
   readonly open?: boolean;
   /** Shown in the field without a `value` (default `Pick a date`, `Pick dates` with `range`). */
   readonly placeholder?: string;
}

/** Props of `menu`: a button opening a menu; a pick sends `action` with the row's `id`. */
export interface MenuProps {
   /** The rows, in order. */
   readonly items?: readonly MenuEntry[];
   /** The trigger's label; without `text` and `icon` a square … button. */
   readonly text?: string;
   /** The trigger's leading icon. */
   readonly icon?: IconName;
   /** The trigger's look, as the `button` variants. */
   readonly trigger?: 'default' | 'quiet' | 'primary' | (string & {});
   /** A chevron after the label, turning over while open. */
   readonly caret?: boolean;
   /** The trigger's size (default `md`). */
   readonly size?: ButtonSize;
   /** `default`, or `dense` rows for long sort and filter menus. */
   readonly variant?: 'default' | 'dense' | (string & {});
   /** The menu shows; a change wins over the user's open or close. */
   readonly open?: boolean;
   /** The highlighted row's `id` (your keyboard cursor). */
   readonly active?: string;
   /** The edge of the trigger the menu lines up with (default `start`). */
   readonly align?: 'start' | 'end' | (string & {});
   /** The trigger is dimmed and does not open; a string is the tooltip reason. */
   readonly disabled?: boolean | string;
}

/** A menu of a `menubar`. */
export interface MenubarMenu {
   /** Names the menu in events and in `open`. */
   readonly id: string;
   /** Its title (default the `id`). */
   readonly label?: string;
   readonly items: readonly MenuEntry[];
}

/** Props of `menubar`: a pick sends `action` with the row's `id` and the menu's as `item`. */
export interface MenubarProps {
   readonly menus?: readonly MenubarMenu[];
   /** `strip` (default) or `pill`. */
   readonly variant?: 'strip' | 'pill' | (string & {});
   /** The app's mark before the titles. */
   readonly icon?: IconName;
   /** The `id` of the menu shown open; a change wins over the pointer. */
   readonly open?: string;
   /** The highlighted row's `id` in the open menu (your keyboard cursor). */
   readonly active?: string;
}

/** A sub-item of a `nav` item. */
export interface NavSubItem {
   readonly id: string;
   /** Default the `id`. */
   readonly label?: string;
   /** A muted note in the menu, the description line in the panel. */
   readonly detail?: string | number;
   readonly icon?: IconName;
}

/** An item of a `nav`. */
export interface NavItem {
   readonly id: string;
   /** Default the `id`. */
   readonly label?: string;
   readonly icon?: IconName;
   /** A small mono pill after the label. */
   readonly count?: string | number;
   /** Sub-items: the item opens a menu (`pills`) or a panel (`links`) instead of being picked. */
   readonly items?: readonly NavSubItem[];
}

/** Props of `nav`: a pick sends `select` with the item or sub-item's id. */
export interface NavProps {
   readonly items?: readonly NavItem[];
   /** The active item or sub-item (a sub-item marks its parent too). */
   readonly active?: string;
   /** The item whose menu or panel shows. */
   readonly open?: string;
   /** `pills` (default) or `links`. */
   readonly variant?: 'pills' | 'links' | (string & {});
}

/** Props of `split`: panes (its children) the user resizes; a drag sends `change` with the shares. */
export interface SplitProps {
   /** `hairline` (default), `gutter` or `tiles`. */
   readonly variant?: 'hairline' | 'gutter' | 'tiles' | (string & {});
   /** `row` (default) panes side by side, `col` stacked. */
   readonly dir?: 'row' | 'col' | (string & {});
   /** Each pane's share, in child order, normalized to sum to 1; a change wins over the user's drags. */
   readonly sizes?: readonly number[];
   /** Each pane's minimum along the split (one value for all, `null` for none), or the common size bound. */
   readonly min?: Extent | readonly (Extent | null)[] | Bounds;
   /** The split's height (default the panes' content in a `row`, `12lines` in a `col`). */
   readonly h?: Extent;
   /** The handle your keys move (0 is the one after the first pane), drawn held. */
   readonly active?: number;
}

/** Props of `scroll`: a scrolling container of its children. */
export interface ScrollProps {
   /** `bar` (default) or `fade` edges. */
   readonly variant?: 'bar' | 'fade' | (string & {});
   /** `end` keeps to the end while content grows, until the user scrolls up. */
   readonly stick?: 'end' | (string & {});
   /** A stuck view follows its end (default `true` with `stick`); a change wins. */
   readonly pinned?: boolean;
   /** When the thumb shows: `auto` (default) while scrolling, `always`. */
   readonly bar?: 'auto' | 'always' | (string & {});
   /** Space between the children (default `none`). */
   readonly gap?: Gap;
   /** What the counts name: a word made plural with an `s`, or `[singular, plural]`. */
   readonly noun?: string | readonly [string, string];
}

/** Props of `carousel`: its children shown one slide at a time; a move sends `change` with the index. */
export interface CarouselProps {
   /** The slide shown, from 0; a change wins over the user's move. */
   readonly index?: number;
   /** Moving past either end wraps around. */
   readonly loop?: boolean;
   /** `peek` (default), `arrows` or `strip`. */
   readonly variant?: 'peek' | 'arrows' | 'strip' | (string & {});
   /** The slides' height: px, or `lines`/`ch` (default 200px). */
   readonly h?: number | `${number}lines` | `${number}ch`;
}

/** A reaction chip of a `message`; a flip sends `change` with its `text` as `value` and `on`. */
export interface Reaction {
   /** An emoji or glyph, its identity. */
   readonly text: string;
   /** How many reacted, the local user included (hidden at 0). */
   readonly count?: number;
   /** The local user reacted. */
   readonly on?: boolean;
}

/** An icon button of a `message` or an `attachment`; a click sends `action` with its `act`. */
export interface IconAction {
   readonly act: string;
   /** Default the act's own icon. */
   readonly icon?: IconName;
   /** The tooltip and accessible name (default the act, capitalized). */
   readonly title?: string;
}

/** Props of `message`: one chat message; its children are the body. */
export interface MessageProps {
   /** `auto` (default), `bubble` or `flat`. */
   readonly variant?: 'auto' | 'bubble' | 'flat' | (string & {});
   /** `end` is the local user. */
   readonly side?: 'start' | 'end' | (string & {});
   /** The name in the header. */
   readonly author?: string;
   /** The time in the header, mono. */
   readonly time?: string;
   /** The avatar (an `avatar`, `icon` or `image`); sent as the child keyed `avatar`, or a child's id. */
   readonly avatar?: SlotNode | string;
   /** Same author as the message before: no header, no avatar. */
   readonly continued?: boolean;
   readonly reactions?: readonly Reaction[];
   /** Icon buttons shown on hover. */
   readonly actions?: readonly IconAction[];
   /** When the actions show: `hover` (default) or `always`. */
   readonly reveal?: 'hover' | 'always' | (string & {});
   /** A quiet mono line under the message (`Delivered`). */
   readonly status?: string;
   /** Still on its way: the status shimmers. */
   readonly pending?: boolean;
}

/** Props of `attachment`: a file attached to a message, with its upload state. */
export interface AttachmentProps {
   /** The file name; also the tooltip and accessible name. */
   readonly name?: string;
   /** `chip` (default), `tile` or `pill`. */
   readonly variant?: 'chip' | 'tile' | 'pill' | (string & {});
   /** The extension shown (default from `name`, else `mime`). */
   readonly ext?: string;
   /** The MIME type, picking the file family when the extension says nothing. */
   readonly mime?: string;
   /** Size in bytes. */
   readonly size?: number;
   /** The meta line at rest, replacing `EXT · size`. */
   readonly meta?: string;
   /** An image's thumbnail blob id. */
   readonly blob?: string;
   /** A file opened by a click when there is no `href`. */
   readonly path?: string;
   /** Upload state: busy states shimmer and draw the progress line, `error` rings it red. */
   readonly state?: 'idle' | 'uploading' | 'processing' | 'error' | 'done' | (string & {});
   /** 0–1 while busy. */
   readonly progress?: number;
   /** The meta line in the `error` state (default `Upload failed`). */
   readonly error?: string;
   /** Adds a × that sends `action` `remove`. */
   readonly removable?: boolean;
   /** Action icons before the ×, in order. */
   readonly actions?: readonly IconAction[];
}

/** The state of a `trace` or one of its steps. */
export type TraceStatus = 'pending' | 'running' | 'done' | 'error' | (string & {});

/** A source of a `trace` or one of its steps. */
export interface TraceSource {
   readonly title?: string;
   readonly href?: string;
   /** Default the `href`'s host, `www.` dropped. */
   readonly site?: string;
}

/** A step of a `trace`; a child keyed by its `id` is its content. */
export interface TraceLine {
   /** Default its index. */
   readonly id?: string;
   readonly icon?: IconName;
   /** The step in words; shimmers while `running`. */
   readonly title?: Spans;
   /** What it acted on, in mono. */
   readonly target?: Spans;
   /** A dim line under the row. */
   readonly detail?: Spans;
   readonly status?: TraceStatus;
   /** Milliseconds elapsed when sent; a live timer while `running`. */
   readonly age?: number;
   /** Its time once settled. */
   readonly took?: number;
   /** Its content shows; a change wins over the user's choice. */
   readonly open?: boolean;
   readonly sources?: readonly TraceSource[];
}

/** Props of `trace`: an agent's steps; a flip sends `toggle` with `open`. */
export interface TraceProps {
   /** `rail` (default) a live log, `summary` a folded head, `card` reasoning. */
   readonly variant?: 'rail' | 'summary' | 'card' | (string & {});
   readonly lines?: readonly TraceLine[];
   readonly status?: TraceStatus;
   /** The head text. */
   readonly title?: Spans;
   /** The `summary` head's text (default derived from the status and time). */
   readonly summary?: Spans;
   /** The head glyph when not running or failed. */
   readonly icon?: IconName;
   /** Milliseconds elapsed when sent; the head's live timer while running or pending. */
   readonly age?: number;
   /** Total time once settled. */
   readonly took?: number;
   /** The head's fold is open (`summary`, `card`); a change wins over the user's choice. */
   readonly open?: boolean;
   /** The `Sources` row at the foot. */
   readonly sources?: readonly TraceSource[];
}

/** A frame of a `stacktrace`. */
export interface StackFrame {
   /** The function (`<anonymous>` when empty). */
   readonly fn?: string;
   /** The file path; a relative one opens against the pane's directory. */
   readonly file?: string;
   /** 1-based line. */
   readonly line?: number;
   /** 1-based column. */
   readonly col?: number;
   /** Your code (default `true`) or a library's. */
   readonly app?: boolean;
   /** A source excerpt for `split`: its first line number, its lines, and the line to mark. */
   readonly code?: {
      readonly start?: number;
      readonly lines: readonly string[];
      readonly line?: number;
   };
}

/** Props of `stacktrace`: a click sends `select`, a double click `activate`, with the frame index. */
export interface StacktraceProps {
   readonly error?: { readonly type?: string; readonly message?: string };
   /** The stack, innermost first. */
   readonly frames?: readonly StackFrame[];
   /** The selected frame's index (default the first app frame). */
   readonly selected?: number;
   /** The excerpts' language (default by the first excerpt's file name). */
   readonly lang?: string;
   /** `frames` (default), `split` or `mono`. */
   readonly variant?: 'frames' | 'split' | 'mono' | (string & {});
}

/** A suggested prompt; a click sends `activate` with its `id` (else its `text`). */
export interface Suggestion {
   readonly id?: string;
   /** The prompt as shown. */
   readonly text: string;
   readonly detail?: string;
   readonly icon?: IconName;
   /** `rows` only: the keycaps at the right (default the item's number). */
   readonly keys?: readonly string[];
}

/** Props of `suggestions`. */
export interface SuggestionsProps {
   /** `chips` (default), `rows` or `cards`. */
   readonly variant?: 'chips' | 'rows' | 'cards' | (string & {});
   readonly items?: readonly Suggestion[];
}

/** A question of a `questions` form. */
export interface Question {
   /** The `item` of its events and its key in `answers`. */
   readonly id: string;
   readonly prompt?: string;
   /** A line under the prompt. */
   readonly description?: string;
   /** The choices; `label` defaults to `value`, `detail` is a second line (`steps` only). */
   readonly choices?: readonly (
      | string
      | { readonly value: string; readonly label?: string; readonly detail?: string }
   )[];
   /** Several picks: the answer is an array of values. */
   readonly multi?: boolean;
   /** Takes a free-text answer from the `input` child keyed `<id>.other`. */
   readonly other?: { readonly placeholder?: string } | boolean;
   /** No Skip; Next and Submit wait for its answer. */
   readonly required?: boolean;
}

/** Props of `questions`: picks send `change`, Submit `action` `submit` with every answer. */
export interface QuestionsProps {
   readonly items?: readonly Question[];
   /** The current question's index. */
   readonly step?: number;
   /** The answers by question id: a value, values (`multi`) or text (Other). */
   readonly answers?: Readonly<Record<string, string | readonly string[]>>;
   /** `steps` (default), `inline` or `list`. */
   readonly variant?: 'steps' | 'inline' | 'list' | (string & {});
   /** `inline`'s hint line; `""` hides it. */
   readonly hint?: string;
   /** The submit label (default `Submit`). */
   readonly submit?: string;
}

/** The tags `el` draws. */
export const EL_TAGS = [
   'div',
   'span',
   'p',
   'section',
   'header',
   'footer',
   'nav',
   'aside',
   'main',
   'article',
   'figure',
   'blockquote',
   'ul',
   'ol',
   'li',
   'dl',
   'dt',
   'dd',
   'h1',
   'h2',
   'h3',
   'h4',
   'pre',
   'code',
   'kbd',
   'strong',
   'b',
   'em',
   'i',
   'del',
   'mark',
   'hr',
   'table',
   'thead',
   'tbody',
   'tr',
   'th',
   'td',
   'label',
   'button',
   'form',
   'input',
] as const;

/** An `el` tag. */
export type ElTag = (typeof EL_TAGS)[number];

/** The props of any `el` (but its tag). */
export interface HtmlProps {
   /** Space-separated classes for your stylesheets. */
   readonly class?: string;
   /** Extra attributes: `data-*`, `aria-*`, `role`, `colspan`, `rowspan`. */
   readonly attrs?: Readonly<Record<string, string | number | boolean>>;
   /** Plain text drawn before the children. */
   readonly text?: string;
}

/** The props of an `el` `input`: a checkbox or a radio. */
export interface ControlProps extends HtmlProps {
   readonly type?: 'checkbox' | 'radio' | (string & {});
   readonly name?: string;
   readonly value?: string;
   readonly checked?: boolean;
   readonly disabled?: boolean;
}

/** Props of `el`. */
export interface ElProps extends ControlProps {
   readonly tag?: ElTag | (string & {});
}

/** The props of each kind, by kind. */
export interface KindProps {
   col: StackProps;
   row: StackProps;
   card: CardProps;
   section: SectionProps;
   rule: RuleProps;
   spacer: SpacerProps;
   text: TextProps;
   md: MdProps;
   code: CodeProps;
   diff: DiffProps;
   ansi: AnsiProps;
   rows: RowsProps;
   math: MathProps;
   image: ImageProps;
   media: MediaProps;
   kv: KvProps;
   stats: StatsProps;
   table: TableProps;
   tree: TreeProps;
   json: JsonProps;
   badge: BadgeProps;
   kbd: KbdProps;
   icon: IconProps;
   list: ListProps;
   item: ItemProps;
   tabs: TabsProps;
   picker: PickerProps;
   spinner: SpinnerProps;
   shimmer: ShimmerProps;
   elapsed: ElapsedProps;
   rate: RateProps;
   progress: ProgressProps;
   meter: MeterProps;
   chart: ChartProps;
   effort: EffortProps;
   editor: EditorProps;
   input: InputProps;
   button: ButtonProps;
   status: StatusProps;
   seg: SegProps;
   toast: ToastProps;
   overlay: OverlayProps;
   tool: ToolProps;
   agent: AgentProps;
   checklist: ChecklistProps;
   prefs: PrefsProps;
   select: SelectProps;
   switch: SwitchProps;
   slider: SliderProps;
   field: FieldProps;
   otp: OtpProps;
   buttons: ButtonsProps;
   avatar: AvatarProps;
   avatars: AvatarsProps;
   breadcrumb: BreadcrumbProps;
   pager: PagerProps;
   empty: EmptyProps;
   skeleton: SkeletonProps;
   calendar: CalendarProps;
   date: DateProps;
   menu: MenuProps;
   menubar: MenubarProps;
   nav: NavProps;
   split: SplitProps;
   scroll: ScrollProps;
   carousel: CarouselProps;
   message: MessageProps;
   attachment: AttachmentProps;
   trace: TraceProps;
   stacktrace: StacktraceProps;
   suggestions: SuggestionsProps;
   questions: QuestionsProps;
   el: ElProps;
}

/** A kind with a builder: every kind but Tern's own `block`. */
export type BuildKind = keyof KindProps;
