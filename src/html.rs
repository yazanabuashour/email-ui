use std::fmt::Write as _;

use crate::{Document, Section};

mod components;

const TEXT: &str = "#202124";
const MUTED: &str = "#5f6368";
const ACCENT: &str = "#6d4aff";
const SECONDARY: &str = "#a14200";
const PANEL: &str = "#f7f5f1";
const BORDER: &str = "#dadce0";

pub fn render(document: &Document) -> String {
    let mut output = document_start(document);
    for section in &document.sections {
        match section {
            Section::Cards { heading, rows } => components::cards(&mut output, heading, rows),
            Section::Stories { heading, rows } => components::stories(&mut output, heading, rows),
            Section::Schedule { heading, groups } => {
                components::schedule(&mut output, heading, groups);
            }
            Section::Summary {
                heading,
                lines,
                links,
            } => components::summary(&mut output, heading, lines, links),
        }
    }
    if let Some(notice) = &document.notice {
        let _result = write!(
            output,
            r#"<tr><td class="mobile-pad" style="padding:5px 20px 14px;"><div style="padding:9px 10px;background:{PANEL};border-left:3px solid {SECONDARY};font-size:11px;line-height:16px;color:{MUTED};"><strong style="color:{TEXT};">{label}</strong> · {text}</div></td></tr>"#,
            label = escape(&notice.label),
            text = escape(&notice.text),
        );
    }
    if !document.footer.is_empty() {
        let _result = writeln!(
            output,
            r#"<tr><td class="mobile-pad" style="padding:12px 20px;border-top:1px solid {BORDER};font-size:11px;line-height:16px;color:{MUTED};">{footer}</td></tr>"#,
            footer = escape(&document.footer),
        );
    }
    output.push_str("</table></td></tr></table>\n</body>\n</html>");
    output
}

fn document_start(document: &Document) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="color-scheme" content="light">
<meta name="supported-color-schemes" content="light">
<title>{title}</title>
<style>
@media only screen and (max-width: 680px) {{
  .email-shell {{ width:100% !important; }}
  .mobile-pad {{ padding-left:16px !important; padding-right:16px !important; }}
  .story-title {{ font-size:16px !important; line-height:21px !important; }}
  .date-cell {{ width:45px !important; }}
}}
</style>
</head>
<body style="margin:0;padding:0;background:#ffffff;color:{TEXT};font-family:Arial,Helvetica,sans-serif;">
<div style="display:none;max-height:0;overflow:hidden;opacity:0;color:transparent;">{preheader}</div>
<table role="presentation" width="100%" cellspacing="0" cellpadding="0" border="0" style="width:100%;background:#ffffff;">
<tr><td align="center" style="padding:0;">
<table role="presentation" width="680" cellspacing="0" cellpadding="0" border="0" class="email-shell" style="width:680px;max-width:680px;background:#ffffff;border-top:3px solid {ACCENT};">
<tr><td class="mobile-pad" style="padding:13px 20px 12px;border-bottom:1px solid {BORDER};">
<table role="presentation" width="100%" cellspacing="0" cellpadding="0" border="0"><tr>
<td valign="top"><div style="font-family:Georgia,'Times New Roman',serif;font-size:21px;line-height:25px;font-weight:bold;color:{TEXT};">{heading}</div><div style="margin-top:3px;font-size:11px;line-height:15px;color:{MUTED};">{preheader}</div></td>
<td align="right" valign="top" style="font-size:11px;line-height:15px;color:{SECONDARY};">{edition}<br><span style="color:{MUTED};">{date}</span></td>
</tr></table>
</td></tr>"#,
        title = escape(&document.title),
        heading = escape(&document.heading),
        preheader = escape(&document.preheader),
        edition = escape(&document.header.edition),
        date = escape(&document.header.date),
    )
}

fn section_start(output: &mut String, heading: &str) {
    let _result = write!(
        output,
        r#"<tr><td class="mobile-pad" style="padding:17px 20px 0;"><h2 style="margin:0 0 9px;font-family:Georgia,'Times New Roman',serif;font-size:21px;line-height:26px;font-weight:normal;color:{TEXT};">{heading}</h2>"#,
        heading = escape(heading),
    );
}

fn linked_title(title: &str, url: &str, style: &str) -> String {
    let title = escape(title);
    if url.is_empty() {
        format!(r#"<span style="{style}">{title}</span>"#)
    } else {
        format!(r#"<a href="{}" style="{style}">{title}</a>"#, escape(url))
    }
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
