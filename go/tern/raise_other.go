//go:build !unix

package tern

import "os"

// raise ends the process as an unhandled signal would.
func raise(os.Signal) { os.Exit(2) }
