package tern

import (
	"os"
	"os/signal"
	"sync"
	"syscall"
)

// onSignal runs fn when the process gets SIGINT, SIGTERM or SIGHUP, then
// raises the signal again so the process ends as it would have. The
// returned func stops watching.
func onSignal(fn func()) (stop func()) {
	ch := make(chan os.Signal, 1)
	quit := make(chan struct{})
	signal.Notify(ch, os.Interrupt, syscall.SIGTERM, syscall.SIGHUP)
	var once sync.Once
	stop = func() {
		once.Do(func() {
			signal.Stop(ch)
			close(quit)
		})
	}
	go func() {
		select {
		case sig := <-ch:
			fn()
			stop()
			raise(sig)
		case <-quit:
		}
	}()
	return stop
}
