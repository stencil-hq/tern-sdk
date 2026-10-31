"""A `screen` app in the UI face with no CSS of its own, laid out as a page:
a head row (title, muted meta, status badges and Refresh), a flat stats
strip, underline tabs switching between three views (Services, Deploys,
Incidents) and, per view, a full-width table beside a detail card for the
selected row. The confirm strip and a key legend sit in the dock. Live
numbers update every second.

Keys: ↑/↓ or j/k move, Tab/⇧Tab (or [ and ]) switch views, a approves the
selected release, Enter opens (or confirms a strip), r refreshes, Esc backs
out one layer, q quits.
Outside Tern it prints the dashboard as plain text."""

from __future__ import annotations

import random
import time
from collections.abc import Sequence
from dataclasses import dataclass, field

import tern_sdk
from tern_sdk import ActionEvent, ActivateEvent, Key, SelectEvent, ui


@dataclass
class Service:
    """One deployed service and its live numbers."""

    id: str
    name: str
    region: str
    version: str
    status: str
    """`healthy`, `degraded` or `down`."""
    cpu: float
    p95: float
    history: list[float] = field(default_factory=list)


@dataclass
class Deploy:
    """One release of a service."""

    id: str
    service: str
    version: str
    author: str
    status: str
    """`awaiting` (waiting for approval), `rolling`, `live`, `failed` or `rejected`."""
    started: float
    progress: float = 0.0
    log: list[str] = field(default_factory=list)


@dataclass
class Incident:
    """One open incident on a service."""

    id: str
    service: str
    title: str
    severity: int
    status: str
    """`open` or `monitoring`."""
    started: float
    log: list[str] = field(default_factory=list)


SERVICE_TONES: dict[str, ui.Tone] = {"healthy": "success", "degraded": "warning", "down": "error"}
DEPLOY_TONES: dict[str, ui.Tone] = {
    "awaiting": "pending", "rolling": "info", "live": "success", "failed": "error", "rejected": "muted",
}  # fmt: skip
INCIDENT_TONES: dict[str, ui.Tone] = {"open": "error", "monitoring": "warning"}
SEVERITY_TONES: dict[int, ui.Tone] = {1: "error", 2: "warning", 3: "neutral"}

VIEWS = ("services", "deploys", "incidents")
NOUNS = {"services": "service", "deploys": "release", "incidents": "incident"}


@dataclass
class Confirm:
    """A confirm strip waiting for an answer: `act` (`approve`, `reject` or `remove`) on item `id`."""

    act: str
    id: str


@dataclass
class State:
    """Everything the view draws."""

    services: list[Service]
    deploys: list[Deploy]
    incidents: list[Incident]
    tab: str = "services"
    filter: str = "all"
    """The Services view's filter: `all` or `attention`."""
    selected: dict[str, str | None] = field(default_factory=dict)
    """The selected row per view."""
    confirm: Confirm | None = None
    opened: str | None = None
    """The item whose detail card is opened (in the active view)."""
    up: float = field(default_factory=time.monotonic)
    refreshed: float = field(default_factory=time.monotonic)

    def __post_init__(self) -> None:
        for tab in VIEWS:
            ids = self.ids(tab)
            self.selected.setdefault(tab, ids[0] if ids else None)

    def visible_services(self) -> list[Service]:
        if self.filter == "attention":
            return [s for s in self.services if s.status != "healthy"]
        return self.services

    def ids(self, tab: str) -> list[str]:
        if tab == "services":
            return [s.id for s in self.visible_services()]
        if tab == "deploys":
            return [d.id for d in self.deploys]
        return [i.id for i in self.incidents]

    def selection(self) -> str | None:
        """The active view's selected row, when it is still listed."""
        id = self.selected.get(self.tab)
        return id if id in self.ids(self.tab) else None

    def service(self, id: str | None) -> Service | None:
        return next((s for s in self.services if s.id == id), None)

    def deploy(self, id: str | None) -> Deploy | None:
        return next((d for d in self.deploys if d.id == id), None)

    def incident(self, id: str | None) -> Incident | None:
        return next((i for i in self.incidents if i.id == id), None)

    def service_name(self, id: str) -> str:
        s = self.service(id)
        return s.name if s else id

    def select(self, id: str | None) -> None:
        if id != self.selected.get(self.tab):
            self.opened = None
        self.selected[self.tab] = id

    def move(self, by: int) -> None:
        ids = self.ids(self.tab)
        if not ids:
            self.select(None)
            return
        current = self.selection()
        at = ids.index(current) if current is not None else (-1 if by > 0 else len(ids))
        self.select(ids[min(max(at + by, 0), len(ids) - 1)])

    def set_filter(self, value: str) -> None:
        """Filters the Services view, keeping a listed row selected."""
        self.filter = value
        ids = self.ids("services")
        if self.selected.get("services") not in ids:
            self.selected["services"] = ids[0] if ids else None
            self.opened = None

    def switch(self, by: int) -> None:
        self.show(VIEWS[(VIEWS.index(self.tab) + by) % len(VIEWS)])

    def show(self, tab: str) -> None:
        if tab in VIEWS and tab != self.tab:
            self.tab = tab
            self.opened = None


def seed() -> State:
    rng = random.Random(7)
    now = time.monotonic()
    raw = [
        ("api", "api-gateway", "us-east-1", "v4.11.3", "healthy"),
        ("auth", "auth", "us-east-1", "v2.3.1", "healthy"),
        ("search", "search-indexer", "eu-west-1", "v1.9.4", "degraded"),
        ("billing", "billing", "us-west-2", "v2.9.8", "healthy"),
        ("media", "media-transcode", "eu-west-1", "v0.18.2", "healthy"),
        ("ledger", "ledger", "us-west-2", "v1.3.2", "healthy"),
        ("notify", "notifications", "ap-south-1", "v5.1.0", "down"),
    ]
    services = []
    for id, name, region, version, status in raw:
        history = [rng.uniform(40, 160) for _ in range(16)]
        cpu = 0.0 if status == "down" else rng.uniform(0.1, 0.9)
        services.append(Service(id, name, region, version, status, cpu, history[-1], history))
    deploys = [
        Deploy("d-2291", "billing", "v3.0.0-rc.2", "maya", "awaiting", now - 252, log=["Built", "Checks passed"]),
        Deploy("d-2290", "ledger", "v1.4.0", "omar", "awaiting", now - 611, log=["Built", "Checks passed"]),
        Deploy("d-2289", "api", "v4.12.0", "lin", "rolling", now - 95, 0.4, ["Built", "Approved by maya"]),
        Deploy("d-2287", "notify", "v5.1.0", "sam", "failed", now - 3_420, 0.3, ["Built", "Health check failed"]),
        Deploy("d-2284", "search", "v1.9.4", "maya", "live", now - 7_980, 1.0, ["Built", "Live on all hosts"]),
        Deploy("d-2280", "auth", "v2.3.1", "lin", "live", now - 86_000, 1.0, ["Built", "Live on all hosts"]),
    ]
    incidents = [
        Incident(
            "INC-412",
            "notify",
            "No heartbeat from notifications",
            1,
            "open",
            now - 245,
            ["Paged on-call", "Rollback of v5.1.0 failed"],
        ),  # fmt: skip
        Incident(
            "INC-409",
            "search",
            "Search p95 above 150 ms",
            3,
            "monitoring",
            now - 5_400,
            ["Opened by alert", "Index shard rebalanced"],
        ),  # fmt: skip
    ]
    return State(services, deploys, incidents)


def tick(state: State, rng: random.Random) -> None:
    """Moves every live number a little and rolls approved releases out."""
    for s in state.services:
        if s.status == "down":
            s.cpu = 0.0
            continue
        s.cpu = min(max(s.cpu + rng.uniform(-0.06, 0.06), 0.02), 0.98)
        s.p95 = min(max(s.p95 + rng.uniform(-12, 12), 20), 400)
        s.history = [*s.history[1:], s.p95]
    for d in state.deploys:
        if d.status != "rolling":
            continue
        d.progress = min(round(d.progress + 0.1, 2), 1.0)
        if d.progress >= 1.0:
            d.status = "live"
            d.log.append("Live on all hosts")
            if (target := state.service(d.service)) is not None:
                target.version = d.version


def ago(since: float) -> str:
    """Seconds since `since` as `45s`, `4m 12s` or `2h 05m`."""
    s = int(time.monotonic() - since)
    if s < 60:
        return f"{s}s"
    if s < 3_600:
        return f"{s // 60}m {s % 60:02d}s"
    return f"{s // 3_600}h {s % 3_600 // 60:02d}m"


# Tables: one `grow` text column each; numbers `align: end` + `mono`; no row
# action buttons. When narrow, the lowest `priority` hides first (a missing one
# is 0): the name column highest, numbers next, sparks and meters lowest.

SERVICE_COLS: list[ui.TableCol] = [
    {"id": "name", "head": "Service", "grow": 1, "priority": 9},
    {"id": "status", "head": "Status", "priority": 7},
    {"id": "region", "head": "Region", "priority": 3},
    {"id": "version", "head": "Version", "mono": True, "priority": 5},
    {"id": "trend", "head": "Latency", "priority": 2},
    {"id": "p95", "head": "p95", "align": "end", "mono": True, "priority": 8},
    {"id": "cpu", "head": "CPU", "priority": 1},
]

DEPLOY_COLS: list[ui.TableCol] = [
    {"id": "service", "head": "Service", "grow": 1, "priority": 9},
    {"id": "version", "head": "Version", "mono": True, "priority": 8},
    {"id": "status", "head": "Status", "priority": 7},
    {"id": "author", "head": "Author", "priority": 3},
    {"id": "progress", "head": "Rollout", "priority": 2},
    {"id": "age", "head": "Started", "align": "end", "mono": True, "priority": 6},
]

INCIDENT_COLS: list[ui.TableCol] = [
    {"id": "title", "head": "Incident", "grow": 1, "priority": 9},
    {"id": "id", "head": "Id", "mono": True, "priority": 2},
    {"id": "severity", "head": "Severity", "priority": 7},
    {"id": "service", "head": "Service", "priority": 4},
    {"id": "age", "head": "Open for", "align": "end", "mono": True, "priority": 8},
]


def service_rows(state: State) -> list[ui.TableRow]:
    rows: list[ui.TableRow] = []
    for s in state.visible_services():
        tone = SERVICE_TONES[s.status]
        cells: dict[str, ui.TableCell] = {
            "name": {"text": s.name, "dot": tone},
            "status": {"text": s.status, "tone": "muted"}
            if s.status == "healthy"
            else {"text": s.status, "tone": tone},
            "region": {"text": s.region, "icon": "globe"},
            "version": s.version,
            "trend": {"spark": {"values": s.history, "tone": tone}},
            "p95": "—" if s.status == "down" else f"{s.p95:.0f} ms",
            "cpu": {"meter": {"value": s.cpu}},
        }
        row: ui.TableRow = {"id": s.id, "cells": cells, "menu": ["remove"]}
        if s.status == "down":
            row["title"] = "No heartbeat for 4 minutes"
        rows.append(row)
    return rows


def deploy_rows(state: State) -> list[ui.TableRow]:
    rows: list[ui.TableRow] = []
    for d in state.deploys:
        tone = DEPLOY_TONES[d.status]
        meter: ui.MeterCellProps = {"value": d.progress}
        if d.status == "failed":
            meter["tone"] = "error"
        cells: dict[str, ui.TableCell] = {
            "service": {"text": state.service_name(d.service), "dot": tone},
            "version": d.version,
            "status": {"text": d.status, "tone": tone},
            "author": {"text": d.author, "tone": "muted"},
            "progress": {"meter": meter},
            "age": ago(d.started),
        }
        rows.append({"id": d.id, "cells": cells})
    return rows


def incident_rows(state: State) -> list[ui.TableRow]:
    rows: list[ui.TableRow] = []
    for i in state.incidents:
        cells: dict[str, ui.TableCell] = {
            "title": {"text": i.title, "dot": INCIDENT_TONES[i.status]},
            "id": {"text": i.id, "tone": "muted"},
            "severity": {"badge": {"text": f"SEV{i.severity}", "tone": SEVERITY_TONES[i.severity]}},
            "service": state.service_name(i.service),
            "age": ago(i.started),
        }
        rows.append({"id": i.id, "cells": cells})
    return rows


# Detail cards: the selected item's `kv`, its one `primary` action (sm, with
# its key) in the head's `end`, a `danger` trigger that only opens a confirm strip.


def open_button(state: State, id: str) -> ui.Node:
    if state.opened == id:
        return ui.button("Close", variant="quiet", size="sm", keys=["esc"], key="close",
                         on_click=lambda _: setattr(state, "opened", None))  # fmt: skip
    return ui.button("Open", icon="open", variant="primary", size="sm", keys=["enter"], key="open",
                     on_click=lambda _: open_selected(state))  # fmt: skip


def danger(state: State, text: str, act: str, id: str) -> ui.Node:
    return ui.row(
        ui.button(
            text,
            icon="trash" if act == "remove" else "x",
            variant="danger",
            size="sm",
            key=act,
            on_click=lambda _: ask(state, act, id),
        ),  # fmt: skip
        justify="end",
        key="danger",
    )


def timeline(lines: Sequence[str]) -> ui.Node:
    return ui.col(*(ui.text([ui.span("· ", "muted"), ui.span(line)], key=n) for n, line in enumerate(lines)),
                  gap="xs", max={"h": "12lines"}, key="log")  # fmt: skip


def service_detail(state: State, s: Service) -> ui.Node:
    tone = SERVICE_TONES[s.status]
    opened = state.opened == s.id
    chart = ui.chart(
        "spark",
        series=[{"value": v} for v in s.history],
        summary=f"p95 over the last {len(s.history)}s",
        key="chart",
    )
    return detail_card(
        s.name,
        s.id,
        open_button(state, s.id),
        ui.kv(
            [
                ("Status", [ui.span(s.status, tone)]),
                ("Region", s.region),
                ("Version", [ui.span(s.version, "mono")]),
                ("p95 latency", [ui.span("—" if s.status == "down" else f"{s.p95:.0f} ms", "mono")]),
                ("CPU", [ui.span(f"{s.cpu * 100:.0f}%", "mono")]),
            ],
            key="kv",
        ),
        chart if opened else None,
        danger(state, "Remove", "remove", s.id),
    )


def deploy_detail(state: State, d: Deploy) -> ui.Node:
    tone = DEPLOY_TONES[d.status]
    opened = state.opened == d.id
    if d.status == "awaiting":
        # Approving is this card's one primary; Enter still opens the log.
        end = ui.button(f"Approve {d.version}", icon="check", variant="primary", size="sm", keys=["a"],
                        key="approve", on_click=lambda _: ask(state, "approve", d.id))  # fmt: skip
    else:
        end = open_button(state, d.id)
    return detail_card(
        state.service_name(d.service),
        d.id,
        end,
        ui.kv(
            [
                ("Status", [ui.span(d.status, tone)]),
                ("Version", [ui.span(d.version, "mono")]),
                ("Author", d.author),
                ("Started", [ui.span(ago(d.started), "mono"), ui.span(" ago", "muted")]),
                ("Rollout", [ui.span(f"{d.progress * 100:.0f}%", "mono")]),
            ],
            key="kv",
        ),
        ui.progress(d.progress, key="progress") if d.status == "rolling" else None,
        timeline(d.log) if opened else None,
        danger(state, "Reject", "reject", d.id) if d.status == "awaiting" else None,
    )


def incident_detail(state: State, i: Incident) -> ui.Node:
    opened = state.opened == i.id
    return detail_card(
        i.title,
        i.id,
        open_button(state, i.id),
        ui.kv(
            [
                ("Status", [ui.span(i.status, INCIDENT_TONES[i.status])]),
                ("Severity", [ui.span(f"SEV{i.severity}", SEVERITY_TONES[i.severity])]),
                ("Service", state.service_name(i.service)),
            ],
            key="kv",
        ),
        ui.row(
            ui.text([ui.span("Open for", "muted")]),
            # A live timer: Tern counts it up between renders.
            ui.elapsed((time.monotonic() - i.started) * 1000),
            gap="sm",
            align="center",
            key="age",
        ),
        timeline(i.log) if opened else None,
    )


def detail_card(head: str, meta: str, end: ui.Node, *body: ui.Node | None) -> ui.Node:
    return ui.card(
        *body,
        head=head,
        meta=meta,
        end=end,
        min={"w": "44ch"},
        max={"w": "56ch"},
        key="detail",
    )


def detail(state: State) -> ui.Node:
    id = state.selection()
    item: ui.Node | None = None
    if state.tab == "services" and (s := state.service(id)) is not None:
        item = service_detail(state, s)
    elif state.tab == "deploys" and (d := state.deploy(id)) is not None:
        item = deploy_detail(state, d)
    elif state.tab == "incidents" and (i := state.incident(id)) is not None:
        item = incident_detail(state, i)
    if item is not None:
        return item
    return ui.card(
        ui.text([ui.span(f"Select a {NOUNS[state.tab]} to see its details.", "muted")]),
        head="Details",
        min={"w": "44ch"},
        max={"w": "56ch"},
        key="detail",
    )


# Confirm strips and actions.


def confirm_strip(state: State, confirm: Confirm) -> ui.Node:
    """A destructive act asks in an error strip with a `danger` button; an
    approval (consequential, not destructive) in an accent strip with a `primary` one."""
    if confirm.act == "remove":
        name = state.service_name(confirm.id)
        question, verb, destructive = f"Remove {name}? Its traffic is dropped and its history is lost.", "Remove", True
    else:
        d = state.deploy(confirm.id)
        release = f"{state.service_name(d.service)} {d.version}" if d else confirm.id
        if confirm.act == "reject":
            question, verb, destructive = f"Reject {release}? The release is discarded.", "Reject", True
        else:
            question, verb, destructive = f"Approve {release}? It rolls out to production now.", "Approve", False
    return ui.card(
        head=question,
        end=ui.row(
            ui.button("Cancel", variant="quiet", keys=["esc"], key="cancel", on_click=lambda _: cancel(state)),
            ui.button(
                verb,
                variant="danger" if destructive else "primary",
                keys=["enter"],
                key="ok",
                on_click=lambda _: confirmed(state),
            ),  # fmt: skip
            gap="sm",
        ),
        tone="error" if destructive else "accent",
        key="confirm",
    )


def ask(state: State, act: str, id: str | None) -> None:
    if id is None:
        return
    if act in ("approve", "reject") and ((d := state.deploy(id)) is None or d.status != "awaiting"):
        return
    if act == "remove" and state.service(id) is None:
        return
    state.select(id)
    state.confirm = Confirm(act, id)


def cancel(state: State) -> None:
    state.confirm = None


def confirmed(state: State) -> None:
    confirm, state.confirm = state.confirm, None
    if confirm is None:
        return
    if confirm.act == "remove":
        # The row taking the removed one's place becomes the selection.
        ids = state.ids("services")
        at = ids.index(confirm.id) if confirm.id in ids else 0
        state.services = [s for s in state.services if s.id != confirm.id]
        state.incidents = [i for i in state.incidents if i.service != confirm.id]
        left = state.ids("services")
        state.select(left[min(at, len(left) - 1)] if left else None)
    elif (d := state.deploy(confirm.id)) is not None:
        if confirm.act == "approve":
            d.status, d.progress, d.started = "rolling", 0.0, time.monotonic()
            d.log.append("Approved")
        else:
            d.status = "rejected"
            d.log.append("Rejected")


def open_selected(state: State) -> None:
    state.opened = state.selection()


def refresh(state: State) -> None:
    state.refreshed = time.monotonic()


def back_out(state: State) -> bool:
    """Escape: closes the innermost layer (confirm strip, then an opened
    detail); `False` when there is nothing to back out of."""
    if state.confirm is not None:
        state.confirm = None
    elif state.opened is not None:
        state.opened = None
    else:
        return False
    return True


# The page.


def head(state: State) -> ui.Node:
    """The head row: title and muted meta on the left, status badges and Refresh on the right."""
    down = sum(s.status == "down" for s in state.services)
    meta = f"   production · up {ago(state.up)} · refreshed {ago(state.refreshed)} ago"
    return ui.row(
        ui.text([ui.span("Deployments", "title"), ui.span(meta, "muted")], key="t"),
        ui.row(
            ui.badge(f"{down} down", tone="error") if down else None,
            ui.badge("Live", tone="success"),
            ui.button(
                "Refresh", icon="repeat", size="sm", keys=["r"], key="refresh", on_click=lambda _: refresh(state)
            ),  # fmt: skip
            gap="sm",
            align="center",
            key="end",
        ),
        justify="between",
        align="center",
        key="head",
    )


def stats(state: State) -> ui.Node:
    services = state.services
    healthy = sum(s.status == "healthy" for s in services)
    awaiting = [d for d in state.deploys if d.status == "awaiting"]
    live = [s.p95 for s in services if s.status != "down"]
    worst = max(live, default=0)
    waiting: ui.StatItem = {"label": "Awaiting approval", "value": str(len(awaiting))}
    if awaiting:
        waiting["tone"] = "pending"
        waiting["sub"] = ", ".join(state.service_name(d.service) for d in awaiting)
    incidents: ui.StatItem = {"label": "Open incidents", "value": str(len(state.incidents))}
    if state.incidents:
        incidents["tone"] = "error"
        incidents["sub"] = f"oldest {ago(min(i.started for i in state.incidents))}"
    return ui.stats(
        [
            {"label": "Healthy", "value": f"{healthy}/{len(services)}", "tone": "success", "sub": "services"},
            waiting,
            incidents,
            {"label": "Worst p95", "value": f"{worst:.0f} ms", "sub": "last minute"},
        ],
        key="stats",
    )


def nav(state: State) -> ui.Node:
    counts = {"services": len(state.services), "deploys": len(state.deploys), "incidents": len(state.incidents)}
    items: list[ui.Tab] = [
        {"id": tab, "label": [ui.span(f"{tab.capitalize()} "), ui.span(str(counts[tab]), "muted")]} for tab in VIEWS
    ]
    return ui.tabs(items, active=state.tab, key="nav", on_select=lambda ev: state.show(ev.item or state.tab))


def toolbar(state: State) -> ui.Node | None:
    """The Services view's toolbar: a muted description left, a segmented filter right."""
    if state.tab != "services":
        return None
    attention = sum(s.status != "healthy" for s in state.services)
    return ui.row(
        ui.text([ui.span("Production services, live", "muted")]),
        ui.tabs(
            [{"id": "all", "label": "All"}, {"id": "attention", "label": f"Attention {attention}"}],
            active=state.filter,
            variant="segmented",
            on_select=lambda ev: state.set_filter(ev.item or "all"),
        ),
        justify="between",
        align="center",
        key="toolbar",
    )


def table(state: State) -> ui.Node:
    def on_select(ev: SelectEvent) -> None:
        state.select(ev.item)

    def on_activate(ev: ActivateEvent) -> None:
        state.select(ev.item)
        open_selected(state)

    def on_action(ev: ActionEvent) -> None:
        ask(state, "remove", ev.item)

    if state.tab == "services":
        cols, rows = SERVICE_COLS, service_rows(state)
        empty = "Nothing needs attention." if state.filter == "attention" else "No services deployed."
    elif state.tab == "deploys":
        cols, rows, empty = DEPLOY_COLS, deploy_rows(state), "No releases in the last day."
    else:
        cols, rows, empty = INCIDENT_COLS, incident_rows(state), "No open incidents."
    return ui.table(
        cols,
        rows,
        selectable=True,
        selected=state.selection(),
        empty=empty,
        key=state.tab,
        on_select=on_select,
        on_activate=on_activate,
        on_action={"remove": on_action},
    )


def legend(state: State) -> ui.Node:
    """The key legend: a muted text with `key` spans."""
    if state.confirm is not None:
        pairs = [("enter", "confirm"), ("esc", "cancel")]
    else:
        pairs = [("↑↓", "move"), ("tab", "views"), ("enter", "open")]
        if state.tab == "deploys":
            pairs.append(("a", "approve"))
        pairs += [("r", "refresh"), ("esc", "back") if state.opened else ("q", "quit")]
    spans: list[ui.Span | str] = []
    for n, (k, what) in enumerate(pairs):
        spans += [ui.span(k, "key"), ui.span(f" {what}" + ("   " if n < len(pairs) - 1 else ""), "muted")]
    return ui.text(spans, key="legend")


def view(state: State) -> dict[str, list[ui.Node]]:
    """The page: head row, stats strip and tabs over the active view (table
    beside the detail card) in `main`; the confirm strip and key legend in the dock."""
    # Master/detail: the table takes the room left by the 44–56ch detail card.
    master = ui.col(toolbar(state), table(state), gap="sm", grow=1, basis=0, key="list")
    body = ui.row(master, detail(state), align="stretch", gap="lg", key="view")
    dock = [legend(state)]
    if state.confirm is not None:
        dock.insert(0, confirm_strip(state, state.confirm))
    return {"main": [head(state), stats(state), nav(state), body], "dock": dock}


def plain_view(state: State) -> list[ui.Node]:
    """Outside Tern the page stacks: plain text has no room for side by side."""
    return [head(state), stats(state), nav(state), table(state), detail(state)]


def main() -> None:
    state = seed()
    rng = random.Random()
    session = tern_sdk.connect(app="dashboard")
    if session is None:
        print(tern_sdk.plain(plain_view(state)))
        return
    with session, session.open(mode="screen", title="Deployments", font="sans") as surface:
        surface.render(view(state))
        next_tick = time.monotonic() + 1
        while True:
            item = session.poll(max(next_tick - time.monotonic(), 0))
            if item is None and time.monotonic() < next_tick:
                break  # end of input
            if time.monotonic() >= next_tick:
                tick(state, rng)
                next_tick = time.monotonic() + 1
            if isinstance(item, Key):
                if (item.ctrl and item.name in ("c", "d")) or item.name == "q":
                    break
                if item.name == "escape":
                    if not back_out(state):
                        break
                elif item.name == "enter":
                    if state.confirm is not None:
                        confirmed(state)
                    else:
                        open_selected(state)
                elif state.confirm is not None:
                    pass  # a confirm strip takes only Enter or Escape
                elif item.name == "tab":
                    state.switch(-1 if item.shift else 1)
                elif item.name in ("[", "]"):
                    state.switch(-1 if item.name == "[" else 1)
                elif item.name in ("down", "j"):
                    state.move(1)
                elif item.name in ("up", "k"):
                    state.move(-1)
                elif item.name == "a" and state.tab == "deploys":
                    ask(state, "approve", state.selection())
                elif item.name == "r":
                    refresh(state)
            if surface.closed:
                break
            surface.render(view(state))


if __name__ == "__main__":
    main()
