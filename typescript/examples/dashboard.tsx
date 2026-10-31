/**
 * A `screen` dashboard in the UI face (`font: 'sans'`), laid out as a page:
 * a head row (title, muted meta, a status badge and Refresh), a flat stats
 * strip, underline tabs switching between Services, Deploys and Incidents,
 * and per view a full-width table beside a detail card for the selected row
 * (its one primary action in the card head, a danger trigger that asks
 * first). The dock holds the confirm strip and a key legend. Live numbers
 * tick every second.
 * Keys: ↑/↓ or j/k move, Tab/⇧Tab (or [/]) switch views, Enter runs the
 * selected row's primary action (or confirms), a approves or acknowledges,
 * s restarts, x rolls back, r refreshes, Esc backs out of a confirm,
 * Esc/q quit. Outside Tern it prints the page as plain text.
 */
import {
   connect,
   plain,
   span,
   type ActionEvent,
   type Handler,
   type IconName,
   type Key,
   type KvItem,
   type TableCol,
   type TableRow,
   type Tone,
} from '@stencil-hq/tern';

type Health = 'healthy' | 'degraded' | 'awaiting' | 'failed';
type DeployStatus = 'awaiting' | 'shipped' | 'failed' | 'rolled-back';
type Severity = 'sev1' | 'sev2' | 'sev3';

interface Service {
   readonly id: string;
   readonly name: string;
   readonly region: string;
   readonly owner: string;
   version: string;
   health: Health;
   rps: number;
   p95: number;
   cpu: number;
   readonly history: number[];
   deployed: number;
}

interface Deploy {
   readonly id: string;
   readonly service: string;
   readonly version: string;
   readonly change: string;
   readonly author: string;
   readonly at: number;
   /** How long the rollout took, in seconds; 0 while it hasn't run. */
   took: number;
   status: DeployStatus;
}

interface Incident {
   readonly id: string;
   readonly title: string;
   readonly service: string;
   readonly severity: Severity;
   readonly opened: number;
   acknowledged: boolean;
}

const HEALTH: Readonly<Record<Health, { label: string; tone: Tone }>> = {
   healthy: { label: 'Healthy', tone: 'success' },
   degraded: { label: 'Degraded', tone: 'warning' },
   awaiting: { label: 'Awaiting approval', tone: 'pending' },
   failed: { label: 'Failed', tone: 'error' },
};

const DEPLOY: Readonly<Record<DeployStatus, { label: string; tone: Tone }>> = {
   awaiting: { label: 'Awaiting approval', tone: 'pending' },
   shipped: { label: 'Shipped', tone: 'success' },
   failed: { label: 'Failed', tone: 'error' },
   'rolled-back': { label: 'Rolled back', tone: 'muted' },
};

const SEVERITY: Readonly<Record<Severity, { label: string; tone: Tone }>> = {
   sev1: { label: 'SEV1', tone: 'error' },
   sev2: { label: 'SEV2', tone: 'warning' },
   sev3: { label: 'SEV3', tone: 'info' },
};

const VIEWS = ['services', 'deploys', 'incidents'] as const;
type ViewId = (typeof VIEWS)[number];

const now = Date.now();
const minutes = (n: number): number => now - n * 60_000;

/** A short history of a value wandering around `base`. */
function history(base: number): number[] {
   return Array.from({ length: 16 }, (_, i) =>
      Math.round(base * (0.8 + 0.4 * Math.abs(Math.sin(i * 0.7 + base))))
   );
}

const services: Service[] = [
   {
      id: 'api',
      name: 'api-gateway',
      region: 'us-east-1',
      owner: 'Platform',
      version: 'v2.14.3',
      health: 'healthy',
      rps: 1840,
      p95: 112,
      cpu: 0.46,
      history: history(1800),
      deployed: minutes(42),
   },
   {
      id: 'auth',
      name: 'auth',
      region: 'us-east-1',
      owner: 'Identity',
      version: 'v1.9.0',
      health: 'awaiting',
      rps: 620,
      p95: 88,
      cpu: 0.31,
      history: history(600),
      deployed: minutes(5),
   },
   {
      id: 'billing',
      name: 'billing-worker',
      region: 'eu-west-1',
      owner: 'Payments',
      version: 'v3.2.1',
      health: 'degraded',
      rps: 210,
      p95: 940,
      cpu: 0.88,
      history: history(220),
      deployed: minutes(180),
   },
   {
      id: 'search',
      name: 'search-indexer',
      region: 'eu-west-1',
      owner: 'Discovery',
      version: 'v0.31.7',
      health: 'healthy',
      rps: 95,
      p95: 240,
      cpu: 0.22,
      history: history(90),
      deployed: minutes(1440),
   },
   {
      id: 'notify',
      name: 'notifications',
      region: 'ap-south-1',
      owner: 'Growth',
      version: 'v5.0.0',
      health: 'awaiting',
      rps: 340,
      p95: 76,
      cpu: 0.18,
      history: history(330),
      deployed: minutes(2),
   },
];

const deploys: Deploy[] = [
   {
      id: 'd-412',
      service: 'notify',
      version: 'v5.0.0',
      change: 'Batch push sends per region',
      author: 'ken',
      at: minutes(2),
      took: 0,
      status: 'awaiting',
   },
   {
      id: 'd-411',
      service: 'auth',
      version: 'v1.9.0',
      change: 'Rotate session signing keys',
      author: 'mara',
      at: minutes(5),
      took: 0,
      status: 'awaiting',
   },
   {
      id: 'd-410',
      service: 'api',
      version: 'v2.14.3',
      change: 'Raise the idle timeout to 75 s',
      author: 'jo',
      at: minutes(42),
      took: 252,
      status: 'shipped',
   },
   {
      id: 'd-409',
      service: 'billing',
      version: 'v3.2.1',
      change: 'Retry invoice writes on conflict',
      author: 'priya',
      at: minutes(180),
      took: 400,
      status: 'shipped',
   },
   {
      id: 'd-408',
      service: 'billing',
      version: 'v3.2.0',
      change: 'Move ledger writes to the queue',
      author: 'priya',
      at: minutes(300),
      took: 95,
      status: 'failed',
   },
   {
      id: 'd-407',
      service: 'search',
      version: 'v0.31.7',
      change: 'Reindex on schema change',
      author: 'sam',
      at: minutes(1440),
      took: 610,
      status: 'shipped',
   },
];

/** Open incidents; a resolved one leaves the list. */
const incidents: Incident[] = [
   {
      id: 'inc-88',
      title: 'p95 above 900 ms on invoice writes',
      service: 'billing',
      severity: 'sev2',
      opened: minutes(38),
      acknowledged: false,
   },
   {
      id: 'inc-87',
      title: 'Push delivery lag in ap-south-1',
      service: 'notify',
      severity: 'sev3',
      opened: minutes(95),
      acknowledged: true,
   },
];

/** What the page shows besides the data. */
const state: {
   view: ViewId;
   /** The selected row of each view. */
   selected: Record<ViewId, string>;
   refreshing: boolean;
   /** The action waiting for confirmation, on a service. */
   confirm: { kind: 'approve' | 'rollback'; service: string } | null;
   note: string;
   updated: number;
} = {
   view: 'services',
   selected: { services: 'api', deploys: 'd-412', incidents: 'inc-88' },
   refreshing: false,
   confirm: null,
   note: '',
   updated: now,
};

const service = (id: string): Service | undefined => services.find(s => s.id === id);

/** `42m ago`, `3h ago`. */
function ago(at: number): string {
   const m = Math.max(0, Math.round((Date.now() - at) / 60_000));
   if (m < 1) return 'just now';
   if (m < 60) return `${m}m ago`;
   return `${Math.round(m / 60)}h ago`;
}

/** `4m 12s`, `—` for a rollout that hasn't run. */
function took(seconds: number): string {
   if (seconds === 0) return '—';
   const m = Math.floor(seconds / 60);
   return m === 0 ? `${seconds}s` : `${m}m ${String(seconds % 60).padStart(2, '0')}s`;
}

/** A button the detail card shows, with the key that runs it. */
interface Action {
   readonly label: string;
   readonly key: string;
   readonly icon: IconName;
   readonly run: () => void;
}

/** The detail card of the selected row. */
interface Detail {
   readonly head: string;
   readonly meta: string;
   readonly items: readonly KvItem[];
   /** The one primary action, drawn in the card head. */
   readonly primary: Action | undefined;
   /** Default buttons in the card's action row. */
   readonly more: readonly Action[];
   /** A destructive trigger: it opens a confirm strip, never acts. */
   readonly danger: Action | undefined;
}

/** One view: its tab, its table and the detail of its selected row. */
interface View {
   readonly label: string;
   /** The muted count after the label. */
   readonly count: () => number;
   readonly ids: () => string[];
   readonly cols: readonly TableCol[];
   readonly rows: () => TableRow[];
   readonly empty: string;
   readonly detail: (id: string) => Detail | undefined;
   /** The row `menu` handlers by act. */
   readonly menu?: Readonly<Record<string, Handler<ActionEvent>>>;
}

const restartAction = (s: Service): Action => ({
   label: 'Restart',
   key: 's',
   icon: 'repeat',
   run: () => restart(s.id),
});

const rollbackAction = (s: Service): Action => ({
   label: `Roll back ${s.name}`,
   key: 'x',
   icon: 'undo',
   run: () => ask('rollback', s.id),
});

const approveAction = (s: Service): Action => ({
   label: `Approve ${s.version}`,
   key: 'a',
   icon: 'check',
   run: () => ask('approve', s.id),
});

const VIEW: Readonly<Record<ViewId, View>> = {
   services: {
      label: 'Services',
      count: () => services.length,
      ids: () => services.map(s => s.id),
      cols: [
         { id: 'name', head: 'Service', grow: 1, priority: 9 },
         { id: 'health', head: 'Health', priority: 6 },
         { id: 'version', head: 'Version', mono: true, priority: 4 },
         { id: 'rps', head: 'Req/s', align: 'end', mono: true, priority: 8 },
         { id: 'p95', head: 'p95', align: 'end', mono: true, priority: 7 },
         { id: 'trend', head: 'Trend', priority: 2 },
         { id: 'cpu', head: 'CPU', priority: 3 },
      ],
      rows: () =>
         services.map(s => {
            const health = HEALTH[s.health];
            return {
               id: s.id,
               menu: ['restart', 'rollback'],
               cells: {
                  name: { text: s.name, dot: health.tone },
                  health: { badge: { text: health.label, tone: health.tone } },
                  version: s.version,
                  rps: s.rps.toLocaleString('en-US'),
                  p95: [{ t: `${s.p95} ms`, s: s.p95 > 500 ? 'warning' : '' }],
                  trend: { spark: { values: s.history, tone: health.tone } },
                  cpu: { meter: { value: s.cpu, thresholds: { warn: 0.7, bad: 0.9 } } },
               },
            };
         }),
      empty: 'No services yet.',
      detail: id => {
         const s = service(id);
         if (!s) return undefined;
         const health = HEALTH[s.health];
         return {
            head: s.name,
            meta: s.region,
            items: [
               { k: 'Health', v: [{ t: health.label, s: health.tone }] },
               { k: 'Version', v: [{ t: s.version, s: 'code' }] },
               { k: 'Deployed', v: ago(s.deployed) },
               { k: 'Owner', v: s.owner },
               {
                  k: 'Throughput',
                  v: [{ t: s.rps.toLocaleString('en-US'), s: 'mono' }, { t: ' req/s', s: 'muted' }],
               },
               { k: 'p95', v: [{ t: `${s.p95} ms`, s: s.p95 > 500 ? 'mono warning' : 'mono' }] },
               { k: 'CPU', v: [{ t: `${Math.round(s.cpu * 100)}%`, s: 'mono' }] },
            ],
            primary: s.health === 'awaiting' ? approveAction(s) : undefined,
            more: [restartAction(s)],
            danger: rollbackAction(s),
         };
      },
      menu: {
         restart: ev => restart(ev.item ?? ''),
         rollback: ev => ask('rollback', ev.item ?? ''),
      },
   },
   deploys: {
      label: 'Deploys',
      count: () => deploys.length,
      ids: () => deploys.map(d => d.id),
      cols: [
         { id: 'change', head: 'Change', grow: 1, priority: 9 },
         { id: 'service', head: 'Service', priority: 7 },
         { id: 'version', head: 'Version', mono: true, priority: 4 },
         { id: 'status', head: 'Status', priority: 8 },
         { id: 'took', head: 'Took', align: 'end', mono: true, priority: 5 },
         { id: 'at', head: 'Started', align: 'end', mono: true, priority: 6 },
      ],
      rows: () =>
         deploys.map(d => {
            const status = DEPLOY[d.status];
            return {
               id: d.id,
               cells: {
                  change: { text: d.change, dot: status.tone },
                  service: service(d.service)?.name ?? d.service,
                  version: d.version,
                  status: { badge: { text: status.label, tone: status.tone } },
                  took: [{ t: took(d.took), s: d.took === 0 ? 'muted' : '' }],
                  at: [{ t: ago(d.at), s: 'muted' }],
               },
            };
         }),
      empty: 'No deploys yet.',
      detail: id => {
         const d = deploys.find(x => x.id === id);
         const s = d && service(d.service);
         if (!d || !s) return undefined;
         const status = DEPLOY[d.status];
         const live = d.status === 'shipped' && s.version === d.version;
         return {
            head: d.id,
            meta: `${s.name} ${d.version}`,
            items: [
               { k: 'Change', v: d.change },
               { k: 'Status', v: [{ t: status.label, s: status.tone }] },
               { k: 'Version', v: [{ t: d.version, s: 'code' }] },
               { k: 'Author', v: d.author },
               { k: 'Started', v: ago(d.at) },
               { k: 'Took', v: [{ t: took(d.took), s: 'mono' }] },
            ],
            primary: d.status === 'awaiting' ? approveAction(s) : undefined,
            more: [],
            danger: live ? rollbackAction(s) : undefined,
         };
      },
   },
   incidents: {
      label: 'Incidents',
      count: () => incidents.length,
      ids: () => incidents.map(i => i.id),
      cols: [
         { id: 'title', head: 'Incident', grow: 1, priority: 9 },
         { id: 'service', head: 'Service', priority: 7 },
         { id: 'severity', head: 'Severity', priority: 8 },
         { id: 'status', head: 'Status', priority: 5 },
         { id: 'opened', head: 'Opened', align: 'end', mono: true, priority: 6 },
      ],
      rows: () =>
         incidents.map(i => {
            const severity = SEVERITY[i.severity];
            return {
               id: i.id,
               cells: {
                  title: { text: i.title, dot: severity.tone },
                  service: service(i.service)?.name ?? i.service,
                  severity: { badge: { text: severity.label, tone: severity.tone } },
                  status: i.acknowledged ? [{ t: 'Acknowledged', s: 'muted' }] : 'Open',
                  opened: [{ t: ago(i.opened), s: 'muted' }],
               },
            };
         }),
      empty: 'No open incidents. Every service is serving traffic.',
      detail: id => {
         const i = incidents.find(x => x.id === id);
         const s = i && service(i.service);
         if (!i || !s) return undefined;
         const severity = SEVERITY[i.severity];
         return {
            head: i.id,
            meta: s.name,
            items: [
               { k: 'Incident', v: i.title },
               { k: 'Severity', v: [{ t: severity.label, s: severity.tone }] },
               {
                  k: 'Status',
                  v: i.acknowledged ? 'Acknowledged' : [{ t: 'Open', s: 'pending' }],
               },
               { k: 'Opened', v: ago(i.opened) },
               { k: 'Owner', v: s.owner },
               { k: 'Service p95', v: [{ t: `${s.p95} ms`, s: 'mono' }] },
            ],
            primary: i.acknowledged
               ? { label: 'Resolve', key: 'a', icon: 'check', run: () => resolve(i.id) }
               : { label: 'Acknowledge', key: 'a', icon: 'bell', run: () => acknowledge(i.id) },
            more: [restartAction(s)],
            danger: rollbackAction(s),
         };
      },
   },
};

/** The selected row of the active view, kept on a row that exists. */
function current(): string {
   const ids = VIEW[state.view].ids();
   const id = state.selected[state.view];
   return ids.includes(id) ? id : (ids[0] ?? '');
}

/** The head row: title and meta on the left, status and Refresh on the right. */
function head() {
   const open = incidents.length;
   const regions = new Set(services.map(s => s.region)).size;
   return (
      <row key="head" justify="between" align="center">
         <text key="title">
            {span('Production services', 'title')}
            {span(
               `   ${services.length} services · ${regions} regions · updated ${ago(state.updated)}`,
               'muted'
            )}
         </text>
         <row key="end" gap="sm" align="center">
            {open > 0 ? (
               <badge key="status" tone="error">
                  {open === 1 ? '1 open incident' : `${open} open incidents`}
               </badge>
            ) : (
               <badge key="status" tone="success">
                  All clear
               </badge>
            )}
            <button
               key="refresh"
               icon="redo"
               keys={['r']}
               busy={state.refreshing}
               onClick={() => refresh()}
            >
               Refresh
            </button>
         </row>
      </row>
   );
}

/** At most four tiles, flat on the page. */
function stats() {
   const awaiting = services.filter(s => s.health === 'awaiting');
   const unhealthy = services.filter(s => s.health === 'degraded' || s.health === 'failed');
   const rps = services.reduce((sum, s) => sum + s.rps, 0);
   const worst = services.reduce((a, b) => (b.p95 > a.p95 ? b : a));
   return (
      <stats
         key="stats"
         items={[
            { label: 'Requests / s', value: rps.toLocaleString('en-US'), sub: 'all regions' },
            {
               label: 'Worst p95',
               value: `${worst.p95} ms`,
               sub: worst.name,
               tone: worst.p95 > 500 ? 'warning' : 'neutral',
            },
            {
               label: 'Unhealthy',
               value: String(unhealthy.length),
               sub: unhealthy.map(s => s.name).join(', ') || 'none',
               tone: unhealthy.length > 0 ? 'error' : 'success',
            },
            {
               label: 'Awaiting approval',
               value: String(awaiting.length),
               sub: awaiting.map(s => s.name).join(', ') || 'none',
               tone: awaiting.length > 0 ? 'pending' : 'neutral',
            },
         ]}
      />
   );
}

/** The underline tabs that switch views, each with a muted count. */
function nav() {
   return (
      <tabs
         key="nav"
         active={state.view}
         items={VIEWS.map(id => ({
            id,
            label: [VIEW[id].label, { t: ` ${VIEW[id].count()}`, s: 'muted' }],
         }))}
         onSelect={ev => pick(ev.item)}
      />
   );
}

/** The detail card of the selected row: its primary action in the head. */
function detailCard(d: Detail) {
   const { primary, danger } = d;
   return (
      <card
         key="detail"
         head={d.head}
         meta={d.meta}
         min={{ w: '44ch' }}
         max={{ w: '56ch' }}
         {...(primary && {
            end: (
               <button
                  variant="primary"
                  size="sm"
                  icon={primary.icon}
                  keys={[primary.key]}
                  onClick={primary.run}
               >
                  {primary.label}
               </button>
            ),
         })}
      >
         <kv key="kv" items={d.items} />
         {(d.more.length > 0 || danger) && (
            <row key="actions" gap="sm" align="center">
               {d.more.map(a => (
                  <button key={a.key} icon={a.icon} keys={[a.key]} onClick={a.run}>
                     {a.label}
                  </button>
               ))}
               {danger && (
                  <button
                     key="danger"
                     icon={danger.icon}
                     variant="danger"
                     keys={[danger.key]}
                     onClick={danger.run}
                  >
                     {danger.label}
                  </button>
               )}
            </row>
         )}
      </card>
   );
}

/**
 * The active view: a full-width table beside the detail card. Plain text
 * prints a `row` on one line, so the fallback stacks them instead.
 */
function body(stacked: boolean) {
   const v = VIEW[state.view];
   const id = current();
   const detail = v.detail(id);
   const table = (
      <col key="list" grow={1} basis={0}>
         <table
            key="table"
            selectable
            selected={id}
            empty={v.empty}
            cols={v.cols}
            rows={v.rows()}
            onSelect={ev => select(ev.item)}
            onActivate={ev => activate(ev.item)}
            onAction={v.menu ?? {}}
         />
      </col>
   );
   const card = detail && detailCard(detail);
   return stacked ? (
      <col key={state.view} gap="md">
         {table}
         {card}
      </col>
   ) : (
      <row key={state.view} gap="lg" align="stretch">
         {table}
         {card}
      </row>
   );
}

/** The confirm strip for the pending approval or rollback. */
function confirmStrip() {
   const c = state.confirm;
   const s = c && service(c.service);
   if (!c || !s) return null;
   const rollback = c.kind === 'rollback';
   return (
      <card
         key="confirm"
         tone={rollback ? 'error' : 'accent'}
         head={
            rollback
               ? `Roll ${s.name} back to the previous release?`
               : `Ship ${s.name} ${s.version} to production?`
         }
         end={
            <row gap="sm">
               <button key="cancel" variant="quiet" keys={['esc']} onClick={cancel}>
                  Cancel
               </button>
               <button
                  key="go"
                  variant={rollback ? 'danger' : 'primary'}
                  keys={['enter']}
                  onClick={confirm}
               >
                  {rollback ? 'Roll back' : 'Approve'}
               </button>
            </row>
         }
      >
         <text key="q">
            {span(
               rollback
                  ? 'Live traffic moves at once.'
                  : 'The rollout starts at once and cannot be paused.',
               'muted'
            )}
         </text>
      </card>
   );
}

/** The key legend, with the last note on the right. */
function legend() {
   const primary = VIEW[state.view].detail(current())?.primary;
   const keys: [string, string][] = [
      ['↑↓', 'move'],
      ['tab', 'switch view'],
   ];
   if (primary) keys.push(['enter', primary.label.toLowerCase()]);
   keys.push(['r', 'refresh'], ['q', 'quit']);
   return (
      <row key="legend" justify="between" align="center">
         <text key="keys">
            {keys.flatMap(([k, what], i) => [
               span(k, 'key'),
               span(i === keys.length - 1 ? ` ${what}` : ` ${what}   `, 'muted'),
            ])}
         </text>
         {state.note !== '' && <text key="note">{span(state.note, 'muted')}</text>}
      </row>
   );
}

/** The page's `main`, rebuilt from the data on every change. */
function main(stacked: boolean) {
   return (
      <col gap="md">
         {head()}
         {stats()}
         {nav()}
         {body(stacked)}
      </col>
   );
}

/** The whole surface: the page in `main`, the strip and legend in `dock`. */
function page() {
   return {
      main: main(false),
      dock: (
         <col gap="sm">
            {confirmStrip()}
            {legend()}
         </col>
      ),
   };
}

let render = (): void => {};

function pick(id: string): void {
   const view = VIEWS.find(v => v === id);
   if (!view) return;
   state.view = view;
   state.confirm = null;
   render();
}

function cycle(by: number): void {
   const at = VIEWS.indexOf(state.view);
   pick(VIEWS[(at + by + VIEWS.length) % VIEWS.length] ?? 'services');
}

function select(id: string): void {
   state.selected[state.view] = id;
   state.confirm = null;
   render();
}

function move(by: number): void {
   const ids = VIEW[state.view].ids();
   const at = ids.indexOf(current());
   const next = ids[Math.min(ids.length - 1, Math.max(0, at + by))];
   if (next) select(next);
}

/** Enter or a double-click: the row's primary action, else just selects it. */
function activate(id: string): void {
   const primary = VIEW[state.view].detail(id)?.primary;
   state.selected[state.view] = id;
   if (primary) primary.run();
   else select(id);
}

/** Runs the selected row's action bound to `name`; false when none is. */
function shortcut(name: string): boolean {
   const d = VIEW[state.view].detail(current());
   const action = d && [d.primary, ...d.more, d.danger].find(a => a?.key === name);
   action?.run();
   return action !== undefined;
}

function ask(kind: 'approve' | 'rollback', id: string): void {
   const s = service(id);
   if (!s || (kind === 'approve' && s.health !== 'awaiting')) return;
   state.confirm = { kind, service: id };
   render();
}

function confirm(): void {
   const c = state.confirm;
   state.confirm = null;
   if (c?.kind === 'rollback') rollback(c.service);
   else if (c?.kind === 'approve') approve(c.service);
}

function cancel(): void {
   state.confirm = null;
   render();
}

function approve(id: string): void {
   const s = service(id);
   if (!s || s.health !== 'awaiting') return;
   s.health = 'healthy';
   s.deployed = Date.now();
   for (const d of deploys) {
      if (d.service === id && d.status === 'awaiting') {
         d.status = 'shipped';
         d.took = Math.round((Date.now() - d.at) / 1000);
      }
   }
   state.note = `${s.name} ${s.version} approved and rolling out.`;
   render();
}

function restart(id: string): void {
   const s = service(id);
   if (!s) return;
   s.deployed = Date.now();
   state.note = `Restarting ${s.name}…`;
   render();
}

function rollback(id: string): void {
   const s = service(id);
   if (s) {
      const live = deploys.find(d => d.service === id && d.version === s.version);
      if (live) live.status = 'rolled-back';
      const [major, minor, patch] = s.version.slice(1).split('.').map(Number);
      s.version = `v${major}.${minor}.${Math.max(0, (patch ?? 1) - 1)}`;
      s.health = 'healthy';
      s.deployed = Date.now();
      state.note = `${s.name} rolled back to ${s.version}.`;
   }
   render();
}

function acknowledge(id: string): void {
   const i = incidents.find(x => x.id === id);
   if (!i) return;
   i.acknowledged = true;
   state.note = `${i.id} acknowledged.`;
   render();
}

function resolve(id: string): void {
   const at = incidents.findIndex(x => x.id === id);
   if (at < 0) return;
   incidents.splice(at, 1);
   state.selected.incidents = incidents[Math.min(at, incidents.length - 1)]?.id ?? '';
   state.note = `${id} resolved.`;
   render();
}

/** One tick of live numbers: traffic and latency wander, the trend scrolls. */
function tick(): void {
   for (const s of services) {
      s.rps = Math.max(1, Math.round(s.rps * (0.94 + Math.random() * 0.12)));
      s.p95 = Math.max(20, Math.round(s.p95 * (0.95 + Math.random() * 0.1)));
      s.cpu = Math.min(0.99, Math.max(0.05, s.cpu + (Math.random() - 0.5) * 0.06));
      s.history.push(s.rps);
      s.history.shift();
   }
   state.updated = Date.now();
}

function refresh(): void {
   if (state.refreshing) return;
   state.refreshing = true;
   render();
   setTimeout(() => {
      tick();
      state.refreshing = false;
      render();
   }, 600);
}

/** Applies one key; false when it quits. */
function key(k: Key): boolean {
   if (k.ctrl && k.name === 'c') return false;
   if (k.name === 'escape') {
      if (state.confirm === null) return false;
      cancel();
   } else if (k.name === 'q') return false;
   else if (k.name === 'enter') {
      if (state.confirm === null) activate(current());
      else confirm();
   } else if (k.name === 'up' || k.name === 'k') move(-1);
   else if (k.name === 'down' || k.name === 'j') move(1);
   else if (k.name === 'tab') cycle(k.shift ? -1 : 1);
   else if (k.name === ']') cycle(1);
   else if (k.name === '[') cycle(-1);
   else if (k.name === 'r') refresh();
   else shortcut(k.name);
   return true;
}

const session = await connect({ app: 'dashboard' });
if (!session) {
   console.log(plain(main(true), process.stdout.columns ?? 100));
} else {
   const surface = session.open({ mode: 'screen', title: 'Services', font: 'sans' });
   render = () => surface.render(page());
   render();
   const timer = setInterval(() => {
      tick();
      render();
   }, 1000);
   for await (const input of session) {
      if (input.type === 'key' && !key(input.key)) break;
   }
   clearInterval(timer);
   await session.close();
}
