#!/bin/sh

set -eu

cd "$(dirname "$0")/.."

python3 - <<'PY'
from html.parser import HTMLParser
from pathlib import Path


class ShellAudit(HTMLParser):
    def __init__(self):
        super().__init__()
        self.elements = []
        self.text_by_id = {}
        self.labels = set()
        self.summary_parents = set()
        self.active_ids = []

    def handle_starttag(self, tag, attributes):
        attrs = dict(attributes)
        self.elements.append((tag, attrs))
        element_id = attrs.get("id")
        if element_id is not None:
            self.text_by_id.setdefault(element_id, [])
            self.active_ids.append(element_id)
        if tag == "label" and attrs.get("for"):
            self.labels.add(attrs["for"])
        if tag == "summary":
            self.summary_parents.update(self.active_ids)

    def handle_endtag(self, tag):
        if self.active_ids and any(
            attributes.get("id") == self.active_ids[-1]
            for element_tag, attributes in reversed(self.elements)
            if element_tag == tag
        ):
            self.active_ids.pop()

    def handle_data(self, data):
        for element_id in self.active_ids:
            self.text_by_id[element_id].append(data)


html = Path("web/index.html").read_text()
bootstrap = Path("web/bootstrap.js").read_text()
audit = ShellAudit()
audit.feed(html)
by_id = {
    attributes["id"]: (tag, attributes)
    for tag, attributes in audit.elements
    if "id" in attributes
}


def require(condition, message):
    if not condition:
        raise SystemExit(message)


html_elements = [attributes for tag, attributes in audit.elements if tag == "html"]
require(html_elements and html_elements[0].get("lang"), "document language is missing")
require(any(tag == "main" for tag, _ in audit.elements), "main landmark is missing")
require("browser-support" in audit.summary_parents, "support disclosure summary is missing")

live_regions = {
    "game-status": ("status", "polite"),
    "game-diagnostics": ("alert", "assertive"),
}
for element_id, (role, live) in live_regions.items():
    tag, attributes = by_id.get(element_id, (None, {}))
    require(tag is not None, f"required shell element is missing: {element_id}")
    require(attributes.get("role") == role, f"{element_id} role is invalid")
    require(attributes.get("aria-live") == live, f"{element_id} live region is invalid")

status_attributes = by_id["game-status"][1]
require(status_attributes.get("aria-atomic") == "true", "game status must be atomic")
log_tag, log_attributes = by_id.get("game-log", (None, {}))
require(log_tag == "p", "keyboard help must remain a paragraph")
require("role" not in log_attributes and "aria-live" not in log_attributes, "keyboard help must not be a live region")

for button_id in (
    "start-button",
    "restart-button",
    "save-button",
    "load-button",
    "clear-save-button",
    "mute-button",
):
    tag, attributes = by_id.get(button_id, (None, {}))
    require(tag == "button", f"{button_id} must be a button")
    require(attributes.get("type") == "button", f"{button_id} must declare type=button")
    name = "".join(audit.text_by_id.get(button_id, [])).strip()
    require(name or attributes.get("aria-label"), f"{button_id} has no accessible name")

tag, attributes = by_id.get("volume-control", (None, {}))
require(tag == "input" and attributes.get("type") == "range", "volume control is not a range input")
require("volume-control" in audit.labels, "volume control label association is missing")

tag, attributes = by_id.get("game-canvas", (None, {}))
require(tag == "canvas", "game canvas is missing")
require(attributes.get("tabindex") is not None, "game canvas is not keyboard focusable")
require(attributes.get("aria-label"), "game canvas has no accessible name")
require(attributes.get("aria-describedby") == "game-log", "game canvas is not associated with keyboard help")

tag, attributes = by_id.get("inventory", (None, {}))
require(tag is not None and attributes.get("role") == "region", "inventory region role is missing")
require(attributes.get("aria-label"), "inventory region is unnamed")
require("button:focus-visible" in html, "focus-visible control styling is missing")
require("clearDiagnostic();\n    const result = await boot();" in bootstrap, "startup must clear stale diagnostics before boot")
require("if (diagnostics.hidden)" in bootstrap, "startup must preserve boot diagnostics and focus")
for tag, attributes in audit.elements:
    if tag == "img":
        require(attributes.get("alt") is not None, "image is missing alt text")

print("Browser accessibility shell audit: PASS")
PY
