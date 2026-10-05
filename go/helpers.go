package tern

import (
	"context"
	"errors"
	"io"
	"os"

	"golang.org/x/term"
)

// PrintOptions configure Print.
type PrintOptions struct {
	// CSS is a stylesheet installed (as "main") before the view.
	CSS string
	// Fallback is written instead of the plain-text rendering without TSP.
	Fallback string
	// Options configure the session Print connects.
	Options
}

// Print shows a static view that stays in the scrollback like command
// output: a flow surface opened with listen:false, its sheet, one frame,
// and a close that keeps it. Without TSP it writes the view's plain-text
// rendering (or Fallback) to Out instead.
func Print(ctx context.Context, view View, opts PrintOptions) error {
	s, err := Connect(ctx, opts.Options)
	if errors.Is(err, ErrUnsupported) {
		return writeFallback(view, opts.Fallback, opts.Out)
	}
	if err != nil {
		return err
	}
	err = printOn(s, view, opts.CSS)
	if e := s.Close(); err == nil {
		err = e
	}
	return err
}

// printOn shows view on s in a non-listening flow surface and closes it.
func printOn(s *Session, view View, css string) error {
	sf, err := s.Open(SurfaceOptions{Mode: Flow, NoListen: true})
	if err != nil {
		return err
	}
	if css != "" {
		if err := sf.Stylesheet("main", css); err != nil {
			return err
		}
	}
	err = sf.Render(view)
	if e := sf.Close(); err == nil {
		err = e
	}
	return err
}

// writeFallback writes fallback, or view as plain text, to out (stdout
// when nil), sized to the terminal when out is one.
func writeFallback(view View, fallback string, out io.Writer) error {
	if out == nil {
		out = os.Stdout
	}
	text := fallback
	if text == "" {
		cols := 0
		if f, ok := out.(*os.File); ok {
			if w, _, err := term.GetSize(int(f.Fd())); err == nil {
				cols = w
			}
		}
		text = Plain(view, cols)
	}
	if text != "" && text[len(text)-1] != '\n' {
		text += "\n"
	}
	_, err := io.WriteString(out, text)
	return err
}

// AskOptions configure Ask.
type AskOptions struct {
	// CSS is a stylesheet installed (as "main") before the view.
	CSS string
	// Submit is the action name that answers (default "submit").
	Submit string
	// Options configure the session Ask connects.
	Options
}

// Answer is the action that submitted an Ask form.
type Answer struct {
	// ID is the node that sent the action (the submit button).
	ID string
	// Act is the action's name.
	Act string
	// Values are the form's named controls as the user left them.
	Values Values
}

// Ask shows view (a form) in a flow surface and waits for the first action
// named opts.Submit, returning its id, act and form values. It returns a
// nil answer when the user presses Escape, Ctrl+C or Ctrl+D, and
// ErrUnsupported without TSP, so the program can ask its own way. Handlers
// on the view's nodes run while it waits. The form stays in the scrollback.
func Ask(ctx context.Context, view View, opts AskOptions) (*Answer, error) {
	if opts.Submit == "" {
		opts.Submit = "submit"
	}
	s, err := Connect(ctx, opts.Options)
	if err != nil {
		return nil, err
	}
	ans, err := askOn(ctx, s, view, opts)
	if e := s.Close(); err == nil {
		err = e
	}
	return ans, err
}

// askOn runs Ask on a connected session.
func askOn(ctx context.Context, s *Session, view View, opts AskOptions) (answer *Answer, err error) {
	sf, err := s.Open(SurfaceOptions{Mode: Flow})
	if err != nil {
		return nil, err
	}
	defer func() {
		if closeErr := sf.Close(); err == nil {
			err = closeErr
		}
	}()
	if opts.CSS != "" {
		if err := sf.Stylesheet("main", opts.CSS); err != nil {
			return nil, err
		}
	}
	if err := sf.Render(view); err != nil {
		return nil, err
	}
	for {
		in, err := s.pop(ctx)
		if err != nil {
			return nil, err
		}
		switch e := in.(type) {
		case Key:
			if e.Is("escape") || e.Is("ctrl+c") || e.Is("ctrl+d") {
				return nil, nil
			}
		case *Action:
			s.dispatch(e)
			if e.SF == sf.ID() && e.Act == opts.Submit {
				return &Answer{ID: e.ID, Act: e.Act, Values: e.Values}, nil
			}
		case Event:
			s.dispatch(e)
		}
	}
}
