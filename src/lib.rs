//! Offline, fixed email components. Callers own notification policy and metadata.
mod document;
mod html;
mod text;

pub use document::{
    Document, Header, Image, Link, Notice, Row, ScheduleGroup, ScheduleRow, Section,
};

use serde::{Deserialize, Serialize};
use url::Url;

pub const RENDER_REQUEST_SCHEMA: &str = "email-ui/render-request/v1";
pub const RENDER_RESULT_SCHEMA: &str = "email-ui/render-result/v1";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RenderRequest {
    pub schema_version: String,
    pub document: Document,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RenderResult {
    pub schema_version: String,
    pub text: String,
    pub html: String,
}

#[derive(Clone, Copy, Debug, thiserror::Error, PartialEq, Eq)]
pub enum RenderError {
    #[error("render request must be valid JSON with only the documented fields and types")]
    InvalidRequest,
    #[error("supported render schema is email-ui/render-request/v1")]
    UnsupportedVersion,
    #[error(
        "render links and images must use ordinary HTTP(S) URLs without credentials or whitespace"
    )]
    UnsafeUrl,
}

impl RenderError {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidRequest => "invalid_render_request",
            Self::UnsupportedVersion => "unsupported_version",
            Self::UnsafeUrl => "unsafe_render_url",
        }
    }
}

/// Decode the distinct renderer protocol without including input in diagnostics.
///
/// # Errors
/// Rejects malformed/unknown fields, unsupported versions, and unsafe URLs.
pub fn render_json(input: &[u8]) -> Result<RenderResult, RenderError> {
    let request: RenderRequest =
        serde_json::from_slice(input).map_err(|_private_error| RenderError::InvalidRequest)?;
    if request.schema_version != RENDER_REQUEST_SCHEMA {
        return Err(RenderError::UnsupportedVersion);
    }
    render(&request.document)
}

/// Render a document without configuration, credentials, filesystem, or network access.
///
/// # Errors
/// Rejects unsafe link and image URLs. Empty links render as spans.
pub fn render(document: &Document) -> Result<RenderResult, RenderError> {
    validate_urls(document)?;
    Ok(RenderResult {
        schema_version: RENDER_RESULT_SCHEMA.to_owned(),
        text: text::render(document),
        html: html::render(document),
    })
}

fn validate_urls(document: &Document) -> Result<(), RenderError> {
    for section in &document.sections {
        match section {
            Section::Cards { rows, .. } | Section::Stories { rows, .. } => {
                for row in rows {
                    validate_url(&row.url, true)?;
                }
            }
            Section::Schedule { groups, .. } => {
                for row in groups.iter().flat_map(|group| &group.rows) {
                    validate_url(&row.url, true)?;
                    for image in &row.images {
                        validate_url(&image.url, false)?;
                    }
                }
            }
            Section::Summary { links, .. } => {
                for link in links {
                    validate_url(&link.url, true)?;
                }
            }
        }
    }
    Ok(())
}

fn validate_url(value: &str, empty_allowed: bool) -> Result<(), RenderError> {
    if empty_allowed && value.is_empty() {
        return Ok(());
    }
    let (scheme, remainder) = value.split_once("://").ok_or(RenderError::UnsafeUrl)?;
    let authority = remainder.split(['/', '?', '#']).next().unwrap_or_default();
    if !(scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https"))
        || authority.is_empty()
        || authority.contains('@')
        || value
            .chars()
            .any(|character| character.is_whitespace() || character.is_control())
        || value.contains('\\')
    {
        return Err(RenderError::UnsafeUrl);
    }
    let url = Url::parse(value).map_err(|_private_error| RenderError::UnsafeUrl)?;
    if url.host_str().is_none() || !url.username().is_empty() || url.password().is_some() {
        return Err(RenderError::UnsafeUrl);
    }
    Ok(())
}
