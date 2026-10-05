// Command chat is an inline surface: a transcript in main and an editor in
// the dock that the program edits from keys. Enter sends, Escape quits.
// Outside Tern it reads lines from stdin instead.
package main

import (
	"bufio"
	"context"
	"errors"
	"fmt"
	"os"
	"strings"
	"unicode/utf16"

	"github.com/stencil-hq/tern-sdk/go/tern"
	"github.com/stencil-hq/tern-sdk/go/tern/ui"
)

// message is one transcript entry.
type message struct {
	user bool
	text string
}

// chat is the program's state.
type chat struct {
	sf     *tern.Surface
	log    []message
	draft  []rune
	cursor int
}

// render shows the current state.
func (c *chat) render() error { return c.sf.Render(c.view()) }

// reply is the program's answer to text.
func reply(text string) string {
	return fmt.Sprintf("You said *%s* (%d characters).", text, len([]rune(text)))
}

// send moves text into the transcript with its reply.
func (c *chat) send(text string) {
	if text = strings.TrimSpace(text); text == "" {
		return
	}
	c.log = append(c.log, message{true, text}, message{false, reply(text)})
}

// insert types s at the cursor.
func (c *chat) insert(s string) {
	r := []rune(s)
	c.draft = append(c.draft[:c.cursor], append(r, c.draft[c.cursor:]...)...)
	c.cursor += len(r)
}

// key applies one key to the draft; false quits.
func (c *chat) key(k tern.Key) bool {
	switch {
	case k.Is("escape"), k.Is("ctrl+c"), k.Is("ctrl+d"):
		return false
	case k.Is("enter"):
		c.send(string(c.draft))
		c.draft, c.cursor = nil, 0
	case k.Is("backspace"):
		if c.cursor > 0 {
			c.draft = append(c.draft[:c.cursor-1], c.draft[c.cursor:]...)
			c.cursor--
		}
	case k.Is("delete"):
		if c.cursor < len(c.draft) {
			c.draft = append(c.draft[:c.cursor], c.draft[c.cursor+1:]...)
		}
	case k.Is("left"):
		c.cursor = max(c.cursor-1, 0)
	case k.Is("right"):
		c.cursor = min(c.cursor+1, len(c.draft))
	case k.Is("home"), k.Is("ctrl+a"):
		c.cursor = 0
	case k.Is("end"), k.Is("ctrl+e"):
		c.cursor = len(c.draft)
	case k.Name == "paste" || k.Text != "" && !k.Ctrl && !k.Alt && !k.Meta:
		c.insert(k.Text)
	}
	return true
}

// view is the transcript and the composer.
func (c *chat) view() tern.View {
	var transcript tern.Nodes
	for i, m := range c.log {
		node := ui.Md{Text: m.text, Common: ui.Common{Key: fmt.Sprint("m", i)}}
		if m.user {
			node.Text = "**you** " + m.text
			node.Tone = ui.ToneUser
		}
		transcript = append(transcript, node)
	}
	if len(transcript) == 0 {
		transcript = tern.Nodes{ui.Text{Spans: []ui.Span{{T: "Say something.", S: "muted"}}, Common: ui.Common{Key: "hint"}}}
	}
	cursor := len(utf16.Encode(c.draft[:c.cursor]))
	editor := ui.Editor{
		Common: ui.Common{Key: "ed", OnSend: func(s *tern.Send) {
			c.send(s.Text)
			_ = c.render()
		}},
		Field: ui.Field{Text: string(c.draft), Cursor: &cursor, Sendable: true,
			Placeholder: "Message (Enter sends, Escape quits)", Prompt: []ui.Span{{T: "> ", S: "accent"}}},
	}
	return tern.View{
		Main: transcript,
		Dock: tern.Nodes{editor, ui.Status{Common: ui.Common{Key: "bar"}, Children: ui.Nodes(
			ui.Seg{Icon: ui.IconMessage, Text: fmt.Sprintf("%d messages", len(c.log))},
			ui.Seg{Side: ui.SideRight, Text: "esc to quit"},
		)}},
	}
}

func main() {
	ctx := context.Background()
	s, err := tern.Connect(ctx, tern.Options{Features: []string{tern.FeatureSend}})
	if errors.Is(err, tern.ErrUnsupported) {
		plain()
		return
	}
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	defer s.Close()
	sf, err := s.Open(tern.SurfaceOptions{Mode: tern.Inline, Title: "chat"})
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		return
	}
	c := &chat{sf: sf}
	_ = c.render()
	_ = sf.Focus("dock.ed")
	_ = s.Run(ctx, func(in tern.Input) error {
		if k, ok := in.(tern.Key); ok && !c.key(k) {
			return tern.ErrStop
		}
		return c.render()
	})
}

// plain chats line by line on stdin and stdout.
func plain() {
	in := bufio.NewScanner(os.Stdin)
	fmt.Print("> ")
	for in.Scan() {
		if text := strings.TrimSpace(in.Text()); text != "" {
			fmt.Println(reply(text))
		}
		fmt.Print("> ")
	}
	fmt.Println()
}
