use std::fmt::Write as _;

use crate::{Image, Link, Row, ScheduleGroup, ScheduleRow};

use super::{BORDER, MUTED, PANEL, SECONDARY, TEXT, escape, linked_title, section_start};

pub fn cards(output: &mut String, heading: &str, rows: &[Row]) {
    section_start(output, heading);
    let _result = write!(
        output,
        r#"<table role="presentation" width="100%" cellspacing="0" cellpadding="0" border="0" style="background:{PANEL};border:1px solid {BORDER};">"#,
    );
    for row in rows {
        let title = linked_title(
            &row.title,
            &row.url,
            "font-family:Georgia,'Times New Roman',serif;font-size:15px;line-height:20px;color:#202124;text-decoration:underline;text-decoration-color:#6d4aff;",
        );
        let _result = write!(
            output,
            r#"<tr><td style="padding:10px 12px;border-bottom:1px solid {BORDER};">
<div style="font-size:11px;line-height:15px;color:{SECONDARY};margin-bottom:3px;">{eyebrow}</div>
{title}
</td></tr>"#,
            eyebrow = escape(&row.eyebrow),
        );
    }
    output.push_str("</table></td></tr>");
}

pub fn stories(output: &mut String, heading: &str, rows: &[Row]) {
    section_start(output, heading);
    for row in rows {
        let title = linked_title(
            &row.title,
            &row.url,
            "font-family:Georgia,'Times New Roman',serif;font-size:17px;line-height:23px;color:#202124;text-decoration:underline;text-decoration-color:#6d4aff;",
        );
        let _result = write!(
            output,
            r#"<table role="presentation" width="100%" cellspacing="0" cellpadding="0" border="0" style="border-bottom:1px solid {BORDER};"><tr><td style="padding:10px 0 11px;">
<div style="font-size:11px;line-height:15px;color:{SECONDARY};margin-bottom:4px;">{eyebrow}</div>
{title}
</td></tr></table>"#,
            eyebrow = escape(&row.eyebrow),
        );
    }
    output.push_str("</td></tr>");
}

pub fn schedule(output: &mut String, heading: &str, groups: &[ScheduleGroup]) {
    section_start(output, heading);
    for group in groups {
        let _result = write!(
            output,
            r#"<table role="presentation" width="100%" cellspacing="0" cellpadding="0" border="0" style="margin-bottom:12px;background:{PANEL};border:1px solid {BORDER};">
<tr><td colspan="3" style="padding:8px 10px;border-bottom:1px solid {BORDER};font-size:12px;line-height:16px;color:#5634d4;">{heading}</td></tr>"#,
            heading = escape(&group.heading),
        );
        for row in &group.rows {
            schedule_row(output, row);
        }
        output.push_str("</table>");
    }
    output.push_str("</td></tr>");
}

fn schedule_row(output: &mut String, row: &ScheduleRow) {
    let _result = write!(
        output,
        r#"<tr><td class="date-cell" width="52" valign="middle" style="width:52px;padding:9px 6px 9px 10px;border-bottom:1px solid {BORDER};">
<div style="font-size:10px;line-height:13px;color:{SECONDARY};">{weekday}</div>
<div style="font-size:19px;line-height:21px;font-weight:bold;color:{TEXT};">{day}</div>
<div style="font-size:10px;line-height:13px;color:{MUTED};">{month}</div>
</td>"#,
        weekday = escape(&row.weekday),
        day = escape(&row.day),
        month = escape(&row.month),
    );
    images(output, &row.images);
    let title = linked_title(
        &row.title,
        &row.url,
        "font-size:14px;line-height:19px;font-weight:bold;color:#202124;text-decoration:underline;text-decoration-color:#6d4aff;",
    );
    let _result = write!(
        output,
        r#"<td valign="middle" style="padding:9px 10px;border-bottom:1px solid {BORDER};">
{title}
<div style="margin-top:3px;font-size:11px;line-height:15px;color:{MUTED};">{metadata}</div>
</td></tr>"#,
        metadata = escape(&row.metadata),
    );
}

fn images(output: &mut String, images: &[Image]) {
    if images.is_empty() {
        return;
    }
    output.push_str(
        r#"<td width="66" valign="middle" style="width:66px;padding:8px 4px;border-bottom:1px solid #dadce0;"><table role="presentation" cellspacing="0" cellpadding="0" border="0"><tr>"#,
    );
    let plate = "#292b31";
    // Preserve the original SiftWire component's two decorative image slots.
    for image in images.iter().take(2) {
        let _result = write!(
            output,
            r#"<td style="padding-right:3px;"><img src="{url}" alt="" width="28" height="28" style="display:block;width:28px;height:28px;border:1px solid {BORDER};border-radius:50%;background:{plate};object-fit:contain;"></td>"#,
            url = escape(&image.url),
        );
    }
    output.push_str("</tr></table></td>");
}

pub fn summary(output: &mut String, heading: &str, lines: &[String], references: &[Link]) {
    if heading.is_empty() {
        output.push_str(r#"<tr><td class="mobile-pad" style="padding:17px 20px 0;">"#);
    } else {
        section_start(output, heading);
    }
    for line in lines {
        let _result = write!(
            output,
            r#"<p style="margin:0 0 9px;font-size:14px;line-height:19px;color:{TEXT};white-space:pre-wrap;">{line}</p>"#,
            line = escape(line),
        );
    }
    for link in references {
        let title = linked_title(
            &link.label,
            &link.url,
            "font-size:14px;line-height:19px;color:#202124;text-decoration:underline;text-decoration-color:#6d4aff;",
        );
        let _result = write!(output, r#"<div style="margin-bottom:9px;">{title}</div>"#);
    }
    output.push_str("</td></tr>");
}
