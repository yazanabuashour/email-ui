#![expect(
    clippy::indexing_slicing,
    clippy::panic_in_result_fn,
    reason = "fixture mutations and assertions protect the public render contract"
)]

use email_ui::{Document, RENDER_REQUEST_SCHEMA, RenderError, RenderRequest, render, render_json};
use serde_json::{Value, json};

const DOCUMENT: &str = include_str!("fixtures/rich-evening.json");

#[test]
fn shared_components_preserve_frozen_siftwire_bytes() -> Result<(), Box<dyn std::error::Error>> {
    let document: Document = serde_json::from_str(DOCUMENT)?;
    let result = render(&document)?;
    assert_eq!(result.html, include_str!("fixtures/rich-evening.html"));
    let request = RenderRequest {
        schema_version: RENDER_REQUEST_SCHEMA.to_owned(),
        document,
    };
    assert_eq!(render_json(&serde_json::to_vec(&request)?)?, result);
    Ok(())
}

#[test]
fn urls_and_nested_fields_share_one_safe_contract() -> Result<(), Box<dyn std::error::Error>> {
    let document: Value = serde_json::from_str(DOCUMENT)?;
    for url in [
        "javascript:private-value-marker",
        "data:text/html,private-value-marker",
        "https://private-value-marker@example.test/",
        "https://@example.test/",
        "https:///private-value-marker",
        "https://example.test/\nprivate-value-marker",
        "https://example.test/ private-value-marker",
        "https://example.test\\private-value-marker?query=value",
        "https://example.test/path\\private-value-marker#fragment",
        "//example.test/private-value-marker",
    ] {
        let mut changed = document.clone();
        changed["sections"][0]["rows"][0]["url"] = json!(url);
        let typed: Document = serde_json::from_value(changed.clone())?;
        assert_eq!(render(&typed), Err(RenderError::UnsafeUrl));
        let request = json!({"schema_version": RENDER_REQUEST_SCHEMA, "document": changed});
        assert_eq!(
            render_json(&serde_json::to_vec(&request)?),
            Err(RenderError::UnsafeUrl)
        );
    }
    let mut image = document.clone();
    image["sections"][2]["groups"][0]["rows"][0]["images"][0]["url"] = json!("");
    assert_eq!(
        render(&serde_json::from_value(image)?),
        Err(RenderError::UnsafeUrl)
    );
    let mut unknown = document.clone();
    unknown["sections"][2]["groups"][0]["rows"][0]["images"][0]["private-value-marker"] =
        json!("private-value-marker");
    let request = json!({"schema_version": RENDER_REQUEST_SCHEMA, "document": unknown});
    assert_eq!(
        render_json(&serde_json::to_vec(&request)?),
        Err(RenderError::InvalidRequest)
    );
    for (url, escaped) in [
        (
            "HTTP://example.test/?a=1&b=2",
            "HTTP://example.test/?a=1&amp;b=2",
        ),
        (
            r"https://example.test/article?pattern=\d",
            r"https://example.test/article?pattern=\d",
        ),
        (
            r"https://example.test/article#pattern=\d",
            r"https://example.test/article#pattern=\d",
        ),
    ] {
        let mut ordinary = document.clone();
        ordinary["sections"][0]["rows"][0]["url"] = json!(url);
        let result = render(&serde_json::from_value(ordinary.clone())?)?;
        let request = json!({"schema_version": RENDER_REQUEST_SCHEMA, "document": ordinary});
        assert_eq!(render_json(&serde_json::to_vec(&request)?)?, result);
        assert!(
            result.html.contains(escaped),
            "safe links retain exact original bytes before escaping"
        );
        assert!(
            result.text.contains(url),
            "plain text retains original URL bytes"
        );
    }
    Ok(())
}

#[test]
fn summary_escapes_content_and_omits_empty_headings() -> Result<(), Box<dyn std::error::Error>> {
    let mut document: Value = serde_json::from_str(DOCUMENT)?;
    document["footer"] = json!("");
    document["notice"] = Value::Null;
    document["sections"] = json!([{
        "kind": "summary", "heading": "", "lines": ["<&>\"'\nsecond line"],
        "links": [{"label": "No link <here>", "url": ""}]
    }]);
    let result = render(&serde_json::from_value(document)?)?;
    assert!(
        !result.html.contains("<h2"),
        "empty summary heading adds no heading"
    );
    assert!(
        !result.html.contains("padding:12px 20px;border-top:"),
        "empty footer adds no row"
    );
    assert!(
        result
            .html
            .contains("&lt;&amp;&gt;&quot;&#39;\nsecond line</p>"),
        "summary content uses the original escaping rules"
    );
    assert!(
        result.html.contains("No link &lt;here&gt;</span>"),
        "empty summary links use spans"
    );
    assert!(
        result
            .text
            .contains("<&>\"'\nsecond line\nNo link <here>\n"),
        "plain text preserves content and line breaks"
    );
    Ok(())
}
