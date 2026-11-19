"""Exercise the real plugin in a disposable desktop Tern profile (Python 3)."""
import json
import os
from pathlib import Path
import shutil
import shlex
import subprocess
import tempfile
import time
from unittest.mock import Mock, patch

from PIL import Image

TERN = shutil.which("tern")
assert TERN, "Put the Tern desktop CLI on PATH"
PLUGIN = Path(__file__).resolve().parent


def wait_for(predicate, seconds=15):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        if predicate():
            return
        time.sleep(0.1)
    raise AssertionError("Timed out waiting for Tern")


def stop_window(process, env, endpoint):
    # A failed graceful shutdown must not skip reaping our GUI child.
    try:
        for args in ([TERN, "kill", "session", "Default"],
                     [TERN, "ctl", "--control", endpoint, "quit"]):
            try:
                subprocess.run(args, env=env, capture_output=True, timeout=15)
            except subprocess.TimeoutExpired:
                print("WARN: graceful test shutdown timed out", flush=True)
    finally:
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=10)


def check_cleanup_timeout():
    child = Mock()
    child.wait.side_effect = [subprocess.TimeoutExpired("test-window", 10),
                              subprocess.TimeoutExpired("test-window", 10), 0]
    with patch.object(subprocess, "run", side_effect=subprocess.TimeoutExpired("shutdown", 15)) as run:
        stop_window(child, {}, "/unused-test-endpoint")
    assert run.call_count == 2
    child.terminate.assert_called_once_with()
    child.kill.assert_called_once_with()
    assert child.wait.call_count == 3
    print("PASS: shutdown timeouts still terminate and reap the test window", flush=True)


def main():
    with tempfile.TemporaryDirectory(prefix="tern-pane-hints-test-") as temp:
        profile = Path(temp)
        endpoint = str(profile / "window.sock")
        env = dict(os.environ)
        for key in list(env):
            if key.startswith("TERN_"):
                del env[key]
        env.update(TERN_CONFIG_DIR=temp, TERN_DAEMON_SOCKET=str(profile / "daemon.sock"),
                   STENCIL_LOG_DIR=str(profile / "logs"))
        (profile / "plugins").mkdir()
        (profile / "plugins/pane-hints.path").write_text(str(PLUGIN) + "\n")
        (profile / "settings.json").write_text(json.dumps({"pane_headers": "Always"}))

        def ctl(*args):
            result = subprocess.run([TERN, "ctl", "--control", endpoint, *args],
                                    env=env, capture_output=True, text=True,
                                    check=False, timeout=25)
            assert result.returncode == 0, (args, result.stdout, result.stderr)
            reply = json.loads(result.stdout)
            assert reply.get("ok"), reply
            return reply

        def state():
            return ctl("state")

        def hints(expected=None):
            ctl("key", "Ctrl+Meta+g")
            wait_for(lambda: state()["picker"] is not None)
            assert state()["picker"]["mode"] == "prompt"
            assert ctl("tree", ".tn-pl-list")["count"] == 0, "No pane list should be shown"
            sheet = next(s for s in ctl("css")["sheets"]
                         if s["name"] == "plugin:local:pane-hints:hints")
            if expected is None:
                assert sheet["rules"] > 2
            else:
                # Use the fixture's known count, not transient animation nodes.
                assert sheet["rules"] == expected + 2, ("Unexpected numbered rules", sheet["rules"], expected)

        def check_visible_numbers():
            # Generated glyphs are absent from the accessibility/layout text.
            # Inspect the actual rendered center of each visible pane instead.
            shot = ctl("shot", "pane-hints-regression")
            image = Image.open(profile / shot["png"]).convert("RGB")
            dump = ctl("dump", "#panel > .tn-stage > section.tn-pane")
            scale = dump["header"]["viewport"]["deviceScaleFactor"]
            visible = [e for e in dump["elements"] if e["visible"]]
            assert visible
            for element in visible:
                x, y, width, height = element["rect"]
                cx, cy = (x + width / 2) * scale, (y + height / 2) * scale
                crop = image.crop((int(cx - 50 * scale), int(cy - 65 * scale),
                                   int(cx + 50 * scale), int(cy + 65 * scale)))
                pixels = crop.load()
                bright = sum(min(pixels[x, y]) > 235
                             for y in range(crop.height) for x in range(crop.width))
                assert bright > 300 * scale * scale, ("Missing large pane number", element["nth"], bright)

        def choose(index, count):
            ctl("type", json.dumps(str(index).zfill(len(str(count)))))
            ctl("key", "Enter")
            wait_for(lambda: state()["picker"] is None)
            sheet = next(s for s in ctl("css")["sheets"]
                         if s["name"] == "plugin:local:pane-hints:hints")
            assert sheet["rules"] == 0, "Overlays must be removed after choosing"

        with (profile / "process.log").open("w") as log:
            process = subprocess.Popen([TERN, "--control", endpoint, temp],
                                       env=env, cwd=temp, stdout=log, stderr=log)
            try:
                wait_for(lambda: Path(endpoint).exists(), 30)
                ctl("ready")
                assert len(state()["panes"]) == 1, "Expected a fresh isolated window"
                ctl("split", "right")
                ctl("split", "down")
                initial = state()
                ids = [p["id"] for p in initial["panes"]]
                hints()
                check_visible_numbers()
                choose(1, 3)
                assert state()["focused"]["id"] == ids[0]
                hints()
                ctl("key", "Escape")
                assert state()["focused"]["id"] == ids[0]
                assert state()["picker"] is None
                hints()
                ctl("key", "9")
                ctl("key", "Enter")
                assert state()["focused"]["id"] == ids[0]
                assert state()["picker"] is None
                print("PASS: shortcut, prompt without list, selection, cancel, invalid number", flush=True)

                # Grow the largest pane each time, so small repeated splits do not
                # hit the native minimum-size limit before the multi-digit case.
                for _ in range(9):
                    panes = state()["panes"]
                    biggest = max(range(len(panes)), key=lambda i: panes[i]["cols"] * panes[i]["rows"])
                    hints()
                    choose(biggest + 1, len(panes))
                    pane = panes[biggest]
                    ctl("split", "right" if pane["cols"] > pane["rows"] * 2 else "down")
                panes = state()["panes"]
                assert len(panes) == 12
                hints()
                choose(10, 12)
                assert state()["focused"]["id"] == panes[9]["id"]
                hints()
                choose(1, 12)
                assert state()["focused"]["id"] == panes[0]["id"]
                ctl("zoom")
                hints()
                ctl("key", "Escape")
                ctl("zoom")
                print("PASS: selection mapping with 12 panes, distinct 01 and 10, prompt in zoom", flush=True)

                # Other tabs and parked panes must not leak into the current list.
                # Floats are rendered after tiled panes and must select the
                # same program as their overlay number.
                before_float = state()["focused"]["id"]
                ctl("pip", "float", "tr")
                hints()
                choose(12, 12)
                assert state()["focused"]["id"] == before_float
                ctl("pip", "dock", "right")
                print("PASS: floating pane selection targets the floating program", flush=True)
                ctl("tab", "new")
                other_id = state()["focused"]["id"]
                hints(expected=1)
                assert len(state()["panes"]) == 1
                check_visible_numbers()
                choose(2, 2)  # Only number 1 is valid in this tab.
                assert state()["focused"]["id"] == other_id
                ctl("tab", "1")
                ctl("plugins", "run", "park_pane")
                hints(expected=11)
                assert len(state()["panes"]) == 11
                focus_before_invalid = state()["focused"]["id"]
                choose(12, 12)  # The parked twelfth pane must not be selectable.
                assert state()["focused"]["id"] == focus_before_invalid
                assert state()["park"]["pile"]["cards"] == 1
                assert state()["focused"]["id"] != other_id
                print("PASS: other tabs and parked panes excluded", flush=True)

                # A raw reader proves digit+Enter are consumed by the picker,
                # rather than accidentally submitted to the focused program.
                ctl("tab", "2")
                ctl("split", "right")
                surviving = state()["focused"]["id"]
                hints()
                check_visible_numbers()
                assert len(state()["panes"]) == 2
                subprocess.run([TERN, "close", str(other_id)], env=env,
                               capture_output=True, text=True, check=True, timeout=15)
                wait_for(lambda: len(state()["panes"]) == 1)
                if state()["picker"] is not None:
                    choose(1, 2)
                assert state()["focused"]["id"] == surviving
                print("PASS: closing a target while choosing does not misdirect focus", flush=True)
                output = profile / "input.json"
                ready = profile / "reader-ready"
                code = ("import os,tty,termios,select,json; "
                        "old=termios.tcgetattr(0); tty.setraw(0); "
                        f"open({str(ready)!r},'w').close(); "
                        "data=os.read(0,32) if select.select([0],[],[],12)[0] else b''; "
                        "termios.tcsetattr(0,termios.TCSANOW,old); "
                        f"open({str(output)!r},'w').write(json.dumps(list(data)))")
                ctl("run", json.dumps("python3 -c " + shlex.quote(code)))
                wait_for(ready.exists)
                hints()
                choose(1, 1)
                wait_for(output.exists, 20)
                assert json.loads(output.read_text()) == [], output.read_text()
                print("PASS: selection keys do not reach the terminal", flush=True)
            finally:
                stop_window(process, env, endpoint)


if __name__ == "__main__":
    check_cleanup_timeout()
    main()
