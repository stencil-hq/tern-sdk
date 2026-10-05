//go:build darwin || linux || freebsd || netbsd || openbsd || dragonfly

package tern

import (
	"errors"
	"io"
	"os"

	"golang.org/x/sys/unix"
)

// pollInterval bounds how long the reader takes to notice Close.
const pollInterval = 10

// pollable reports whether f's fd can be polled, so reading it can stop
// without a byte arriving.
func pollable(f *os.File) bool {
	fds := []unix.PollFd{{Fd: int32(f.Fd()), Events: unix.POLLIN}}
	_, err := unix.Poll(fds, 0)
	return err == nil && fds[0].Revents&unix.POLLNVAL == 0
}

// pollRead reads f only when poll says it has input, checking done between
// polls, so nothing is read after the session stops.
func pollRead(f *os.File, chunks chan<- readResult, done <-chan struct{}) {
	fd := int(f.Fd())
	buf := make([]byte, 4096)
	fds := []unix.PollFd{{Fd: int32(fd), Events: unix.POLLIN}}
	for {
		select {
		case <-done:
			return
		default:
		}
		n, err := unix.Poll(fds, pollInterval)
		if errors.Is(err, unix.EINTR) || err == nil && n == 0 {
			continue
		}
		if err == nil && fds[0].Revents&(unix.POLLIN|unix.POLLHUP|unix.POLLERR) == 0 {
			continue
		}
		if err == nil {
			n, err = unix.Read(fd, buf)
			if errors.Is(err, unix.EINTR) || errors.Is(err, unix.EAGAIN) {
				continue
			}
			if err == nil && n == 0 {
				err = io.EOF
			}
		}
		if err != nil {
			select {
			case chunks <- readResult{err: err}:
			case <-done:
			}
			return
		}
		select {
		case chunks <- readResult{data: append([]byte(nil), buf[:n]...)}:
		case <-done:
			return
		}
	}
}
