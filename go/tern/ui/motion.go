package ui

import "github.com/stencil-hq/tern-sdk/go/tern"

// Spinner is an activity indicator (kind spinner).
type Spinner struct {
	Common
	// Style is the indicator (default braille).
	Style SpinnerStyle `json:"style,omitzero"`
	// Label is text after the indicator.
	Label Rich `json:"label,omitzero"`
}

// SpinnerStyle is a spinner's indicator.
type SpinnerStyle string

// Spinner styles.
const (
	SpinnerBraille   SpinnerStyle = "braille"
	SpinnerDots      SpinnerStyle = "dots"
	SpinnerStarburst SpinnerStyle = "starburst"
	SpinnerOrbit     SpinnerStyle = "orbit"
)

// Node builds the spinner.
func (s Spinner) Node() tern.Node { return Build(tern.KindSpinner, s, nil) }

// Shimmer is a shimmering label (kind shimmer).
type Shimmer struct {
	Common
	// Spans are styled text; they win over Text.
	Spans []Span `json:"spans,omitzero"`
	// Text is plain text.
	Text string `json:"text,omitzero"`
	// Mode is classic (default) or kitt.
	Mode ShimmerMode `json:"mode,omitzero"`
	// Palette is the three sweep colors as span tokens.
	Palette *ShimmerPalette `json:"palette,omitzero"`
}

// ShimmerMode is how a shimmer sweeps.
type ShimmerMode string

// Shimmer modes.
const (
	ShimmerClassic ShimmerMode = "classic"
	ShimmerKitt    ShimmerMode = "kitt"
)

// ShimmerPalette is a shimmer's sweep colors.
type ShimmerPalette struct {
	Low  string `json:"low,omitzero"`
	Mid  string `json:"mid,omitzero"`
	High string `json:"high,omitzero"`
}

// Node builds the shimmer.
func (s Shimmer) Node() tern.Node { return Build(tern.KindShimmer, s, nil) }

// Elapsed is a live timer (kind elapsed).
type Elapsed struct {
	Common
	// Age is the milliseconds already elapsed; negative counts down.
	Age float64 `json:"age,omitzero"`
	// Stopped freezes the display at this many ms.
	Stopped *float64 `json:"stopped,omitzero"`
	// Format is short (default) or clock.
	Format ElapsedFormat `json:"format,omitzero"`
}

// ElapsedFormat is how an elapsed time reads.
type ElapsedFormat string

// Elapsed formats.
const (
	ElapsedShort ElapsedFormat = "short"
	ElapsedClock ElapsedFormat = "clock"
)

// Node builds the elapsed.
func (e Elapsed) Node() tern.Node { return Build(tern.KindElapsed, e, nil) }

// Rate is a number easing between updates (kind rate).
type Rate struct {
	Common
	// Value is the target value.
	Value float64 `json:"value,omitzero"`
	// Unit is appended after a space.
	Unit string `json:"unit,omitzero"`
}

// Node builds the rate.
func (r Rate) Node() tern.Node { return Build(tern.KindRate, r, nil) }

// Progress is a progress bar (kind progress).
type Progress struct {
	Common
	// Value is the fraction done, 0–1; nil is indeterminate.
	Value *float64 `json:"value,omitzero"`
	// Label is text after the bar.
	Label Rich `json:"label,omitzero"`
}

// Node builds the progress.
func (p Progress) Node() tern.Node { return Build(tern.KindProgress, p, nil) }

// Meter is a value as a bar, ring or grid (kind meter).
type Meter struct {
	Common
	// Value is the level, 0–1.
	Value *float64 `json:"value,omitzero"`
	// Parts are stacked parts.
	Parts []Part `json:"parts,omitzero"`
	// Thresholds set data-level warn or bad.
	Thresholds *Thresholds `json:"thresholds,omitzero"`
	// Style is bar (default), ring or blocks.
	Style MeterStyle `json:"style,omitzero"`
	// Size is sm (default), md or lg.
	Size Size `json:"size,omitzero"`
	// Steps is the blocks style's cell count, 1–20.
	Steps int `json:"steps,omitzero"`
	// Marks are ticks on the track.
	Marks []MeterMark `json:"marks,omitzero"`
	// Label is text after the shape.
	Label Rich `json:"label,omitzero"`
	// Total is text after the label.
	Total Rich `json:"total,omitzero"`
}

// MeterStyle is a meter's shape.
type MeterStyle string

// Meter styles.
const (
	MeterBar    MeterStyle = "bar"
	MeterRing   MeterStyle = "ring"
	MeterBlocks MeterStyle = "blocks"
)

// MeterMark is a tick on a meter's track.
type MeterMark struct {
	At    float64  `json:"at"`
	Tone  Tone     `json:"tone,omitzero"`
	Title string   `json:"title,omitzero"`
	Icon  IconName `json:"icon,omitzero"`
}

// Node builds the meter.
func (m Meter) Node() tern.Node { return Build(tern.KindMeter, m, nil) }

// Chart is bars, a sparkline or a heatmap (kind chart).
type Chart struct {
	Common
	// Kind is heatmap (default), bars or spark.
	Kind ChartKind `json:"kind,omitzero"`
	// Size is sm, md (default) or lg.
	Size Size `json:"size,omitzero"`
	// Token is the fill color.
	Token string `json:"token,omitzero"`
	// Summary is a line under the chart.
	Summary Rich `json:"summary,omitzero"`
	// Series are the bars of bars and spark charts.
	Series []Point `json:"series,omitzero"`
	// Cells are a heatmap's rows of intensities, 0–1; nil cells are transparent.
	Cells [][]*float64 `json:"cells,omitzero"`
	// Tips are a heatmap's tooltips, indexed like Cells.
	Tips [][]string `json:"tips,omitzero"`
	// Rows label a heatmap's rows.
	Rows []string `json:"rows,omitzero"`
	// Cols label a heatmap's columns.
	Cols []ChartCol `json:"cols,omitzero"`
}

// ChartKind is which chart.
type ChartKind string

// Chart kinds.
const (
	ChartHeatmap ChartKind = "heatmap"
	ChartBars    ChartKind = "bars"
	ChartSpark   ChartKind = "spark"
)

// Point is one bar of a series.
type Point struct {
	Value float64 `json:"value"`
	Label string  `json:"label,omitzero"`
	Title string  `json:"title,omitzero"`
}

// ChartCol labels a heatmap column.
type ChartCol struct {
	At    int    `json:"at"`
	Label string `json:"label"`
}

// Node builds the chart.
func (c Chart) Node() tern.Node { return Build(tern.KindChart, c, nil) }

// Effort is a thinking-effort glyph (kind effort).
type Effort struct {
	Common
	// Level is the rung.
	Level EffortLevel `json:"level,omitzero"`
}

// EffortLevel is a thinking-effort rung.
type EffortLevel string

// Effort levels.
const (
	EffortOff     EffortLevel = "off"
	EffortMinimal EffortLevel = "minimal"
	EffortLow     EffortLevel = "low"
	EffortMedium  EffortLevel = "medium"
	EffortHigh    EffortLevel = "high"
	EffortXHigh   EffortLevel = "xhigh"
	EffortMax     EffortLevel = "max"
)

// Node builds the effort.
func (e Effort) Node() tern.Node { return Build(tern.KindEffort, e, nil) }
