/**
 * Typed props of every kind (except Tern's own `block`), the props every
 * node takes, and the enumerations they use. Every enumeration keeps a
 * `(string & {})` escape hatch for values newer than the SDK.
 */

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
}

/** Text given as a string or as a list of strings and styled spans. */
export type Spans = string | readonly (string | SpanData)[];

/** Tern's named icons and their aliases. */
export type IconName =
   | 'app-btop'
   | 'app-bun'
   | 'app-claude'
   | 'app-codex'
   | 'app-deno'
   | 'app-docker'
   | 'app-go'
   | 'app-hammer'
   | 'app-helix'
   | 'app-hermes'
   | 'app-htop'
   | 'app-java'
   | 'app-node'
   | 'app-omp'
   | 'app-python'
   | 'app-ruby'
   | 'app-rust'
   | 'app-vim'
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

/** What a pointer does on a node. */
export interface Actions {
   readonly click?: string;
   readonly dblclick?: string;
   readonly menu?: readonly string[];
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
   readonly head?: Spans;
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
   readonly head?: Spans;
   readonly collapsible?: boolean;
   readonly collapsed?: boolean;
}

/** Props of `rule`. */
export interface RuleProps {
   readonly label?: Spans;
}

/** Props of `spacer`. */
export interface SpacerProps {
   readonly size?: Gap;
}

/** Where overflowing text is cut. */
export type Truncate = 'end' | 'start' | 'middle' | (string & {});

/** Props of `text`. */
export interface TextProps {
   readonly text?: string;
   readonly spans?: readonly (string | SpanData)[];
   readonly wrap?: 'word' | 'char' | 'none' | (string & {});
   readonly truncate?: Truncate;
   readonly lines?: number;
   readonly measure?: 'prose' | (string & {});
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
   /** Byte ranges `[start, end)` in the line. */
   readonly ranges?: readonly (readonly [number, number])[];
}

/** Props of `code`. */
export interface CodeProps {
   readonly text?: string;
   readonly lang?: string;
   readonly path?: string;
   readonly numbers?: boolean;
   readonly start?: number;
   readonly marks?: readonly CodeMark[];
   readonly wrap?: boolean;
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

/** A column of a `table`. */
export interface TableCol {
   readonly id: string;
   readonly head?: Spans;
   readonly align?: 'start' | 'center' | 'end' | (string & {});
   readonly truncate?: Truncate;
   readonly priority?: number;
   readonly grow?: number;
}

/** A row of a `table`: one cell per column id. */
export interface TableRow {
   readonly id: string;
   readonly cells: Readonly<Record<string, Spans | MeterCell>>;
}

/** Props of `table`. */
export interface TableProps {
   readonly cols?: readonly TableCol[];
   readonly rows?: readonly TableRow[];
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

/** Props of `chart`. */
export interface ChartProps {
   readonly kind?: 'heatmap' | 'bars' | 'spark' | (string & {});
   readonly size?: 'sm' | 'md' | 'lg' | (string & {});
   readonly token?: string;
   readonly summary?: Spans;
   readonly series?: readonly {
      readonly value: number;
      readonly label?: string;
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

/** Props of `input`. */
export interface InputProps {
   readonly text?: string;
   /** The caret, in UTF-16 units. */
   readonly cursor?: number;
   /** The selection anchor, in UTF-16 units. */
   readonly anchor?: number;
   readonly decor?: readonly Decor[];
   readonly ghost?: string;
   readonly placeholder?: string;
   readonly prompt?: Spans;
   /** A vim mode label; turns native editing off. */
   readonly mode?: string;
   readonly lang?: string;
   readonly readonly?: boolean;
   /** The owner accepts an atomic `send`. */
   readonly sendable?: boolean;
}

/** Props of `editor`. */
export interface EditorProps extends InputProps {
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
   readonly size?: 'sm' | 'md' | 'lg' | 'full' | (string & {});
   readonly modal?: boolean;
   readonly head?: Spans;
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
   readonly status?: Status;
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
   kv: KvProps;
   table: TableProps;
   tree: TreeProps;
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
   status: StatusProps;
   seg: SegProps;
   toast: ToastProps;
   overlay: OverlayProps;
   tool: ToolProps;
   agent: AgentProps;
   checklist: ChecklistProps;
   prefs: PrefsProps;
   el: ElProps;
}

/** A kind with a builder: every kind but Tern's own `block`. */
export type BuildKind = keyof KindProps;
