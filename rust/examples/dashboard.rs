//! A `screen` dashboard in the UI face (`font: sans`) with no CSS of its own,
//! laid out as a page: a head row (the title, muted meta, a status badge and
//! Refresh at its end), a flat stats strip, underline tabs with a view per tab
//! (Services, Deploys, Incidents). A view is a full-width table with one
//! growing text column beside a detail card for the selected row: its `kv`,
//! the one `primary` action with its key in the card head's end and a
//! `danger` trigger that opens a confirm strip in the dock, above the key
//! legend. It updates on a timer. Keys: Tab and ⇧Tab switch views, ↑/↓ or
//! j/k move, Enter opens, a runs the selected row's action, r refreshes; in
//! the confirm strip Enter confirms and Esc backs out; otherwise Esc or q
//! quits.
//! Outside Tern it prints the dashboard as plain text.

use std::time::{Duration, Instant};

use tern_sdk::{
	Font, Input, Options, Session, SurfaceOptions, nodes, spans,
	ui::{
		self, BadgeSpec, Bound, Column, Extent, MeterCell, Node, SparkCell, StatItem, TableRow, Text,
		TextAlign, TextCell, Tone, View,
	},
};

/// How often the numbers move.
const TICK: Duration = Duration::from_secs(1);

/// How long the fleet had been up when the dashboard started, in seconds.
const UPTIME: u64 = 39 * 60;

/// A view of the page, one per tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Page {
	/// The services and their health.
	Services,
	/// The deploys rolling out.
	Deploys,
	/// The incidents.
	Incidents,
}

impl Page {
	/// The tabs in order.
	const ALL: [Self; 3] = [Self::Services, Self::Deploys, Self::Incidents];

	/// The tab id.
	const fn id(self) -> &'static str {
		match self {
			Self::Services => "services",
			Self::Deploys => "deploys",
			Self::Incidents => "incidents",
		}
	}

	/// The page with tab id `id`.
	fn from_id(id: &str) -> Option<Self> {
		Self::ALL.into_iter().find(|p| p.id() == id)
	}

	/// Its place among the tabs.
	const fn index(self) -> usize {
		match self {
			Self::Services => 0,
			Self::Deploys => 1,
			Self::Incidents => 2,
		}
	}

	/// The next tab, or the previous one with `back`, wrapping around.
	const fn turn(self, back: bool) -> Self {
		let n = Self::ALL.len();
		Self::ALL[(self.index() + if back { n - 1 } else { 1 }) % n]
	}
}

/// A service's health.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Health {
	/// Serving normally.
	Healthy,
	/// Slow or erroring.
	Degraded,
	/// Drained: no replicas.
	Down,
}

impl Health {
	/// The label and the tone of its dot and badge.
	const fn look(self) -> (&'static str, Tone) {
		match self {
			Self::Healthy => ("Healthy", Tone::Success),
			Self::Degraded => ("Degraded", Tone::Warning),
			Self::Down => ("Down", Tone::Error),
		}
	}
}

/// A deploy's state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
	/// Rolling out.
	Running,
	/// Built, waiting for someone to approve it.
	Waiting,
	/// Failed its checks.
	Failed,
	/// Live everywhere.
	Done,
}

impl State {
	/// The label and the tone of its dot and badge.
	const fn look(self) -> (&'static str, Tone) {
		match self {
			Self::Running => ("Rolling out", Tone::Accent),
			Self::Waiting => ("Needs approval", Tone::Pending),
			Self::Failed => ("Failed", Tone::Error),
			Self::Done => ("Live", Tone::Success),
		}
	}
}

/// How bad an incident is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Severity {
	/// Customers can't work.
	Critical,
	/// Something is broken for some.
	Major,
	/// A nuisance.
	Minor,
}

impl Severity {
	/// The label and the tone of its dot and badge.
	const fn look(self) -> (&'static str, Tone) {
		match self {
			Self::Critical => ("SEV1", Tone::Error),
			Self::Major => ("SEV2", Tone::Warning),
			Self::Minor => ("SEV3", Tone::Neutral),
		}
	}
}

/// Where an incident is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
	/// Nobody has picked it up.
	Open,
	/// Someone is on it.
	Acknowledged,
	/// Fixed.
	Resolved,
}

impl Phase {
	/// The label and the tone of its badge.
	const fn look(self) -> (&'static str, Tone) {
		match self {
			Self::Open => ("Open", Tone::Error),
			Self::Acknowledged => ("Acknowledged", Tone::Pending),
			Self::Resolved => ("Resolved", Tone::Success),
		}
	}
}

/// A destructive action behind the confirm strip.
struct Risk {
	/// The trigger's and the confirm button's label.
	button:  &'static str,
	/// The confirm strip's question.
	ask:     String,
	/// What happens, under the question.
	then:    &'static str,
	/// The note once done.
	done:    &'static str,
	/// Why it can't be done now, if it can't.
	blocked: Option<&'static str>,
}

/// One service.
#[derive(Clone, Debug)]
struct Service {
	/// Row id and name.
	id:       &'static str,
	/// The region.
	region:   &'static str,
	/// The running version.
	version:  &'static str,
	/// Running replicas.
	replicas: u32,
	/// How it's doing.
	health:   Health,
	/// Requests per second, oldest first.
	rps:      Vec<f64>,
	/// 95th percentile latency, in milliseconds.
	p95:      f64,
	/// Share of requests failing, in percent.
	errors:   f64,
}

impl Service {
	/// The label of its primary action.
	const fn action(&self) -> &'static str {
		match self.health {
			Health::Down => "Start",
			Health::Healthy | Health::Degraded => "Scale up",
		}
	}

	/// Runs its primary action; returns the note.
	const fn act(&mut self) -> &'static str {
		if matches!(self.health, Health::Down) {
			self.replicas = 2;
			self.health = Health::Healthy;
			return "Started";
		}
		self.replicas += 1;
		self.health = Health::Healthy;
		"Scaled up"
	}

	/// Draining it.
	fn risk(&self) -> Risk {
		Risk {
			button:  "Drain",
			ask:     format!("Drain {}?", self.id),
			then:    "It stops taking traffic until you start it again.",
			done:    "Drained",
			blocked: matches!(self.health, Health::Down).then_some("Already drained"),
		}
	}
}

/// One deploy.
#[derive(Clone, Debug)]
struct Deploy {
	/// Row id.
	id:      &'static str,
	/// The service.
	service: &'static str,
	/// The commit.
	commit:  &'static str,
	/// The region.
	region:  &'static str,
	/// Where it is.
	state:   State,
	/// Rollout progress, 0–1.
	done:    f64,
	/// Requests per second, oldest first.
	rps:     Vec<f64>,
}

impl Deploy {
	/// The label of its primary action.
	const fn action(&self) -> Option<&'static str> {
		match self.state {
			State::Waiting => Some("Approve"),
			State::Failed => Some("Retry"),
			State::Running | State::Done => None,
		}
	}

	/// Runs its primary action; returns the note.
	const fn act(&mut self) -> Option<&'static str> {
		match self.state {
			State::Waiting => {
				self.state = State::Running;
				self.done = 0.02;
				Some("Approved")
			},
			State::Failed => {
				self.state = State::Running;
				self.done = 0.0;
				Some("Retrying")
			},
			State::Running | State::Done => None,
		}
	}

	/// Cancelling it.
	fn risk(&self) -> Risk {
		Risk {
			button:  "Cancel deploy",
			ask:     format!("Cancel the {} deploy?", self.id),
			then:    "Its rollout stops where it is.",
			done:    "Cancelled",
			blocked: matches!(self.state, State::Done).then_some("Already live"),
		}
	}
}

/// One incident.
#[derive(Clone, Debug)]
struct Incident {
	/// Row id.
	id:       &'static str,
	/// What's wrong.
	title:    &'static str,
	/// The service it hits.
	service:  &'static str,
	/// How bad it is.
	severity: Severity,
	/// Where it is.
	phase:    Phase,
	/// Seconds since it opened.
	age:      u64,
}

impl Incident {
	/// The label of its primary action.
	const fn action(&self) -> Option<&'static str> {
		match self.phase {
			Phase::Open => Some("Acknowledge"),
			Phase::Acknowledged => Some("Resolve"),
			Phase::Resolved => None,
		}
	}

	/// Runs its primary action; returns the note.
	const fn act(&mut self) -> Option<&'static str> {
		match self.phase {
			Phase::Open => {
				self.phase = Phase::Acknowledged;
				Some("Acknowledged")
			},
			Phase::Acknowledged => {
				self.phase = Phase::Resolved;
				Some("Resolved")
			},
			Phase::Resolved => None,
		}
	}

	/// Dismissing it.
	fn risk(&self) -> Risk {
		Risk {
			button:  "Dismiss",
			ask:     format!("Dismiss {}?", self.id),
			then:    "It leaves the list without a postmortem.",
			done:    "Dismissed",
			blocked: None,
		}
	}
}

/// What the view's handlers report.
#[derive(Clone, Debug)]
enum Msg {
	/// A tab of the page was picked.
	Page(String),
	/// The deploys filter was picked.
	Filter(String),
	/// Refresh was clicked.
	Refresh,
	/// A row was selected.
	Select(Page, String),
	/// A row was opened (double click).
	Open(Page, String),
	/// The detail card's primary action.
	Act(Page, String),
	/// The detail card's `danger` trigger (shows the confirm strip).
	Ask(Page, String),
	/// The confirm strip's destructive button.
	Confirm,
	/// The confirm strip's Keep button.
	Keep,
}

/// The dashboard's state.
struct Dash {
	/// All services.
	services:   Vec<Service>,
	/// All deploys.
	deploys:    Vec<Deploy>,
	/// All incidents.
	incidents:  Vec<Incident>,
	/// The tab shown.
	page:       Page,
	/// The deploys filter: `all` or `attention`.
	filter:     String,
	/// The selected row of each page.
	selected:   [Option<&'static str>; 3],
	/// The row the confirm strip asks about.
	confirming: Option<(Page, &'static str)>,
	/// A note about the last thing that happened.
	note:       String,
	/// Ticks so far (drives the fake traffic).
	ticks:      u32,
}

/// Sample traffic around `base`, twelve points.
fn traffic(base: f64, seed: u32) -> Vec<f64> {
	(0..12u32)
		.map(|i| base + f64::from((i * 7 + seed) % 23))
		.collect()
}

/// `secs` as a short duration: `42s`, `4m 05s`, `1h 03m`.
fn ago(secs: u64) -> String {
	match secs {
		0..60 => format!("{secs}s"),
		60..3600 => format!("{}m {:02}s", secs / 60, secs % 60),
		_ => format!("{}h {:02}m", secs / 3600, secs % 3600 / 60),
	}
}

/// `v` in the mono face.
fn mono(v: impl Into<String>) -> Text {
	ui::span(v).style("mono").into()
}

impl Dash {
	/// Sample data.
	fn new() -> Self {
		let service = |id, region, version, replicas, health, p95, errors, seed| Service {
			id,
			region,
			version,
			replicas,
			health,
			rps: traffic(120.0, seed),
			p95,
			errors,
		};
		let deploy = |id: &'static str, service, commit, region, state, done, seed| Deploy {
			id,
			service,
			commit,
			region,
			state,
			done,
			rps: traffic(40.0, seed),
		};
		let incident = |id, title, service, severity, phase, age| Incident {
			id,
			title,
			service,
			severity,
			phase,
			age,
		};
		Self {
			services:   vec![
				service("api-gateway", "us-east-1", "2.14.0", 6, Health::Healthy, 84.0, 0.1, 3),
				service("auth", "eu-west-1", "1.9.2", 3, Health::Healthy, 41.0, 0.0, 8),
				service("search-indexer", "us-west-2", "0.31.4", 2, Health::Degraded, 412.0, 2.7, 13),
				service("web", "us-east-1", "5.0.1", 8, Health::Healthy, 118.0, 0.2, 5),
				service("billing", "ap-south-1", "3.3.0", 2, Health::Healthy, 67.0, 0.0, 11),
				service("notifications", "eu-west-1", "1.2.7", 0, Health::Down, 0.0, 0.0, 2),
			],
			deploys:    vec![
				deploy("api", "api-gateway", "4f2c9e1", "us-east-1", State::Running, 0.42, 3),
				deploy("auth", "auth", "b71d03a", "eu-west-1", State::Waiting, 0.0, 8),
				deploy("search", "search-indexer", "09ae5f7", "us-west-2", State::Failed, 0.63, 13),
				deploy("web", "web", "e3c1a44", "us-east-1", State::Done, 1.0, 5),
				deploy("billing", "billing", "5d8e2b0", "ap-south-1", State::Waiting, 0.0, 11),
			],
			incidents:  vec![
				incident(
					"INC-311",
					"Search results lag behind writes",
					"search-indexer",
					Severity::Major,
					Phase::Open,
					14 * 60 + 12,
				),
				incident(
					"INC-310",
					"Notification emails not sent",
					"notifications",
					Severity::Critical,
					Phase::Acknowledged,
					52 * 60 + 40,
				),
				incident(
					"INC-307",
					"Slow logins from India",
					"auth",
					Severity::Minor,
					Phase::Resolved,
					5 * 3600 + 18 * 60,
				),
			],
			page:       Page::Services,
			filter:     "all".into(),
			selected:   [Some("api-gateway"), Some("api"), Some("INC-311")],
			confirming: None,
			note:       String::new(),
			ticks:      0,
		}
	}

	/// The deploys the filter shows.
	fn shown_deploys(&self) -> Vec<&Deploy> {
		self
			.deploys
			.iter()
			.filter(|d| self.filter == "all" || matches!(d.state, State::Waiting | State::Failed))
			.collect()
	}

	/// The ids of the rows `page` shows, in order.
	fn ids(&self, page: Page) -> Vec<&'static str> {
		match page {
			Page::Services => self.services.iter().map(|s| s.id).collect(),
			Page::Deploys => self.shown_deploys().iter().map(|d| d.id).collect(),
			Page::Incidents => self.incidents.iter().map(|i| i.id).collect(),
		}
	}

	/// Moves `page`'s selection by `by` rows; `0` keeps a selection still
	/// shown and otherwise picks the first row.
	fn pick(&mut self, page: Page, by: isize) {
		let ids = self.ids(page);
		let slot = &mut self.selected[page.index()];
		let Some(last) = ids.len().checked_sub(1) else {
			*slot = None;
			return;
		};
		let at = slot
			.and_then(|s| ids.iter().position(|id| *id == s))
			.map_or(0, |i| i.saturating_add_signed(by).min(last));
		*slot = Some(ids[at]);
	}

	/// Selects row `id` of `page` if it is there.
	fn select(&mut self, page: Page, id: &str) {
		self.selected[page.index()] = self.ids(page).into_iter().find(|s| *s == id);
	}

	/// The destructive action on row `id` of `page`.
	fn risk(&self, page: Page, id: &str) -> Option<Risk> {
		match page {
			Page::Services => self.services.iter().find(|s| s.id == id).map(Service::risk),
			Page::Deploys => self.deploys.iter().find(|d| d.id == id).map(Deploy::risk),
			Page::Incidents => self
				.incidents
				.iter()
				.find(|i| i.id == id)
				.map(Incident::risk),
		}
	}

	/// Advances the rollouts, the traffic and the clocks one tick.
	fn tick(&mut self) {
		self.ticks += 1;
		let ticks = self.ticks;
		for (s, i) in self.services.iter_mut().zip(0u32..) {
			let swing = f64::from((ticks * 5 + i * 11) % 17);
			s.rps.remove(0);
			if s.health == Health::Down {
				s.rps.push(0.0);
				s.p95 = 0.0;
			} else {
				s.rps.push(110.0 + swing * 3.0);
				let base = if s.health == Health::Degraded {
					380.0
				} else {
					60.0
				};
				s.p95 = base + swing * 4.0;
			}
		}
		for (d, i) in self.deploys.iter_mut().zip(0u32..) {
			let swing = f64::from((ticks * 5 + i * 11) % 17);
			d.rps.remove(0);
			d.rps.push(38.0 + swing);
			if d.state == State::Running {
				d.done = (d.done + 0.04).min(1.0);
				if d.done >= 1.0 {
					d.state = State::Done;
				}
			}
		}
		for inc in &mut self.incidents {
			if inc.phase != Phase::Resolved {
				inc.age += TICK.as_secs();
			}
		}
	}

	/// Applies a message from the view.
	fn update(&mut self, msg: Msg) {
		match msg {
			Msg::Page(id) => {
				if let Some(page) = Page::from_id(&id) {
					self.page = page;
				}
			},
			Msg::Filter(filter) => {
				self.filter = filter;
				self.pick(Page::Deploys, 0);
			},
			Msg::Refresh => {
				self.tick();
				self.note = "Refreshed".into();
			},
			Msg::Select(page, id) => self.select(page, &id),
			Msg::Open(page, id) => {
				self.select(page, &id);
				self.note = format!("Opened {id}");
			},
			Msg::Act(page, id) => {
				let done = match page {
					Page::Services => self
						.services
						.iter_mut()
						.find(|s| s.id == id)
						.map(Service::act),
					Page::Deploys => self
						.deploys
						.iter_mut()
						.find(|d| d.id == id)
						.and_then(Deploy::act),
					Page::Incidents => self
						.incidents
						.iter_mut()
						.find(|i| i.id == id)
						.and_then(Incident::act),
				};
				if let Some(done) = done {
					self.note = format!("{done} {id}");
				}
				self.pick(page, 0);
			},
			Msg::Ask(page, id) => {
				self.confirming = self
					.ids(page)
					.into_iter()
					.find(|s| *s == id)
					.filter(|_| self.risk(page, &id).is_some_and(|r| r.blocked.is_none()))
					.map(|s| (page, s));
			},
			Msg::Confirm => {
				let Some((page, id)) = self.confirming.take() else {
					return;
				};
				let Some(risk) = self.risk(page, id) else {
					return;
				};
				match page {
					Page::Services => {
						if let Some(s) = self.services.iter_mut().find(|s| s.id == id) {
							s.replicas = 0;
							s.health = Health::Down;
						}
					},
					Page::Deploys => self.deploys.retain(|d| d.id != id),
					Page::Incidents => self.incidents.retain(|i| i.id != id),
				}
				self.note = format!("{} {id}", risk.done);
				self.pick(page, 0);
			},
			Msg::Keep => self.confirming = None,
		}
	}

	/// Runs the selected row's primary action (key `a`).
	fn act_selected(&mut self) {
		if let Some(id) = self.selected[self.page.index()] {
			self.update(Msg::Act(self.page, id.into()));
		}
	}

	/// The whole screen. `plain` stacks the detail under the table instead
	/// of beside it, for the plain-text fallback.
	fn view(&self, plain: bool) -> View<Msg> {
		let main = nodes![self.head(), self.stats().key("stats"), self.nav(), self.body(plain)];
		let mut dock: Vec<Node<Msg>> = Vec::new();
		if let Some((page, id)) = self.confirming
			&& let Some(risk) = self.risk(page, id)
		{
			dock.push(confirm(&risk).into());
		}
		dock.push(self.legend().into());
		View::new().main(main).dock(dock)
	}

	/// The head row: the title and muted meta, a status badge and Refresh.
	fn head(&self) -> ui::Row<Msg> {
		let title = ui::text(spans![
			ui::span("Fleet").style("title"),
			" ",
			ui::span(format!(
				"production · {} services · up {}",
				self.services.len(),
				ago(UPTIME + u64::from(self.ticks))
			))
			.style("muted"),
		])
		.key("title");
		let worst = self
			.services
			.iter()
			.map(|s| s.health)
			.max_by_key(|h| match h {
				Health::Healthy => 0,
				Health::Degraded => 1,
				Health::Down => 2,
			})
			.unwrap_or(Health::Healthy);
		let status = match worst {
			Health::Healthy => ui::badge("Operational").tone(Tone::Success),
			Health::Degraded => ui::badge("Degraded").tone(Tone::Warning),
			Health::Down => ui::badge("Partial outage").tone(Tone::Error),
		};
		let refresh = ui::button("Refresh")
			.key("refresh")
			.icon("repeat")
			.keys(["r"])
			.on_click(Msg::Refresh);
		ui::row()
			.key("head")
			.justify("between")
			.align("center")
			.child(title)
			.child(
				ui::row()
					.key("end")
					.gap("sm")
					.align("center")
					.children(nodes![status.key("status"), refresh]),
			)
	}

	/// The stats strip: at most four tiles, directly on the page.
	fn stats(&self) -> ui::Stats<Msg> {
		let total = self.services.len();
		let healthy = self
			.services
			.iter()
			.filter(|s| s.health == Health::Healthy)
			.count();
		let deploys = |state| self.deploys.iter().filter(|d| d.state == state).count();
		let open = self
			.incidents
			.iter()
			.filter(|i| i.phase != Phase::Resolved)
			.count();
		let unacked = self
			.incidents
			.iter()
			.filter(|i| i.phase == Phase::Open)
			.count();
		let rps: f64 = self.services.iter().filter_map(|s| s.rps.last()).sum();
		let mut services = StatItem::new("Healthy", format!("{healthy}/{total}")).sub("services");
		if healthy < total {
			services = services.tone(Tone::Warning);
		}
		let mut incidents =
			StatItem::new("Open incidents", open.to_string()).sub(format!("{unacked} unacknowledged"));
		if unacked > 0 {
			incidents = incidents.tone(Tone::Error);
		}
		ui::stats([
			services,
			StatItem::new("Rolling out", deploys(State::Running).to_string())
				.sub(format!("{} need approval", deploys(State::Waiting))),
			incidents,
			StatItem::new("Traffic", format!("{rps:.0}")).sub("req/s"),
		])
	}

	/// The tabs: one per view, with muted counts.
	fn nav(&self) -> ui::Tabs<Msg> {
		let count = |n: usize| ui::span(n.to_string()).style("muted");
		let open = self
			.incidents
			.iter()
			.filter(|i| i.phase != Phase::Resolved)
			.count();
		ui::tabs()
			.key("nav")
			.active(self.page.id())
			.tab("services", spans!["Services ", count(self.services.len())])
			.tab("deploys", spans!["Deploys ", count(self.deploys.len())])
			.tab("incidents", spans!["Incidents ", count(open)])
			.on_select(|id| Msg::Page(id.to_owned()))
	}

	/// The active view: its table beside the selected row's detail card
	/// (under it when `plain`).
	fn body(&self, plain: bool) -> Node<Msg> {
		let page = self.page;
		let selected = self.selected[page.index()];
		let (list, detail) = match page {
			Page::Services => (
				nodes![self.services_table()],
				selected
					.and_then(|id| self.services.iter().find(|s| s.id == id))
					.map(service_card),
			),
			Page::Deploys => (
				nodes![self.deploys_bar(), self.deploys_table()],
				selected
					.and_then(|id| self.deploys.iter().find(|d| d.id == id))
					.map(deploy_card),
			),
			Page::Incidents => (
				nodes![self.incidents_table()],
				selected
					.and_then(|id| self.incidents.iter().find(|i| i.id == id))
					.map(incident_card),
			),
		};
		let mut parts: Vec<Node<Msg>> = nodes![
			ui::col()
				.key("list")
				.gap("sm")
				.grow(1.0)
				.basis(0.0)
				.children(list)
		];
		if let Some(card) = detail {
			parts.push(
				card
					.key("detail")
					.min(Bound::w(Extent::Ch(44.0)))
					.max(Bound::w(Extent::Ch(56.0)))
					.into(),
			);
		}
		if plain {
			ui::col().key(page.id()).gap("md").children(parts).into()
		} else {
			ui::row()
				.key(page.id())
				.align("stretch")
				.gap("lg")
				.children(parts)
				.into()
		}
	}

	/// A selectable table of `page` with its handlers and selection.
	fn table(&self, page: Page, empty: &str) -> ui::Table<Msg> {
		let mut table = ui::table()
			.key("table")
			.selectable(true)
			.empty(empty)
			.on_select(move |id| Msg::Select(page, id.to_owned()))
			.on_activate(move |id| Msg::Open(page, id.to_owned()));
		if let Some(id) = self.selected[page.index()] {
			table = table.selected(id);
		}
		table
	}

	/// The services table.
	fn services_table(&self) -> ui::Table<Msg> {
		let mut table = self
			.table(Page::Services, "No services yet")
			.col(Column::new("name", "Service").grow(1.0).priority(6.0))
			.col(Column::new("region", "Region").priority(3.0))
			.col(Column::new("version", "Version").mono(true).priority(2.0))
			.col(Column::new("traffic", "Traffic").priority(1.0))
			.col(
				Column::new("rps", "req/s")
					.align(TextAlign::End)
					.mono(true)
					.priority(4.0),
			)
			.col(
				Column::new("p95", "p95")
					.align(TextAlign::End)
					.mono(true)
					.priority(4.0),
			)
			.col(
				Column::new("errors", "Errors")
					.align(TextAlign::End)
					.mono(true)
					.priority(4.0),
			)
			.col(Column::new("health", "Health").priority(5.0));
		for s in &self.services {
			let (label, tone) = s.health.look();
			let mut errors = TextCell::new(format!("{:.1}%", s.errors));
			if s.errors >= 1.0 {
				errors = errors.tone(Tone::Error);
			}
			table = table.row(
				TableRow::new(s.id)
					.cell("name", TextCell::new(s.id).dot(tone.clone()))
					.cell("region", s.region)
					.cell("version", s.version)
					.cell("traffic", SparkCell::new(s.rps.clone()))
					.cell("rps", format!("{:.0}", s.rps.last().copied().unwrap_or_default()))
					.cell("p95", format!("{:.0} ms", s.p95))
					.cell("errors", errors)
					.cell("health", BadgeSpec::new(label).tone(tone)),
			);
		}
		table
	}

	/// The deploys toolbar: a muted description and the filter.
	fn deploys_bar(&self) -> ui::Row<Msg> {
		let attention = self
			.deploys
			.iter()
			.filter(|d| matches!(d.state, State::Waiting | State::Failed))
			.count();
		ui::row()
			.key("bar")
			.justify("between")
			.align("center")
			.child(
				ui::text(ui::span("Rollouts to production, newest first").style("muted")).key("about"),
			)
			.child(
				ui::tabs()
					.key("filter")
					.variant("segmented")
					.active(self.filter.clone())
					.tab("all", "All")
					.tab("attention", spans![
						"Needs you ",
						ui::span(attention.to_string()).style("muted")
					])
					.on_select(|id| Msg::Filter(id.to_owned())),
			)
	}

	/// The deploys table.
	fn deploys_table(&self) -> ui::Table<Msg> {
		let mut table = self
			.table(Page::Deploys, "Nothing needs you right now")
			.col(Column::new("service", "Service").grow(1.0).priority(6.0))
			.col(Column::new("commit", "Commit").mono(true).priority(3.0))
			.col(Column::new("region", "Region").priority(2.0))
			.col(Column::new("traffic", "Traffic").priority(1.0))
			.col(Column::new("rollout", "Rollout").priority(1.0))
			.col(Column::new("state", "State").priority(5.0));
		for d in self.shown_deploys() {
			let (label, tone) = d.state.look();
			table = table.row(
				TableRow::new(d.id)
					.cell("service", TextCell::new(d.service).dot(tone.clone()))
					.cell("commit", d.commit)
					.cell("region", d.region)
					.cell("traffic", SparkCell::new(d.rps.clone()))
					.cell("rollout", MeterCell {
						value: Some(d.done),
						title: Some(format!("{:.0}%", d.done * 100.0)),
						..MeterCell::default()
					})
					.cell("state", BadgeSpec::new(label).tone(tone)),
			);
		}
		table
	}

	/// The incidents table.
	fn incidents_table(&self) -> ui::Table<Msg> {
		let mut table = self
			.table(Page::Incidents, "No incidents. All quiet.")
			.col(Column::new("title", "Incident").grow(1.0).priority(6.0))
			.col(Column::new("id", "Id").mono(true).priority(2.0))
			.col(Column::new("service", "Service").priority(3.0))
			.col(Column::new("severity", "Severity").priority(4.0))
			.col(
				Column::new("age", "Open for")
					.align(TextAlign::End)
					.mono(true)
					.priority(4.0),
			)
			.col(Column::new("phase", "State").priority(5.0));
		for i in &self.incidents {
			let (sev, sev_tone) = i.severity.look();
			let (phase, phase_tone) = i.phase.look();
			table = table.row(
				TableRow::new(i.id)
					.cell("title", TextCell::new(i.title).dot(sev_tone.clone()))
					.cell("id", i.id)
					.cell("service", i.service)
					.cell("severity", BadgeSpec::new(sev).tone(sev_tone))
					.cell("age", ago(i.age))
					.cell("phase", BadgeSpec::new(phase).tone(phase_tone)),
			);
		}
		table
	}

	/// The key legend.
	fn legend(&self) -> ui::Row<Msg> {
		let key = |k: &str| ui::span(k).style("key");
		let muted = |t: &str| ui::span(t).style("muted");
		let keys = if self.confirming.is_some() {
			spans![key("enter"), muted(" confirm  "), key("esc"), muted(" back")]
		} else {
			spans![
				key("tab"),
				muted(" views  "),
				key("j"),
				key("k"),
				muted(" move  "),
				key("enter"),
				muted(" open  "),
				key("a"),
				muted(" action  "),
				key("r"),
				muted(" refresh  "),
				key("q"),
				muted(" quit"),
			]
		};
		let note = if self.note.is_empty() {
			"Up to date"
		} else {
			self.note.as_str()
		};
		ui::row()
			.key("legend")
			.justify("between")
			.align("center")
			.child(ui::text(keys).key("keys"))
			.child(ui::text(muted(note)).key("note"))
	}
}

/// A detail card: the head with the primary action at its end, the `kv` and
/// the `danger` trigger.
fn detail(
	page: Page,
	id: &'static str,
	meta: impl Into<Text>,
	kv: ui::Kv<Msg>,
	action: Option<&'static str>,
	risk: &Risk,
) -> ui::Card<Msg> {
	let mut card = ui::card().head(id).meta(meta);
	if let Some(label) = action {
		card = card.end(
			ui::button(label)
				.key("act")
				.variant("primary")
				.keys(["a"])
				.on_click(Msg::Act(page, id.into())),
		);
	}
	let mut trigger = ui::button(risk.button)
		.key("risk")
		.variant("danger")
		.on_click(Msg::Ask(page, id.into()));
	if let Some(why) = risk.blocked {
		trigger = trigger.disabled(why);
	}
	card
		.child(kv.key("kv"))
		.child(ui::row().key("bar").child(trigger))
}

/// The selected service's detail card.
fn service_card(s: &Service) -> ui::Card<Msg> {
	let (label, _) = s.health.look();
	let rps = s.rps.last().copied().unwrap_or_default();
	let kv = ui::kv()
		.item("Health", ui::span(label).style("strong"))
		.item("Version", mono(s.version))
		.item("Replicas", mono(s.replicas.to_string()))
		.item("Traffic", mono(format!("{rps:.0} req/s")))
		.item("p95", mono(format!("{:.0} ms", s.p95)))
		.item("Errors", mono(format!("{:.1}%", s.errors)));
	detail(Page::Services, s.id, s.region, kv, Some(s.action()), &s.risk())
}

/// The selected deploy's detail card.
fn deploy_card(d: &Deploy) -> ui::Card<Msg> {
	let (label, _) = d.state.look();
	let rps = d.rps.last().copied().unwrap_or_default();
	let kv = ui::kv()
		.item("State", ui::span(label).style("strong"))
		.item("Service", d.service)
		.item("Commit", mono(d.commit))
		.item("Rollout", mono(format!("{:.0}%", d.done * 100.0)))
		.item("Traffic", mono(format!("{rps:.0} req/s")));
	detail(Page::Deploys, d.id, d.region, kv, d.action(), &d.risk())
}

/// The selected incident's detail card.
fn incident_card(i: &Incident) -> ui::Card<Msg> {
	let (phase, _) = i.phase.look();
	let (sev, _) = i.severity.look();
	let kv = ui::kv()
		.item("State", ui::span(phase).style("strong"))
		.item("What", i.title)
		.item("Service", i.service)
		.item("Severity", sev)
		.item("Open for", mono(ago(i.age)));
	detail(Page::Incidents, i.id, i.service, kv, i.action(), &i.risk())
}

/// The confirm strip before a destructive action: `error` with a `danger`
/// confirm; its buttons sit in the head's end.
fn confirm(risk: &Risk) -> ui::Card<Msg> {
	let buttons = ui::row()
		.gap("sm")
		.child(
			ui::button("Keep")
				.key("keep")
				.variant("quiet")
				.keys(["esc"])
				.on_click(Msg::Keep),
		)
		.child(
			ui::button(risk.button)
				.key("go")
				.variant("danger")
				.keys(["enter"])
				.on_click(Msg::Confirm),
		);
	ui::card()
		.key("confirm")
		.tone(Tone::Error)
		.head(risk.ask.clone())
		.end(buttons)
		.child(ui::text(risk.then).key("t"))
}

fn main() -> Result<(), tern_sdk::Error> {
	let mut dash = Dash::new();
	let Some(mut session) = Session::connect(Options::new().app("dashboard"))? else {
		print!("{}", dash.view(true).plain(100));
		return Ok(());
	};
	let sf = session.open(SurfaceOptions::screen().title("Fleet").font(Font::Sans))?;
	session.render(sf, dash.view(false))?;
	let mut next = Instant::now() + TICK;
	loop {
		let left = next.saturating_duration_since(Instant::now());
		match session.next(Some(left))? {
			None => {
				dash.tick();
				next = Instant::now() + TICK;
			},
			Some(Input::Msg(msg, _)) => dash.update(msg),
			Some(Input::Key(key)) => {
				if key.is("escape") && dash.confirming.is_some() {
					dash.update(Msg::Keep);
				} else if key.is("enter") && dash.confirming.is_some() {
					dash.update(Msg::Confirm);
				} else if key.is("escape") || key.is("q") || key.is("ctrl+c") {
					break;
				} else if key.is("tab") || key.is("]") {
					dash.page = dash.page.turn(false);
				} else if key.is("shift+tab") || key.is("[") {
					dash.page = dash.page.turn(true);
				} else if key.is("up") || key.is("k") {
					dash.pick(dash.page, -1);
				} else if key.is("down") || key.is("j") {
					dash.pick(dash.page, 1);
				} else if key.is("r") {
					dash.update(Msg::Refresh);
				} else if key.is("a") {
					dash.act_selected();
				} else if key.is("enter") {
					if let Some(id) = dash.selected[dash.page.index()] {
						dash.update(Msg::Open(dash.page, id.into()));
					}
				} else {
					continue;
				}
			},
			Some(Input::Event(_) | Input::Clipboard(_)) => continue,
		}
		session.render(sf, dash.view(false))?;
	}
	session.close()?;
	Ok(())
}
