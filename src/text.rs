use std::fmt::Write as _;

use crate::{Document, Section};

pub fn render(document: &Document) -> String {
    let mut output = String::new();
    line(&mut output, &document.heading);
    line(&mut output, &document.preheader);
    line(&mut output, &document.header.edition);
    line(&mut output, &document.header.date);
    for section in &document.sections {
        output.push('\n');
        match section {
            Section::Cards { heading, rows } | Section::Stories { heading, rows } => {
                line(&mut output, heading);
                for row in rows {
                    line(&mut output, &row.eyebrow);
                    link(&mut output, &row.title, &row.url);
                }
            }
            Section::Schedule { heading, groups } => {
                line(&mut output, heading);
                for group in groups {
                    line(&mut output, &group.heading);
                    for row in &group.rows {
                        let _result = writeln!(output, "{} {} {}", row.weekday, row.day, row.month);
                        link(&mut output, &row.title, &row.url);
                        line(&mut output, &row.metadata);
                    }
                }
            }
            Section::Summary {
                heading,
                lines,
                links,
            } => {
                line(&mut output, heading);
                for value in lines {
                    line(&mut output, value);
                }
                for value in links {
                    link(&mut output, &value.label, &value.url);
                }
            }
        }
    }
    if let Some(notice) = &document.notice {
        let _result = writeln!(output, "\n{} · {}", notice.label, notice.text);
    }
    if !document.footer.is_empty() {
        output.push('\n');
        line(&mut output, &document.footer);
    }
    output
}

fn line(output: &mut String, value: &str) {
    if !value.is_empty() {
        output.push_str(value);
        output.push('\n');
    }
}

fn link(output: &mut String, label: &str, url: &str) {
    if url.is_empty() {
        line(output, label);
    } else {
        let _result = writeln!(output, "{label}: {url}");
    }
}
