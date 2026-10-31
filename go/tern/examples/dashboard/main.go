// Command dashboard is a screen app in the sans face, laid out as a page: a
// head row (title, meta, status badges, Refresh), a flat stats strip,
// underline tabs over three views (Services, Deploys, Incidents), each a
// full-width table beside a detail card for the selected row, and a dock
// holding the confirm strip and a key legend. Figures and timers update
// live.
//
// Keys: ↑/↓ or j/k move, Tab/⇧Tab (or ] and [) switch views, Enter runs the
// selected item's primary action (deploy, approve, acknowledge), x asks for
// its destructive one (restart, roll back; Enter confirms, Esc cancels), r
// refreshes, Esc or q quits. Outside Tern it prints the page once as plain
// text.
package main

import (
	"context"
	"errors"
	"fmt"
	"math"
	"math/rand/v2"
	"os"
	"slices"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/stencil-hq/tern-sdk/go/tern"
	"github.com/stencil-hq/tern-sdk/go/tern/ui"
)

// service is one row of the fleet.
type service struct {
	id, region, version string
	// state is "up", "degraded", "down" or "pending" (a deploy awaits approval).
	state   string
	cpu     float64
	latency []float64
	rps     int
}

// deploy is one rollout of a service.
type deploy struct {
	id, service, change, by string
	from, version           string
	// state is "awaiting" (approval), "live", "replaced" or "rolled back".
	state string
	at    time.Time
}

// incident is an open incident on a service.
type incident struct {
	id, service, title string
	down               bool // the service is down, not just degraded
	acked              bool
	opened             time.Time
}

// The views, in tab order.
const (
	viewServices  = "services"
	viewDeploys   = "deploys"
	viewIncidents = "incidents"
)

var views = []string{viewServices, viewDeploys, viewIncidents}

// dashboard is the program's state.
type dashboard struct {
	mu        sync.Mutex
	services  []service
	deploys   []deploy
	incidents []incident
	view      string
	selected  map[string]string // the selected row id of each view
	confirm   bool              // the confirm strip is up
	started   time.Time
	refresh   time.Time
	notice    string
	// plain stacks the table and its detail card for the plain-text
	// fallback, which prints a row's children on one line.
	plain bool
}

// newDashboard is the starting fleet at now.
func newDashboard(now time.Time) *dashboard {
	ago := func(d time.Duration) time.Time { return now.Add(-d) }
	return &dashboard{
		services: []service{
			{id: "api", region: "iad1", version: "v2.14.0", state: "up", cpu: .42,
				latency: []float64{38, 41, 40, 44, 39, 42, 47, 43}, rps: 1840},
			{id: "web", region: "iad1", version: "v5.2.1", state: "up", cpu: .28,
				latency: []float64{22, 21, 25, 24, 23, 22, 26, 24}, rps: 3120},
			{id: "auth", region: "fra1", version: "v1.9.3", state: "degraded", cpu: .81,
				latency: []float64{60, 72, 95, 110, 98, 120, 131, 126}, rps: 610},
			{id: "billing", region: "fra1", version: "v2.9.1", state: "pending", cpu: .12,
				latency: []float64{30, 31, 29, 30, 32, 31, 30, 29}, rps: 95},
			{id: "search", region: "sfo1", version: "v0.31.4", state: "down", cpu: 0,
				latency: []float64{80, 85, 90, 0, 0, 0, 0, 0}, rps: 0},
			{id: "media", region: "sfo1", version: "v4.4.2", state: "up", cpu: .55,
				latency: []float64{51, 49, 55, 53, 50, 48, 52, 54}, rps: 720},
		},
		deploys: []deploy{
			{id: "d-1048", service: "billing", change: "Ledger export to Parquet", by: "ci",
				from: "v2.9.1", version: "v3.0.0-rc.2", state: "awaiting", at: ago(6 * time.Minute)},
			{id: "d-1047", service: "api", change: "Rate limits per API key", by: "mara",
				from: "v2.13.2", version: "v2.14.0", state: "live", at: ago(42 * time.Minute)},
			{id: "d-1046", service: "auth", change: "Session store on Redis 7", by: "jun",
				from: "v1.9.2", version: "v1.9.3", state: "live", at: ago(110 * time.Minute)},
			{id: "d-1045", service: "search", change: "Tokenizer for CJK queries", by: "ci",
				from: "v0.31.3", version: "v0.31.4", state: "live", at: ago(3 * time.Hour)},
			{id: "d-1044", service: "web", change: "Checkout copy fixes", by: "lena",
				from: "v5.2.0", version: "v5.2.1", state: "live", at: ago(26 * time.Hour)},
			{id: "d-1043", service: "media", change: "AVIF thumbnails", by: "ci",
				from: "v4.4.1", version: "v4.4.2", state: "live", at: ago(50 * time.Hour)},
		},
		incidents: []incident{
			{id: "inc-312", service: "search", title: "No healthy instances", down: true, opened: ago(11 * time.Minute)},
			{id: "inc-311", service: "auth", title: "p95 latency over 120 ms", opened: ago(38 * time.Minute)},
		},
		view:     viewServices,
		selected: map[string]string{viewServices: "auth", viewDeploys: "d-1048", viewIncidents: "inc-312"},
		started:  ago(39 * time.Minute),
		refresh:  now,
	}
}

// look is a state's tone and label.
type look struct {
	tone  ui.Tone
	label string
}

var serviceLook = map[string]look{
	"up":       {ui.ToneSuccess, "Healthy"},
	"degraded": {ui.ToneWarning, "Degraded"},
	"down":     {ui.ToneError, "Down"},
	"pending":  {ui.TonePending, "Awaiting deploy"},
}

var deployLook = map[string]look{
	"awaiting":    {ui.TonePending, "Awaiting approval"},
	"live":        {ui.ToneSuccess, "Live"},
	"replaced":    {ui.ToneMuted, "Replaced"},
	"rolled back": {ui.ToneWarning, "Rolled back"},
}

func (i incident) severity() look {
	if i.down {
		return look{ui.ToneError, "Down"}
	}
	return look{ui.ToneWarning, "Degraded"}
}

func (i incident) status() look {
	if i.acked {
		return look{ui.ToneAccent, "Investigating"}
	}
	return look{ui.ToneMuted, "Open"}
}

// service is the service with id, or nil.
func (d *dashboard) service(id string) *service {
	i := slices.IndexFunc(d.services, func(s service) bool { return s.id == id })
	if i < 0 {
		return nil
	}
	return &d.services[i]
}

// rowIDs is the row ids of the active view, in order.
func (d *dashboard) rowIDs() []string {
	var ids []string
	switch d.view {
	case viewServices:
		for _, s := range d.services {
			ids = append(ids, s.id)
		}
	case viewDeploys:
		for _, p := range d.deploys {
			ids = append(ids, p.id)
		}
	case viewIncidents:
		for _, i := range d.incidents {
			ids = append(ids, i.id)
		}
	}
	return ids
}

// keepSelection selects the active view's first row when its selected row
// is gone.
func (d *dashboard) keepSelection() {
	ids := d.rowIDs()
	if slices.Contains(ids, d.selected[d.view]) {
		return
	}
	d.selected[d.view] = ""
	if len(ids) > 0 {
		d.selected[d.view] = ids[0]
	}
}

// move selects the row delta rows away from the selected one.
func (d *dashboard) move(delta int) {
	ids := d.rowIDs()
	if len(ids) == 0 {
		return
	}
	i := slices.Index(ids, d.selected[d.view])
	d.selected[d.view] = ids[min(max(i+delta, 0), len(ids)-1)]
	d.confirm = false
}

// selectView shows view.
func (d *dashboard) selectView(view string) {
	if slices.Contains(views, view) {
		d.view, d.confirm = view, false
		d.keepSelection()
	}
}

// cycleView shows the view delta tabs away, wrapping.
func (d *dashboard) cycleView(delta int) {
	i := slices.Index(views, d.view)
	d.selectView(views[(i+delta+len(views))%len(views)])
}

// approve rolls out the awaiting deploy id.
func (d *dashboard) approve(id string) {
	i := slices.IndexFunc(d.deploys, func(p deploy) bool { return p.id == id })
	if i < 0 || d.deploys[i].state != "awaiting" {
		return
	}
	p := &d.deploys[i]
	for j := range d.deploys {
		if q := &d.deploys[j]; q.service == p.service && q.state == "live" {
			q.state = "replaced"
		}
	}
	p.state, p.at = "live", time.Now()
	if s := d.service(p.service); s != nil {
		s.version, s.state = p.version, "up"
	}
	d.notice = "Deployed " + p.service + " " + p.version
}

// awaiting is the awaiting deploy of service, or nil.
func (d *dashboard) awaiting(service string) *deploy {
	i := slices.IndexFunc(d.deploys, func(p deploy) bool { return p.service == service && p.state == "awaiting" })
	if i < 0 {
		return nil
	}
	return &d.deploys[i]
}

// restart brings service id back up and closes its incidents.
func (d *dashboard) restart(id string) {
	s := d.service(id)
	if s == nil {
		return
	}
	if s.state != "pending" {
		s.state = "up"
	}
	s.cpu = .2
	if s.latency[len(s.latency)-1] == 0 || s.latency[len(s.latency)-1] > 60 {
		s.latency = append(s.latency[1:], 40)
	}
	d.incidents = slices.DeleteFunc(d.incidents, func(i incident) bool { return i.service == id })
	d.keepSelection()
	d.notice = "Restarted " + id
}

// rollback returns the live deploy id's service to the version before it.
func (d *dashboard) rollback(id string) {
	i := slices.IndexFunc(d.deploys, func(p deploy) bool { return p.id == id })
	if i < 0 || d.deploys[i].state != "live" {
		return
	}
	p := &d.deploys[i]
	p.state = "rolled back"
	if s := d.service(p.service); s != nil {
		s.version = p.from
		if s.state != "pending" {
			s.state = "up"
		}
	}
	d.incidents = slices.DeleteFunc(d.incidents, func(i incident) bool { return i.service == p.service })
	d.notice = "Rolled back " + p.service + " to " + p.from
}

// action is a named action on the selected item.
type action struct {
	verb string
	run  func()
}

// primary is the selected item's primary action, or nil when it has none.
func (d *dashboard) primary() *action {
	id := d.selected[d.view]
	switch d.view {
	case viewServices:
		if s := d.service(id); s != nil {
			if p := d.awaiting(s.id); p != nil {
				pid := p.id
				return &action{"Deploy " + p.version, func() { d.approve(pid) }}
			}
		}
	case viewDeploys:
		if i := slices.IndexFunc(d.deploys, func(p deploy) bool { return p.id == id }); i >= 0 && d.deploys[i].state == "awaiting" {
			return &action{"Approve", func() { d.approve(id) }}
		}
	case viewIncidents:
		if i := slices.IndexFunc(d.incidents, func(i incident) bool { return i.id == id }); i >= 0 && !d.incidents[i].acked {
			return &action{"Acknowledge", func() { d.acknowledge(id) }}
		}
	}
	return nil
}

// acknowledge marks incident id as being investigated.
func (d *dashboard) acknowledge(id string) {
	if i := slices.IndexFunc(d.incidents, func(i incident) bool { return i.id == id }); i >= 0 {
		d.incidents[i].acked = true
		d.notice = "Acknowledged " + id
	}
}

// risk is a destructive action on the selected item; it runs only from
// the confirm strip.
type risk struct {
	action
	head, meta string
	why        string // set when unavailable: the reason
}

// risky is the selected item's destructive action, or nil with nothing
// selected.
func (d *dashboard) risky() *risk {
	id := d.selected[d.view]
	restart := func(svc string) *risk {
		return &risk{action{"Restart", func() { d.restart(svc) }}, "Restart " + svc + "?", "Open connections drop.", ""}
	}
	switch d.view {
	case viewServices:
		if d.service(id) != nil {
			return restart(id)
		}
	case viewDeploys:
		i := slices.IndexFunc(d.deploys, func(p deploy) bool { return p.id == id })
		if i < 0 {
			return nil
		}
		p := d.deploys[i]
		r := &risk{action{"Roll back", func() { d.rollback(id) }},
			"Roll back " + p.service + " to " + p.from + "?", p.version + " stops serving traffic.", ""}
		if p.state != "live" {
			r.why = "Only a live deploy rolls back"
		}
		return r
	case viewIncidents:
		if i := slices.IndexFunc(d.incidents, func(i incident) bool { return i.id == id }); i >= 0 {
			return restart(d.incidents[i].service)
		}
	}
	return nil
}

// tick moves the live figures one step; jitter is a value in [-1, 1).
func (d *dashboard) tick(jitter func() float64) {
	for i := range d.services {
		s := &d.services[i]
		if s.state == "down" {
			s.latency = append(s.latency[1:], 0)
			continue
		}
		last := s.latency[len(s.latency)-1]
		s.latency = append(s.latency[1:], math.Max(5, math.Round(last+jitter()*6)))
		s.cpu = math.Min(1, math.Max(.02, s.cpu+jitter()*.05))
		s.rps = max(0, s.rps+int(jitter()*40))
	}
	d.refresh = time.Now()
}

func randomJitter() float64 { return rand.Float64()*2 - 1 }

// key applies a key; false quits.
func (d *dashboard) key(k tern.Key) bool {
	switch {
	case k.Is("escape"):
		// Escape backs out of the confirm strip, else quits.
		if d.confirm {
			d.confirm = false
			return true
		}
		return false
	case k.Is("q"), k.Is("ctrl+c"):
		return false
	case k.Is("up"), k.Is("k"):
		d.move(-1)
	case k.Is("down"), k.Is("j"):
		d.move(1)
	case k.Is("tab"), k.Is("]"):
		d.cycleView(1)
	case k.Is("shift+tab"), k.Is("["):
		d.cycleView(-1)
	case k.Is("enter"):
		if d.confirm {
			if r := d.risky(); r != nil && r.why == "" {
				r.run()
			}
			d.confirm = false
		} else if a := d.primary(); a != nil {
			a.run()
		}
	case k.Is("x"):
		r := d.risky()
		d.confirm = r != nil && r.why == ""
	case k.Is("r"):
		d.tick(randomJitter)
	}
	return true
}

// page is the whole dashboard; rerender runs after a handler changes state.
func (d *dashboard) page(rerender func()) tern.View {
	// on wraps a state change for a handler: lock, change, unlock, render.
	on := func(f func()) func() {
		return func() {
			d.mu.Lock()
			f()
			d.mu.Unlock()
			rerender()
		}
	}

	var body tern.Element
	switch d.view {
	case viewServices:
		body = d.servicesView(on)
	case viewDeploys:
		body = d.deploysView(on)
	default:
		body = d.incidentsView(on)
	}
	main := tern.Nodes{d.head(on), d.stats(), d.tabs(on), body}
	if d.notice != "" {
		main = append(main, ui.Toast{Common: ui.Common{Key: "notice"}, Text: d.notice, TTL: new(3000.0)})
	}
	return tern.View{Main: main, Dock: d.dock(on)}
}

// head is the head row: title and meta on the left, status badges and
// Refresh on the right.
func (d *dashboard) head(on func(func()) func()) tern.Element {
	meta := fmt.Sprintf("%d services · up %s · updated %s",
		len(d.services), short(time.Since(d.started)), d.refresh.Format("15:04:05"))
	health := ui.Badge{Text: "All healthy", Common: ui.Common{Tone: ui.ToneSuccess}}
	if n := len(d.incidents); n > 0 {
		health = ui.Badge{Text: plural(n, "incident"), Common: ui.Common{Tone: ui.ToneWarning}}
		if slices.ContainsFunc(d.incidents, func(i incident) bool { return i.down }) {
			health.Tone = ui.ToneError
		}
	}
	return ui.Row{
		Common: ui.Common{Key: "head"}, Justify: ui.JustifyBetween, Align: ui.AlignCenter,
		Children: ui.Nodes(
			ui.Text{Spans: []ui.Span{{T: "Fleet", S: "title"}, {T: "  " + meta, S: "muted"}}},
			ui.Row{Gap: ui.GapSM, Align: ui.AlignCenter, Children: ui.Nodes(
				ui.Badge{Text: "Live", Common: ui.Common{Tone: ui.ToneInfo}},
				health,
				ui.Button{
					Common: ui.Common{Key: "refresh", OnClick: func(*tern.Action) { on(func() { d.tick(randomJitter) })() }},
					Text:   "Refresh", Icon: ui.IconRepeat, Keys: []string{"r"}, Size: ui.SizeSM,
				},
			)},
		),
	}
}

// stats is the flat strip of headline figures, directly on the page.
func (d *dashboard) stats() tern.Element {
	up, rps := 0, 0
	regions := map[string]bool{}
	for _, s := range d.services {
		if s.state == "up" {
			up++
		}
		rps += s.rps
		regions[s.region] = true
	}
	incidents := ui.Stat{Label: ui.T("Open incidents"), Value: ui.T(strconv.Itoa(len(d.incidents))),
		Tone: ui.ToneSuccess, Sub: ui.T("all clear")}
	if len(d.incidents) > 0 {
		var on []string
		incidents.Tone = ui.ToneWarning
		for _, i := range d.incidents {
			on = append(on, i.service)
			if i.down {
				incidents.Tone = ui.ToneError
			}
		}
		incidents.Sub = ui.T(strings.Join(on, ", "))
	}
	queued := ui.Stat{Label: ui.T("Awaiting deploy"), Value: ui.T("0"), Sub: ui.T("nothing queued")}
	var waiting []string
	for _, p := range d.deploys {
		if p.state == "awaiting" {
			waiting = append(waiting, p.service)
		}
	}
	if len(waiting) > 0 {
		queued.Value, queued.Tone, queued.Sub = ui.T(strconv.Itoa(len(waiting))), ui.TonePending, ui.T(strings.Join(waiting, ", "))
	}
	return ui.Stats{Common: ui.Common{Key: "stats"}, Items: []ui.Stat{
		{Label: ui.T("Healthy"), Value: ui.T(fmt.Sprintf("%d/%d", up, len(d.services))), Sub: ui.T("services up")},
		incidents,
		queued,
		{Label: ui.T("Traffic"), Value: ui.T(thousands(rps) + " rps"), Sub: ui.T(fmt.Sprintf("across %d regions", len(regions)))},
	}}
}

// tabs is the navigation: one underline tab per view, with a muted count.
func (d *dashboard) tabs(on func(func()) func()) tern.Element {
	tab := func(id, name string, n int) ui.Tab {
		return ui.Tab{ID: id, Label: ui.Spans(ui.Span{T: name}, ui.Span{T: " " + strconv.Itoa(n), S: "muted"})}
	}
	return ui.Tabs{
		Common: ui.Common{Key: "views", OnSelect: func(e *tern.Select) { on(func() { d.selectView(e.Item) })() }},
		Active: d.view,
		Items: []ui.Tab{
			tab(viewServices, "Services", len(d.services)),
			tab(viewDeploys, "Deploys", len(d.deploys)),
			tab(viewIncidents, "Incidents", len(d.incidents)),
		},
	}
}

// split is a view: the table beside the selected row's detail card (or the
// table alone with nothing selected). The plain fallback stacks them.
func (d *dashboard) split(table ui.Table, detail tern.Element) tern.Element {
	if detail == nil {
		return table
	}
	if d.plain {
		return ui.Col{Common: ui.Common{Key: "view"}, Gap: ui.GapLG, Children: ui.Nodes(table, detail)}
	}
	return ui.Row{Common: ui.Common{Key: "view"}, Align: ui.AlignStretch, Gap: ui.GapLG, Children: ui.Nodes(
		ui.Col{Common: ui.Common{Grow: 1, Basis: ui.BasisFrac(0)}, Children: ui.Nodes(table)},
		detail,
	)}
}

// tableEvents is the handlers every view's table shares: a click selects,
// a double click runs the primary action.
func (d *dashboard) tableEvents(key string, on func(func()) func()) ui.Common {
	return ui.Common{
		Key: key,
		OnSelect: func(e *tern.Select) {
			on(func() { d.selected[d.view], d.confirm = e.Item, false })()
		},
		OnActivate: func(e *tern.Activate) {
			on(func() {
				d.selected[d.view], d.confirm = e.Item, false
				if a := d.primary(); a != nil {
					a.run()
				}
			})()
		},
	}
}

// card is the detail card of the selected item: the primary action (or a
// status badge) in its head's end, the kv, and the destructive trigger.
func (d *dashboard) card(head, meta string, status look, pairs []ui.Pair, on func(func()) func()) tern.Element {
	var end tern.Element = ui.Badge{Text: status.label, Common: ui.Common{Tone: status.tone}}
	if a := d.primary(); a != nil {
		end = ui.Button{
			Common: ui.Common{Key: "primary", OnClick: func(*tern.Action) { on(a.run)() }},
			Text:   a.verb, Keys: []string{"enter"}, Variant: ui.ButtonPrimary, Size: ui.SizeSM,
		}
	}
	children := ui.Nodes(ui.KV{Common: ui.Common{Key: "kv"}, Items: pairs})
	if r := d.risky(); r != nil {
		// A danger trigger only opens the confirm strip.
		trigger := ui.Button{
			Common: ui.Common{Key: "risk", OnClick: func(*tern.Action) { on(func() { d.confirm = true })() }},
			Text:   r.verb, Keys: []string{"x"}, Variant: ui.ButtonDanger, Size: ui.SizeSM,
		}
		switch {
		case r.why != "":
			trigger.Disabled = ui.Because(r.why)
		case d.confirm:
			trigger.Disabled = ui.Off
		}
		children = append(children, ui.Row{Common: ui.Common{Key: "actions"}, Children: ui.Nodes(trigger)})
	}
	return ui.Card{
		Common:   ui.Common{Key: "detail", Min: &ui.Bounds{W: ui.Ch(44)}, Max: &ui.Bounds{W: ui.Ch(56)}},
		Head:     ui.T(head),
		Meta:     ui.T(meta),
		End:      end,
		Children: children,
	}
}

func (d *dashboard) servicesView(on func(func()) func()) tern.Element {
	common := d.tableEvents("services", on)
	common.OnAction = map[string]func(*tern.Action){
		"restart": func(e *tern.Action) { on(func() { d.selected[d.view], d.confirm = e.Item, true })() },
	}
	table := ui.Table{
		Common:     common,
		Selectable: true,
		Selected:   d.selected[viewServices],
		Empty:      ui.T("No services."),
		Cols: []ui.Column{
			// A missing priority is 0 and hides first: the name never
			// hides, numbers next, sparks and meters go first.
			{ID: "name", Head: ui.T("Service"), Grow: 1, Priority: 5},
			{ID: "state", Head: ui.T("Status"), Priority: 3},
			{ID: "region", Head: ui.T("Region"), Priority: 2},
			{ID: "version", Head: ui.T("Version"), Mono: true, Priority: 2},
			{ID: "cpu", Head: ui.T("CPU"), Priority: 1},
			{ID: "latency", Head: ui.T("Latency"), Priority: 1},
			{ID: "p95", Head: ui.T("p95"), Align: ui.AlignEnd, Mono: true, Priority: 4},
			{ID: "rps", Head: ui.T("Req/s"), Align: ui.AlignEnd, Mono: true, Priority: 4},
		},
	}
	for _, s := range d.services {
		l := serviceLook[s.state]
		cpu := s.cpu
		table.Rows = append(table.Rows, ui.TableRow{ID: s.id, Menu: []string{"restart"}, Cells: map[string]ui.Cell{
			"name":    ui.TextCell{Text: ui.T(s.id), Dot: l.tone, Title: l.label},
			"state":   ui.BadgeCell{Text: l.label, Tone: l.tone},
			"region":  ui.T(s.region),
			"version": ui.T(s.version),
			"cpu":     ui.MeterCell{Value: &cpu, Thresholds: &ui.Thresholds{Warn: new(.7), Bad: new(.9)}},
			"latency": ui.SparkCell{Values: s.latency, Tone: l.tone},
			"p95":     ui.T(ms(s)),
			"rps":     ui.T(thousands(s.rps)),
		}})
	}
	s := d.service(d.selected[viewServices])
	if s == nil {
		return d.split(table, nil)
	}
	l := serviceLook[s.state]
	open := 0
	for _, i := range d.incidents {
		if i.service == s.id {
			open++
		}
	}
	return d.split(table, d.card(s.id, s.region+" · "+s.version, l, []ui.Pair{
		{K: ui.T("Status"), V: ui.Spans(ui.Span{T: l.label, S: string(l.tone)})},
		{K: ui.T("Version"), V: ui.Spans(ui.Span{T: s.version, S: "mono"})},
		{K: ui.T("CPU"), V: ui.Spans(ui.Span{T: fmt.Sprintf("%.0f%%", s.cpu*100), S: "mono"})},
		{K: ui.T("Latency"), V: ui.Spans(ui.Span{T: ms(*s), S: "mono"}, ui.Span{T: " p95", S: "muted"})},
		{K: ui.T("Traffic"), V: ui.Spans(ui.Span{T: thousands(s.rps) + " rps", S: "mono"})},
		{K: ui.T("Incidents"), V: ui.Spans(ui.Span{T: strconv.Itoa(open), S: "mono"})},
	}, on))
}

func (d *dashboard) deploysView(on func(func()) func()) tern.Element {
	table := ui.Table{
		Common:     d.tableEvents("deploys", on),
		Selectable: true,
		Selected:   d.selected[viewDeploys],
		Empty:      ui.T("No deploys yet."),
		Cols: []ui.Column{
			{ID: "change", Head: ui.T("Change"), Grow: 1, Priority: 5},
			{ID: "service", Head: ui.T("Service"), Priority: 3},
			{ID: "version", Head: ui.T("Version"), Mono: true, Priority: 2},
			{ID: "state", Head: ui.T("Status"), Priority: 3},
			{ID: "by", Head: ui.T("By"), Priority: 1},
			{ID: "age", Head: ui.T("Age"), Align: ui.AlignEnd, Mono: true, Priority: 4},
		},
	}
	var sel *deploy
	for i, p := range d.deploys {
		l := deployLook[p.state]
		table.Rows = append(table.Rows, ui.TableRow{ID: p.id, Cells: map[string]ui.Cell{
			"change":  ui.TextCell{Text: ui.T(p.change), Dot: l.tone, Title: l.label},
			"service": ui.T(p.service),
			"version": ui.T(p.version),
			"state":   ui.BadgeCell{Text: l.label, Tone: l.tone},
			"by":      ui.Spans(ui.Span{T: p.by, S: "muted"}),
			"age":     ui.T(short(time.Since(p.at))),
		}})
		if p.id == d.selected[viewDeploys] {
			sel = &d.deploys[i]
		}
	}
	if sel == nil {
		return d.split(table, nil)
	}
	l := deployLook[sel.state]
	return d.split(table, d.card(sel.change, sel.service+" · "+sel.id, l, []ui.Pair{
		{K: ui.T("Status"), V: ui.Spans(ui.Span{T: l.label, S: string(l.tone)})},
		{K: ui.T("Version"), V: ui.Spans(ui.Span{T: sel.from, S: "mono muted"}, ui.Span{T: " → "}, ui.Span{T: sel.version, S: "mono"})},
		{K: ui.T("By"), V: ui.T(sel.by)},
		{K: ui.T("Started"), V: ui.Spans(ui.Span{T: short(time.Since(sel.at)), S: "mono"}, ui.Span{T: " ago", S: "muted"})},
	}, on))
}

func (d *dashboard) incidentsView(on func(func()) func()) tern.Element {
	table := ui.Table{
		Common:     d.tableEvents("incidents", on),
		Selectable: true,
		Selected:   d.selected[viewIncidents],
		Empty:      ui.T("No open incidents. Every service is healthy."),
		Cols: []ui.Column{
			{ID: "title", Head: ui.T("Incident"), Grow: 1, Priority: 5},
			{ID: "service", Head: ui.T("Service"), Priority: 3},
			{ID: "severity", Head: ui.T("Severity"), Priority: 3},
			{ID: "status", Head: ui.T("Status"), Priority: 2},
			{ID: "open", Head: ui.T("Open for"), Align: ui.AlignEnd, Mono: true, Priority: 4},
		},
	}
	var sel *incident
	for i, inc := range d.incidents {
		sev, st := inc.severity(), inc.status()
		table.Rows = append(table.Rows, ui.TableRow{ID: inc.id, Cells: map[string]ui.Cell{
			"title":    ui.TextCell{Text: ui.T(inc.title), Dot: sev.tone, Title: sev.label},
			"service":  ui.T(inc.service),
			"severity": ui.BadgeCell{Text: sev.label, Tone: sev.tone},
			"status":   ui.Spans(ui.Span{T: st.label, S: string(st.tone)}),
			"open":     ui.T(clock(time.Since(inc.opened))),
		}})
		if inc.id == d.selected[viewIncidents] {
			sel = &d.incidents[i]
		}
	}
	if sel == nil {
		return d.split(table, nil)
	}
	sev, st := sel.severity(), sel.status()
	svc := "unknown"
	if s := d.service(sel.service); s != nil {
		svc = serviceLook[s.state].label
	}
	return d.split(table, d.card(sel.title, sel.service+" · "+sel.id, st, []ui.Pair{
		{K: ui.T("Severity"), V: ui.Spans(ui.Span{T: sev.label, S: string(sev.tone)})},
		{K: ui.T("Status"), V: ui.Spans(ui.Span{T: st.label, S: string(st.tone)})},
		{K: ui.T("Service"), V: ui.Spans(ui.Span{T: sel.service}, ui.Span{T: " · " + svc, S: "muted"})},
		{K: ui.T("Open for"), V: ui.Spans(ui.Span{T: clock(time.Since(sel.opened)), S: "mono"})},
	}, on))
}

// dock is the confirm strip, when up, over the key legend.
func (d *dashboard) dock(on func(func()) func()) tern.Element {
	var legend [][2]string
	r := d.risky()
	if d.confirm && r != nil {
		legend = [][2]string{{"enter", strings.ToLower(r.verb)}, {"esc", "cancel"}}
	} else {
		legend = [][2]string{{"↑↓", "select"}}
		if a := d.primary(); a != nil {
			legend = append(legend, [2]string{"enter", strings.ToLower(strings.Fields(a.verb)[0])})
		}
		if r != nil && r.why == "" {
			legend = append(legend, [2]string{"x", strings.ToLower(r.verb)})
		}
		legend = append(legend, [2]string{"tab", "views"}, [2]string{"r", "refresh"}, [2]string{"q", "quit"})
	}
	var spans []ui.Span
	for i, kv := range legend {
		if i > 0 {
			spans = append(spans, ui.Span{T: "   "})
		}
		spans = append(spans, ui.Span{T: kv[0], S: "key"}, ui.Span{T: " " + kv[1], S: "muted"})
	}
	children := ui.Nodes(ui.Text{Common: ui.Common{Key: "legend"}, Spans: spans})
	if d.confirm && r != nil {
		// Destructive: an error-toned strip whose confirm is danger.
		strip := ui.Card{
			Common: ui.Common{Key: "confirm", Tone: ui.ToneError},
			Head:   ui.T(r.head),
			Meta:   ui.T(r.meta),
			End: ui.Row{Gap: ui.GapSM, Children: ui.Nodes(
				ui.Button{Common: ui.Common{Key: "cancel", OnClick: func(*tern.Action) { on(func() { d.confirm = false })() }},
					Text: "Cancel", Keys: []string{"esc"}, Variant: ui.ButtonQuiet},
				ui.Button{Common: ui.Common{Key: "yes", OnClick: func(*tern.Action) {
					on(func() {
						r.run()
						d.confirm = false
					})()
				}}, Text: r.verb, Keys: []string{"enter"}, Variant: ui.ButtonDanger},
			)},
		}
		children = append(ui.Nodes(strip), children...)
	}
	return ui.Col{Common: ui.Common{Key: "dock"}, Gap: ui.GapSM, Children: children}
}

// ms is a service's latest latency, "–" while it is down.
func ms(s service) string {
	if s.state == "down" {
		return "–"
	}
	return fmt.Sprintf("%.0f ms", s.latency[len(s.latency)-1])
}

// short is a coarse duration: 45s, 12m, 3h 5m, 2d.
func short(d time.Duration) string {
	switch {
	case d < time.Minute:
		return fmt.Sprintf("%ds", int(d.Seconds()))
	case d < time.Hour:
		return fmt.Sprintf("%dm", int(d.Minutes()))
	case d < 24*time.Hour:
		return fmt.Sprintf("%dh %dm", int(d.Hours()), int(d.Minutes())%60)
	}
	return fmt.Sprintf("%dd", int(d.Hours()/24))
}

// clock is a running timer: 4:05 or 1:02:09.
func clock(d time.Duration) string {
	s := int(d.Seconds())
	if s >= 3600 {
		return fmt.Sprintf("%d:%02d:%02d", s/3600, s/60%60, s%60)
	}
	return fmt.Sprintf("%d:%02d", s/60, s%60)
}

// thousands is n with comma separators.
func thousands(n int) string {
	s := strconv.Itoa(n)
	for i := len(s) - 3; i > 0; i -= 3 {
		s = s[:i] + "," + s[i:]
	}
	return s
}

func plural(n int, word string) string {
	if n == 1 {
		return "1 " + word
	}
	return strconv.Itoa(n) + " " + word + "s"
}

func main() {
	d := newDashboard(time.Now())
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	s, err := tern.Connect(ctx, tern.Options{})
	if errors.Is(err, tern.ErrUnsupported) {
		d.plain = true
		fmt.Print(tern.Plain(d.page(func() {}), 0))
		return
	}
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	defer s.Close()
	sf, err := s.Open(tern.SurfaceOptions{Mode: tern.Screen, Title: "dashboard", Font: tern.FontSans})
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		return
	}
	var render func()
	render = func() {
		d.mu.Lock()
		v := d.page(render)
		d.notice = ""
		d.mu.Unlock()
		_ = sf.Render(v)
	}
	render()
	go func() {
		t := time.NewTicker(2 * time.Second)
		defer t.Stop()
		for {
			select {
			case <-ctx.Done():
				return
			case <-t.C:
				d.mu.Lock()
				d.tick(randomJitter)
				d.mu.Unlock()
				render()
			}
		}
	}()
	_ = s.Run(ctx, func(in tern.Input) error {
		if k, ok := in.(tern.Key); ok {
			d.mu.Lock()
			more := d.key(k)
			d.mu.Unlock()
			if !more {
				return tern.ErrStop
			}
			render()
		}
		return nil
	})
}
