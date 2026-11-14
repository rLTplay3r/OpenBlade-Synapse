"""Merge researched factory defaults (tools/research/*.json) into app/devices/factory.json.

Each research record: product, dpi_stages, default_dpi, default_poll_hz, max_poll_hz, max_dpi,
confidence, sources, notes. Product names are matched to OpenRazer's product names (aliases below).
"""

import glob
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
_VARIANT = re.compile(r"\s*\((Wired|Wireless|Receiver|Bluetooth)\)\s*$")


def all_products():
    """Product names of every mouse in app/devices/openrazer_mice.json (variants merged)."""
    mice = json.loads((ROOT / "app" / "devices" / "openrazer_mice.json").read_text(encoding="utf-8"))
    return sorted({_VARIANT.sub("", m["name"]).strip() for m in mice.values()})

# research name -> OpenRazer product name(s) it also covers
ALIASES = {
    "Razer Naga Left-Handed Edition 2020": ["Razer Naga Left Handed Edition 2020"],
    "Razer Naga Hex": ["Razer Naga Hex", "Razer Naga Hex (Red)"],
    "Razer Viper Mini Signature Edition": ["Razer Viper Mini SE"],
    "Razer Basilisk V3 Pro 35K": ["Razer Basilisk V3 Pro 35K", "Razer Basilisk V3 Pro 35K Phantom Green Edition"],
    "Razer DeathAdder 3.5G": ["Razer DeathAdder 3.5G", "Razer DeathAdder 3.5G Black"],
    "Razer DeathAdder Essential": ["Razer DeathAdder Essential", "Razer DeathAdder Essential (White Edition)"],
    "Razer Basilisk Ultimate": ["Razer Basilisk Ultimate", "RazerBasiliskUltimateReceiver"],
    "Razer Mamba Wireless": ["Razer Mamba Wireless"],
    "Razer Lancehead Wireless": ["Razer Lancehead Wireless"],
}

# Verified by hand in this project (master guide + razer.com), not part of a research batch.
MANUAL = [{
    "product": "Razer DeathAdder V3", "dpi_stages": [400, 800, 1600, 3200, 6400], "default_dpi": 1600,
    "default_poll_hz": 1000, "max_poll_hz": 8000, "max_dpi": 30000, "confidence": "high",
    "sources": ["https://dl.razerzone.com/master-guides/RazerSynapse3/DEATHADDERV3-00000178-en.pdf",
                "https://www.razer.com/gaming-mice/razer-deathadder-v3"],
    "notes": "Master guide: stages 400, 800, 1600 (default), 3200, 6400; Synapse FAQ: default 1000 Hz, up to 8000 Hz.",
}]

FIELDS = ("dpi_stages", "default_dpi", "default_poll_hz", "max_poll_hz", "max_dpi", "confidence", "sources", "notes")


def norm(name):
    name = re.sub(r"\s*\(.*?\)\s*", " ", name)  # drop "(2015 Chroma, 8200 DPI)" etc.
    return re.sub(r"\s+", " ", name).strip().lower()


def main():
    products = all_products()
    by_norm = {norm(p): p for p in products}
    records = list(MANUAL)
    for f in sorted(glob.glob(str(ROOT / "tools" / "research" / "*.json"))):
        records += json.load(open(f, encoding="utf-8"))

    out, unmatched = {}, []
    for r in records:
        # exact name first: "DeathAdder Essential (2021)" must not collapse into "DeathAdder Essential"
        exact = r["product"] if r["product"] in products else None
        targets = ALIASES.get(r["product"]) or [exact or by_norm.get(norm(r["product"])) or r["product"]]
        for t in targets:
            if t not in products:
                unmatched.append(f"{r['product']} -> {t}")
                continue
            entry = {k: r.get(k) for k in FIELDS}
            stages = entry["dpi_stages"]
            if stages and entry["default_dpi"] and entry["default_dpi"] not in stages:
                entry["notes"] = (entry["notes"] or "") + " [default_dpi not among stages]"
            out[t] = entry

    missing = [p for p in products if p not in out]
    data = {"_comment": "Factory defaults per product, researched from official Razer sources "
                        "(master guides on dl.razerzone.com, mysupport.razer.com, razer.com). "
                        "Built by tools/build_factory.py from tools/research/*.json. null = not documented.",
            **dict(sorted(out.items()))}
    (ROOT / "app" / "devices" / "factory.json").write_text(json.dumps(data, ensure_ascii=False, indent=1) + "\n",
                                                           encoding="utf-8")
    print(f"{len(out)} products written, {len(missing)} without data")
    for u in unmatched:
        print("UNMATCHED", u)
    for m in missing:
        print("MISSING", m)


if __name__ == "__main__":
    main()
