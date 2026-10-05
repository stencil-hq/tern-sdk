package tern

import (
	"encoding/json"
	"os"
	"sync"
	"time"
)

// recorder appends every logical TSP message to a JSONL file
// (TERN_TSP_RECORD), the format Tern's surface-play replays.
type recorder struct {
	mu sync.Mutex
	f  *os.File
}

// recordLine is one line of a recording.
type recordLine struct {
	T      int64             `json:"t"`
	Dir    string            `json:"dir"`
	Verb   string            `json:"verb"`
	Params map[string]string `json:"params"`
	Body   any               `json:"body"`
}

// openRecorder opens path for appending; nil (recording nothing) when path
// is empty or can't be opened.
func openRecorder(path string) *recorder {
	if path == "" {
		return nil
	}
	f, err := os.OpenFile(path, os.O_CREATE|os.O_APPEND|os.O_WRONLY, 0o644)
	if err != nil {
		return nil
	}
	return &recorder{f: f}
}

// record appends a message whose body is JSON.
func (r *recorder) record(dir, verb string, params []Param, body []byte) {
	if r == nil {
		return
	}
	var v any = json.RawMessage(body)
	if !json.Valid(body) {
		v = string(body)
	}
	r.write(dir, verb, params, v)
}

// recordString appends a message whose body is text (a blob's base64).
func (r *recorder) recordString(dir, verb string, params []Param, body string) {
	if r == nil {
		return
	}
	r.write(dir, verb, params, body)
}

// write appends one line.
func (r *recorder) write(dir, verb string, params []Param, body any) {
	line, err := Marshal(recordLine{T: time.Now().UnixMilli(), Dir: dir, Verb: verb, Params: paramsObject(params), Body: body})
	if err != nil {
		return
	}
	r.mu.Lock()
	defer r.mu.Unlock()
	_, _ = r.f.Write(append(line, '\n'))
}

// close closes the file.
func (r *recorder) close() {
	if r == nil {
		return
	}
	r.mu.Lock()
	defer r.mu.Unlock()
	_ = r.f.Close()
}
