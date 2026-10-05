//go:build !(darwin || linux || freebsd || netbsd || openbsd || dragonfly || windows)

package tern

import "os"

// pollable is false where input needs an explicit cancellation hook.
func pollable(*os.File) bool { return false }

// pollRead is never called where pollable is false.
func pollRead(*os.File, chan<- readResult, <-chan struct{}) {}
