use std::{collections::HashMap, io};

pub mod line;
pub mod supported_instructions;
mod transform_directives;
mod transform_labels;

pub fn process_line(
    line: line::Line,
    supported_directives: &[String],
    labels: &HashMap<String, String>
) -> Result<Vec<line::Line>, io::Error> {
    let line = transform_labels::transform_labels(line, labels)?;
    transform_directives::transform_directives(line, supported_directives)
}