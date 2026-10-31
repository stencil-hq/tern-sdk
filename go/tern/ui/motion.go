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

// Chart is bars, a sparkline, a heatmap, a line, an area or a donut (kind
// chart). Its Min and Max are the value domain and shadow Common's size
// bounds.
type Chart struct {
	Common
	// Kind is heatmap (default), bars, spark, line, area or donut.
	Kind ChartKind `json:"kind,omitzero"`
	// Size is sm, md (default) or lg.
	Size Size `json:"size,omitzero"`
	// Token is the fill color, the first series' on a line or area chart.
	Token string `json:"token,omitzero"`
	// Summary is a line under the chart.
	Summary Rich `json:"summary,omitzero"`
	// Series are the bars of bars and spark charts, the points of a
	// one-series line or area chart, or a donut's segments.
	Series []Point `json:"series,omitzero"`
	// Min is the low end of a bars, spark, line or area chart's value
	// domain; values are clamped into it.
	Min *float64 `json:"min,omitzero"`
	// Max is the high end of a bars, spark, line or area chart's value
	// domain (default from the largest series or mark value).
	Max *float64 `json:"max,omitzero"`
	// Marks are dashed reference lines across a bars, spark, line or area chart.
	Marks []ChartMark `json:"marks,omitzero"`
	// Cells are a heatmap's rows of intensities, 0–1; nil cells are transparent.
	Cells [][]*float64 `json:"cells,omitzero"`
	// Tips are a heatmap's tooltips, indexed like Cells.
	Tips [][]string `json:"tips,omitzero"`
	// Rows label a heatmap's rows.
	Rows []string `json:"rows,omitzero"`
	// Cols label a heatmap's columns.
	Cols []ChartCol `json:"cols,omitzero"`
	// Lines are several line or area series, one line each, sent as series
	// in place of Series.
	Lines []ChartSeries `json:"-"`
	// Labels are a line or area chart's x labels by point index ("" leaves
	// a point unlabelled).
	Labels []string `json:"labels,omitzero"`
	// Unit is written with every value a line, area or donut chart prints: a
	// currency symbol ($, €, £, ¥) leads it, %, × and ° attach after it, words
	// follow a space.
	Unit string `json:"unit,omitzero"`
	// Last is the label beside a one-series line's last point (default its
	// value with Unit).
	Last Last `json:"last,omitzero"`
	// Total is what a donut's middle says (default the sum, with Unit).
	Total Rich `json:"total,omitzero"`
	// Caption is a quiet line under a donut's ring.
	Caption Rich `json:"caption,omitzero"`
}

// ChartKind is which chart.
type ChartKind string

// Chart kinds.
const (
	ChartHeatmap ChartKind = "heatmap"
	ChartBars    ChartKind = "bars"
	ChartSpark   ChartKind = "spark"
	ChartLine    ChartKind = "line"
	ChartArea    ChartKind = "area"
	ChartDonut   ChartKind = "donut"
)

// Point is one bar of a series, a point of a line or a donut's segment.
type Point struct {
	Value float64 `json:"value"`
	Label string  `json:"label,omitzero"`
	Title string  `json:"title,omitzero"`
	// Tone fills this bar instead of the chart's Token.
	Tone Tone `json:"tone,omitzero"`
	// Token colors a donut's segment.
	Token string `json:"token,omitzero"`
	// Gap breaks a line or area chart here; the point is sent as null.
	Gap bool `json:"-"`
}

// MarshalJSON writes the point, or null for a gap.
func (p Point) MarshalJSON() ([]byte, error) {
	if p.Gap {
		return []byte("null"), nil
	}
	type fields Point
	return tern.Marshal(fields(p))
}

// ChartSeries is one series of a line or area chart with several.
type ChartSeries struct {
	// Label names its legend chip and its part of the tooltip.
	Label string `json:"label,omitzero"`
	// Token is its color (default the next chart color).
	Token string `json:"token,omitzero"`
	// Values are its points, sharing x positions by index.
	Values []Point `json:"values"`
}

// Last is the label beside a line's last point: HideLast, or LastSays. Its
// zero value is absent (the value with the unit).
type Last struct{ v any }

// HideLast draws no label beside the last point.
var HideLast = Last{false}

// LastSays draws text beside the last point.
func LastSays(text Rich) Last { return Last{text} }

// IsZero reports whether l is absent.
func (l Last) IsZero() bool { return l.v == nil }

// MarshalJSON writes false or the text.
func (l Last) MarshalJSON() ([]byte, error) { return tern.Marshal(l.v) }

// ChartMark is a dashed reference line (a target, a limit) at Value, in
// value units; outside the domain it is clamped to its edge.
type ChartMark struct {
	Value float64 `json:"value"`
	// Tone colors the line (default muted foreground).
	Tone Tone `json:"tone,omitzero"`
	// Title is the line's tooltip.
	Title string `json:"title,omitzero"`
}

// ChartCol labels a heatmap column.
type ChartCol struct {
	At    int    `json:"at"`
	Label string `json:"label"`
}

// Node builds the chart.
func (c Chart) Node() tern.Node {
	n := Build(tern.KindChart, c, nil)
	if len(c.Lines) > 0 && n.Props["series"] == nil {
		lines := Build(tern.KindChart, struct {
			Series []ChartSeries `json:"series"`
		}{c.Lines}, nil)
		if n.Props == nil {
			n.Props = map[string]any{}
		}
		n.Props["series"] = lines.Props["series"]
	}
	return n
}

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

// Skeleton is placeholder blocks in the shape of content on its way (kind
// skeleton). Common.Tone tints the shapes; Common.Aria names it (default
// "Loading").
type Skeleton struct {
	Common
	// Variant is SkeletonShimmer (default), SkeletonPulse or SkeletonWire.
	Variant SkeletonVariant `json:"variant,omitzero"`
	// Shape is what it stands for (default ShapeText).
	Shape SkeletonShape `json:"shape,omitzero"`
	// Lines is ShapeText's line count, 1–24 (default 3).
	Lines int `json:"lines,omitzero"`
	// Rows is ShapeRows' row count, 1–50 (default 4).
	Rows int `json:"rows,omitzero"`
	// Tiles is ShapeCard's stat tile count, 1–8 (default 3).
	Tiles int `json:"tiles,omitzero"`
}

// SkeletonVariant is a skeleton's look.
type SkeletonVariant string

// Skeleton looks.
const (
	// SkeletonShimmer is soft blocks with one band sweeping across them.
	SkeletonShimmer SkeletonVariant = "shimmer"
	// SkeletonPulse is soft blocks fading in and out slowly.
	SkeletonPulse SkeletonVariant = "pulse"
	// SkeletonWire is dashed hairline outlines, still.
	SkeletonWire SkeletonVariant = "wire"
)

// SkeletonShape is what a skeleton stands for.
type SkeletonShape string

// Skeleton shapes.
const (
	ShapeText   SkeletonShape = "text"
	ShapeAvatar SkeletonShape = "avatar"
	ShapeRows   SkeletonShape = "rows"
	ShapeCard   SkeletonShape = "card"
	ShapeBlock  SkeletonShape = "block"
)

// Node builds the skeleton.
func (s Skeleton) Node() tern.Node { return Build(tern.KindSkeleton, s, nil) }
