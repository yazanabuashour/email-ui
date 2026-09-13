#![expect(
    clippy::indexing_slicing,
    clippy::panic_in_result_fn,
    reason = "integration assertions identify renderer contract regressions"
)]

use std::error::Error;
use std::io::Write as _;
use std::process::{Command, Output, Stdio};

use serde_json::{Value, json};

const EXAMPLE: &[u8] = include_bytes!("../examples/render-request.json");

fn render(input: &[u8]) -> Result<Output, Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let mut child = Command::new(env!("CARGO_BIN_EXE_email-ui"))
        .env_clear()
        .current_dir(directory.path())
        .arg("render")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("missing stdin pipe")?
        .write_all(input)?;
    Ok(child.wait_with_output()?)
}

#[test]
fn offline_cli_matches_library_without_environment_or_config() -> Result<(), Box<dyn Error>> {
    let expected = email_ui::render_json(EXAMPLE)?;
    let mut boundary = EXAMPLE.to_vec();
    boundary.resize(1_048_576, b' ');
    for input in [EXAMPLE, boundary.as_slice()] {
        let output = render(input)?;
        assert!(
            output.status.success(),
            "renderer must not load configuration"
        );
        assert!(
            output.stderr.is_empty(),
            "renderer success has no diagnostics"
        );
        let actual: email_ui::RenderResult = serde_json::from_slice(&output.stdout)?;
        assert_eq!(actual, expected);
    }
    assert_eq!(
        expected.text,
        "Example automation\nA check needs attention.\nAlert\n\nCheck failed\nThe fictional service check did not complete.\nRunbook: https://example.test/runbook\n\nInspect before retrying.\n"
    );
    Ok(())
}

#[test]
fn invalid_render_input_never_echoes_private_values() -> Result<(), Box<dyn Error>> {
    let request: Value = serde_json::from_slice(EXAMPLE)?;
    let mut unknown = request.clone();
    unknown["document"]["sections"][0]["private-value-marker"] = json!("private-value-marker");
    let mut version = request.clone();
    version["schema_version"] = json!("private-value-marker");
    let mut unsafe_url = request.clone();
    unsafe_url["document"]["sections"][0]["links"][0]["url"] =
        json!("javascript:private-value-marker");
    let mut malformed_type = request;
    malformed_type["document"]["sections"][0]["lines"] = json!("private-value-marker");
    let invalid = [
        (
            b"{\"private-value-marker\":".to_vec(),
            "invalid_render_request",
        ),
        (serde_json::to_vec(&unknown)?, "invalid_render_request"),
        (serde_json::to_vec(&version)?, "unsupported_version"),
        (serde_json::to_vec(&unsafe_url)?, "unsafe_render_url"),
        (
            serde_json::to_vec(&malformed_type)?,
            "invalid_render_request",
        ),
    ];
    for (input, code) in invalid {
        let output = render(&input)?;
        assert!(!output.status.success(), "invalid input must fail");
        assert!(
            output.stdout.is_empty(),
            "failed rendering emits no partial body"
        );
        let stderr = String::from_utf8(output.stderr)?;
        assert!(
            !stderr.contains("private-value-marker"),
            "diagnostics must redact input"
        );
        let error: Value = serde_json::from_str(&stderr)?;
        assert_eq!(error["schema_version"], "email-ui/error/v1");
        assert_eq!(error["error"]["code"], code);
    }
    let output = render(&vec![b' '; 1_048_577])?;
    assert!(!output.status.success(), "oversized input must fail");
    assert!(output.stdout.is_empty(), "oversized input emits no body");
    let error: Value = serde_json::from_slice(&output.stderr)?;
    assert_eq!(error["error"]["code"], "input_limit");
    let message = error["error"]["message"]
        .as_str()
        .ok_or("missing limit message")?;
    assert!(
        message
            .contains("resource=stdin request configured_limit=1048576 requested=at_least_1048577"),
        "limit diagnostic includes resource and measurements"
    );
    Ok(())
}
