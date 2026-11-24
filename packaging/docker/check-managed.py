"""Runs inside the smoke-test container with its network disabled."""
import ctypes as c
import json
import time
from pathlib import Path
from urllib.request import Request, urlopen

project = Path("/openaec/SuperCloud project.ofs")
settings = Path("/config/org.openaec.open-frame-studio/settings.json")
x11 = c.CDLL("libX11.so.6")
x11.XOpenDisplay.argtypes = [c.c_char_p]
x11.XOpenDisplay.restype = c.c_void_p
x11.XKeysymToKeycode.argtypes = [c.c_void_p, c.c_ulong]
x11.XKeysymToKeycode.restype = c.c_uint
x11.XFlush.argtypes = [c.c_void_p]
xtest = c.CDLL("libXtst.so.6")
xtest.XTestFakeKeyEvent.argtypes = [c.c_void_p, c.c_uint, c.c_int, c.c_ulong]

# Wait for the persisted UI settings. A Ctrl+S in the running desktop must
# round-trip the actual requested document, adding its persistent project ID.
deadline = time.monotonic() + 100
while time.monotonic() < deadline:
    try:
        prefs = json.loads(settings.read_text())
        if prefs.get("locale") == "en" and prefs.get("theme") == "blue":
            break
    except (OSError, ValueError):
        pass
    time.sleep(1)
else:
    raise AssertionError("The desktop did not apply and persist its startup settings")

display = x11.XOpenDisplay(b":0")
assert display, "X display unavailable"
ctrl = x11.XKeysymToKeycode(display, 0xFFE3)
key_s = x11.XKeysymToKeycode(display, ord("s"))
while time.monotonic() < deadline:
    for key, down in [(ctrl, 1), (key_s, 1), (key_s, 0), (ctrl, 0)]:
        xtest.XTestFakeKeyEvent(display, key, down, 0)
    x11.XFlush(display)
    time.sleep(1)
    saved = json.loads(project.read_text())
    if saved.get("id"):
        break
else:
    raise AssertionError("Desktop Save did not write the startup document to the mounted folder")
assert saved["projectInfo"]["name"] == "Container smoke"

token = Path("/run/secrets/ofs-mcp").read_text().strip()
headers = {"Content-Type": "application/json", "Accept": "application/json, text/event-stream",
           "Authorization": "Bearer " + token}
def rpc(method, params):
    payload = {"jsonrpc": "2.0", "id": 1, "method": method, "params": params}
    request = Request("http://127.0.0.1:3001/mcp", data=json.dumps(payload).encode(), headers=headers)
    with urlopen(request, timeout=5) as response:
        return json.load(response)["result"]

init = rpc("initialize", {"protocolVersion": "2025-11-25", "capabilities": {},
                          "clientInfo": {"name": "container-smoke", "version": "1"}})
headers["MCP-Protocol-Version"] = init["protocolVersion"]
assert "frame_stand" in {tool["name"] for tool in rpc("tools/list", {})["tools"]}
result = rpc("tools/call", {"name": "frame_stand", "arguments": {}})
data = json.loads(result["content"][0]["text"])
assert data["project"]["name"] == "Container smoke" and data["file"]["readOnly"] is True
print("Managed locale, theme, native open/save and authenticated read-only MCP passed")
