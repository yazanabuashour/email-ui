use std::io::{self, Read as _, Write as _};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use serde::Serialize;

// Inherited Mailgate transport compatibility tripwire; see receipts/render-input-size.json.
const REQUEST_LIMIT_BYTES: u64 = 1_048_576;

#[derive(Parser)]
#[command(
    version,
    about = "Render fixed email components offline; never send email"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Read an email-ui/render-request/v1 document from stdin and emit JSON.
    Render,
}

#[derive(Serialize)]
struct CliError {
    code: &'static str,
    path: &'static str,
    message: String,
}

impl CliError {
    fn new(code: &'static str, path: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            path,
            message: message.into(),
        }
    }
}

fn main() -> ExitCode {
    match run(&Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            write_error(&error);
            ExitCode::FAILURE
        }
    }
}

fn run(cli: &Cli) -> Result<(), CliError> {
    match cli.command {
        Command::Render => {
            let input = read_request(io::stdin().lock())?;
            let result = email_ui::render_json(&input)
                .map_err(|error| CliError::new(error.code(), "$", error.to_string()))?;
            write_json(io::stdout().lock(), &result)
        }
    }
}

fn read_request(input: impl io::Read) -> Result<Vec<u8>, CliError> {
    let requested_at_least = REQUEST_LIMIT_BYTES.saturating_add(1);
    let mut bytes = Vec::new();
    input
        .take(requested_at_least)
        .read_to_end(&mut bytes)
        .map_err(|_private_error| {
            CliError::new("input_io", "stdin request", "could not read stdin request")
        })?;
    let requested = u64::try_from(bytes.len()).map_err(|_private_error| {
        CliError::new(
            "input_io",
            "stdin request",
            "could not measure stdin request",
        )
    })?;
    if requested > REQUEST_LIMIT_BYTES {
        return Err(CliError::new(
            "input_limit",
            "stdin request",
            format!(
                "resource=stdin request configured_limit={REQUEST_LIMIT_BYTES} requested=at_least_{requested}"
            ),
        ));
    }
    Ok(bytes)
}

fn write_json(mut writer: impl io::Write, value: &impl Serialize) -> Result<(), CliError> {
    serde_json::to_writer_pretty(&mut writer, value).map_err(|_private_error| {
        CliError::new("output_io", "stdout", "could not write render result")
    })?;
    writeln!(writer).map_err(|_private_error| {
        CliError::new("output_io", "stdout", "could not finish render result")
    })
}

fn write_error(error: &CliError) {
    #[derive(Serialize)]
    struct ErrorEnvelope<'a> {
        schema_version: &'static str,
        error: &'a CliError,
    }

    let mut stderr = io::stderr().lock();
    let result = serde_json::to_writer_pretty(
        &mut stderr,
        &ErrorEnvelope {
            schema_version: "email-ui/error/v1",
            error,
        },
    );
    if result.is_ok() {
        // The process is already failing and has no safe fallback if stderr closes.
        drop(writeln!(stderr));
    }
}
