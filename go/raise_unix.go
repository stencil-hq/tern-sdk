//go:build unix

package tern

import (
	"os"
	"syscall"
)

// raise sends sig to the process again, now with its default action.
func raise(sig os.Signal) {
	if s, ok := sig.(syscall.Signal); ok {
		_ = syscall.Kill(os.Getpid(), s)
	}
}
