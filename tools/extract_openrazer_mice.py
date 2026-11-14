"""Dump every mouse OpenRazer supports (from the installed daemon) to JSON.

Usage: python3 tools/extract_openrazer_mice.py app/devices/openrazer_mice.json
Fields come straight from OpenRazer: USB PID, max DPI, settable poll rates, capabilities, image URL.
Factory DPI stages are NOT in OpenRazer; they live in app/devices/factory.json (researched separately).
"""

import inspect
import json
import re
import sys

from openrazer_daemon.hardware import mouse
from openrazer_daemon.hardware.device_base import RazerDevice

out = {}
for cls_name, cls in inspect.getmembers(mouse, inspect.isclass):
    if not issubclass(cls, RazerDevice) or cls.__module__ != mouse.__name__:
        continue
    methods = getattr(cls, "METHODS", []) or []
    if "get_device_type_mouse" not in methods:
        continue
    doc = (cls.__doc__ or "").strip()
    m = re.search(r"Class for the (.+)", doc)
    name = m.group(1).strip() if m else cls_name
    poll = cls.POLL_RATES or ([125, 500, 1000] if "set_poll_rate" in methods else [])
    out[f"{cls.USB_PID:04x}"] = {
        "class": cls_name,
        "name": name,
        "pid": f"{cls.USB_PID:04x}",
        "max_dpi": getattr(cls, "DPI_MAX", None),
        "poll_rates": sorted(poll),
        "dpi_stages": "set_dpi_stages" in methods,
        "dpi": "set_dpi_xy" in methods or "set_dpi_xy_byte" in methods,
        "battery": "get_battery" in methods,
        "image": getattr(cls, "DEVICE_IMAGE", None),
    }

json.dump(dict(sorted(out.items(), key=lambda kv: kv[1]["name"])), open(sys.argv[1], "w"), indent=1, ensure_ascii=False)
print(len(out), "mice")
