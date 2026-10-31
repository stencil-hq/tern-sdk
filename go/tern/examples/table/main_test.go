package main

import (
	"io/fs"
	"testing"
	"testing/fstest"

	"github.com/stencil-hq/tern-sdk/go/tern"
)

func TestDirectoryViewPlainAlignsSizes(t *testing.T) {
	entries, err := fs.ReadDir(fstest.MapFS{
		"a.txt":   {Data: make([]byte, 5)},
		"big.bin": {Data: make([]byte, 3000)},
		"src/x":   {Data: nil},
	}, ".")
	if err != nil {
		t.Fatal(err)
	}
	got := tern.Plain(directoryView(entries), 80)
	want := "Name     Size\na.txt       5\nbig.bin  2.9K\nsrc/\n"
	if got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}
