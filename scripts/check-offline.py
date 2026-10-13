#!/usr/bin/env python3
"""Check the release executable using public synthetic fixtures only."""

import hashlib
import json
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent.parent
BINARY = ROOT / "target/release/email-ui"
RECEIPT = json.loads((ROOT / "receipts/render-input-size.json").read_text())
PARITY = {
    item["fixture"]: item
    for item in json.loads((ROOT / "receipts/standalone-parity.json").read_text())["fixtures"]
}


def render(payload, directory):
    result = subprocess.run(
        [str(BINARY), "render"],
        input=payload,
        capture_output=True,
        cwd=directory,
        env={},
        check=True,
    )
    if result.stderr:
        raise SystemExit("successful rendering must have no diagnostics")
    output = json.loads(result.stdout)
    if set(output) != {"schema_version", "text", "html"}:
        raise SystemExit("render result must contain only schema_version, text, and html")
    if output["schema_version"] != "email-ui/render-result/v1":
        raise SystemExit("unsupported render result schema")
    return output


def main():
    measurements = []
    with tempfile.TemporaryDirectory() as directory:
        for item in RECEIPT["standalone_measurements"]:
            path = item["fixture"]
            data = (ROOT / path).read_bytes()
            if path == "examples/render-request.json":
                payload = data
            else:
                payload = (
                    json.dumps(
                        {
                            "schema_version": "email-ui/render-request/v1",
                            "document": json.loads(data),
                        },
                        indent=2,
                        ensure_ascii=False,
                    )
                    + "\n"
                ).encode()
            measured = {
                "fixture": path,
                "fixture_bytes": len(data),
                "render_request_bytes": len(payload),
            }
            if measured != item:
                raise SystemExit(f"{path}: remeasure and update the input receipt")
            output = render(payload, directory)
            # Preserve historical receipts while allowing the mobile header change.
            original_html = (
                output["html"]
                .replace(' class="header-cell"', "")
                .replace(' class="header-cell header-meta"', "")
                .replace(
                    "  .header-cell { display:block !important; width:100% !important; }\n"
                    "  .header-meta { padding-top:8px !important; text-align:left !important; }\n",
                    "",
                )
            )
            hashes = {
                f"{kind}_sha256": hashlib.sha256(output[kind].encode()).hexdigest()
                for kind in ("html", "text")
            }
            if (
                hashlib.sha256(original_html.encode()).hexdigest()
                != PARITY[path]["html_sha256"]
            ):
                raise SystemExit(
                    f"{path}: preserve pre-extraction HTML except the approved mobile header"
                )
            if hashes["text_sha256"] != PARITY[path]["text_sha256"]:
                raise SystemExit(f"{path}: preserve pre-extraction plain-text bytes")
            measurements.append({**measured, **hashes})
    print(json.dumps({"offline_release_parity": "pass", "measurements": measurements}))


if __name__ == "__main__":
    main()
