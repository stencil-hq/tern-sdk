package main

import (
	"io/fs"
	"testing"
	"testing/fstest"

	tern "github.com/stencil-hq/tern-sdk/go"
)

func TestDirectoryViewSeparatesHeaderAndFileKeys(t *testing.T) {
	entries, err := fs.ReadDir(fstest.MapFS{
		"head":       {Data: []byte("header name")},
		"entry:head": {Data: []byte("prefixed name")},
		"0":          {Data: []byte("index name")},
	}, ".")
	if err != nil {
		t.Fatal(err)
	}
	view := directoryView(entries)
	if _, err := tern.NewReconciler("table").Diff(view); err != nil {
		t.Fatalf("filename collides with the header: %v", err)
	}
	root := view.Main.Node()
	rows := root.Children[0].Node().Children
	if got := rows[0].Node().Props["key"]; got != "head" {
		t.Fatalf("header key = %v", got)
	}
	for i, entry := range entries {
		if got := rows[i+1].Node().Props["key"]; got != "entry:"+entry.Name() {
			t.Errorf("file %q key = %v", entry.Name(), got)
		}
	}
}
