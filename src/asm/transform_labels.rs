use std::collections::HashMap;
use std::io;

use crate::asm::line::Line;

fn is_label_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '$'
}

fn replace_labels_in_arg(arg: &str, labels: &HashMap<String, String>) -> String {
    let mut out = String::with_capacity(arg.len());
    let mut token = String::new();

    for ch in arg.chars() {
        if is_label_char(ch) {
            token.push(ch);
            continue;
        }

        if !token.is_empty() {
            if let Some(replacement) = labels.get(&token) {
                out.push_str(replacement);
            } else {
                out.push_str(&token);
            }
            token.clear();
        }

        out.push(ch);
    }

    if !token.is_empty() {
        if let Some(replacement) = labels.get(&token) {
            out.push_str(replacement);
        } else {
            out.push_str(&token);
        }
    }

    out
}

pub(crate) fn transform_labels(
    mut line: Line,
    labels: &HashMap<String, String>,
) -> Result<Line, io::Error> {
    line.label = line.label.map(|label| {
        labels.get(&label).map(String::clone).unwrap_or(label)
    });

    if let Some(body) = line.body.as_mut() {
        let (_, args) = body.data_ref_mut();
        for arg in args.iter_mut() {
            *arg = replace_labels_in_arg(arg, labels);
        }
    }

    Ok(line)
}