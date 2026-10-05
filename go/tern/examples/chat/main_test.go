package main

import (
	"testing"

	"github.com/stencil-hq/tern-sdk/go/tern"
)

func TestDraftEditsCodePoints(t *testing.T) {
	c := new(chat)
	c.insert("A😀界e\u0301")
	c.key(tern.Key{Name: "home"})
	c.key(tern.Key{Name: "right"})
	c.key(tern.Key{Name: "delete"})
	if string(c.draft) != "A界e\u0301" {
		t.Fatal(string(c.draft))
	}
	c.key(tern.Key{Name: "right"})
	c.key(tern.Key{Name: "backspace"})
	c.insert("🙂")
	if string(c.draft) != "A🙂e\u0301" || c.cursor != 2 {
		t.Fatal(string(c.draft), c.cursor)
	}
}
