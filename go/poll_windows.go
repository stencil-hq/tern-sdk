//go:build windows

package tern

import (
	"os"
	"strings"
	"unicode/utf16"
	"unsafe"

	"golang.org/x/sys/windows"
)

var readConsoleInput = windows.NewLazySystemDLL("kernel32.dll").NewProc("ReadConsoleInputW")

// consoleRecord mirrors INPUT_RECORD's KEY_EVENT_RECORD union (20 bytes).
type consoleRecord struct {
	EventType  uint16
	Padding    uint16
	Down       int32
	Repeat     uint16
	VirtualKey uint16
	Scan       uint16
	Char       uint16
	Control    uint32
}

func pollable(f *os.File) bool {
	var mode uint32
	return windows.GetConsoleMode(windows.Handle(f.Fd()), &mode) == nil
}

// pollRead consumes records rather than calling a character read after a
// signalled wait: modifiers, focus and resize records must not block Close.
func pollRead(f *os.File, chunks chan<- readResult, done <-chan struct{}) {
	h := windows.Handle(f.Fd())
	var high uint16
	for {
		select {
		case <-done:
			return
		default:
		}
		state, err := windows.WaitForSingleObject(h, 10)
		if err == nil && state == uint32(windows.WAIT_TIMEOUT) {
			continue
		}
		var rec consoleRecord
		var count uint32
		if err == nil {
			result, _, callErr := readConsoleInput.Call(uintptr(h), uintptr(unsafe.Pointer(&rec)), 1, uintptr(unsafe.Pointer(&count)))
			if result == 0 {
				err = callErr
			}
		}
		if err != nil {
			select {
			case chunks <- readResult{err: err}:
			case <-done:
			}
			return
		}
		if count == 0 || rec.EventType != 1 || rec.Down == 0 {
			continue
		}
		var text string
		if rec.Char >= 0xd800 && rec.Char <= 0xdbff {
			high = rec.Char
			continue
		}
		if rec.Char >= 0xdc00 && rec.Char <= 0xdfff && high != 0 {
			text = string(utf16.DecodeRune(rune(high), rune(rec.Char)))
		} else if rec.Char != 0 {
			text = string(rune(rec.Char))
		} else {
			text = consoleKey(rec.VirtualKey, rec.Control)
		}
		high = 0
		if text == "" {
			continue
		}
		if rec.Repeat > 1 {
			text = strings.Repeat(text, int(rec.Repeat))
		}
		select {
		case chunks <- readResult{data: []byte(text)}:
		case <-done:
			return
		}
	}
}

func consoleKey(key uint16, control uint32) string {
	// VT mode supplies character sequences for most keys; these cover native
	// console key records with no character. Lone modifiers are ignored.
	switch key {
	case 0x25:
		return "\x1b[D"
	case 0x26:
		return "\x1b[A"
	case 0x27:
		return "\x1b[C"
	case 0x28:
		return "\x1b[B"
	case 0x21:
		return "\x1b[5~"
	case 0x22:
		return "\x1b[6~"
	case 0x23:
		return "\x1b[F"
	case 0x24:
		return "\x1b[H"
	case 0x2d:
		return "\x1b[2~"
	case 0x2e:
		return "\x1b[3~"
	case 0x20:
		if control&0xc != 0 {
			return "\x00"
		}
	}
	return ""
}
