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
    assert not result.stderr, "successful rendering has no diagnostics"
    output = json.loads(result.stdout)
    assert set(output) == {"schema_version", "text", "html"}
    assert output["schema_version"] == "email-ui/render-result/v1"
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
            assert measured == item, "remeasure and update the input receipt"
            output = render(payload, directory)
            if path == "tests/fixtures/rich-evening.json":
                assert output["html"].encode() == (
                    ROOT / "tests/fixtures/rich-evening.html"
                ).read_bytes(), "release CLI must preserve the frozen SiftWire HTML"
            hashes = {
                f"{kind}_sha256": hashlib.sha256(output[kind].encode()).hexdigest()
                for kind in ("html", "text")
            }
            for name, digest in hashes.items():
                assert digest == PARITY[path][name], "preserve pre-extraction body bytes"
            measurements.append({**measured, **hashes})
    print(json.dumps({"offline_release_parity": "pass", "measurements": measurements}))


if __name__ == "__main__":
    main()
