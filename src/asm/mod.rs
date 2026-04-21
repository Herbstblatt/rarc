use std::io;

pub mod line;
pub mod transform_directives;

pub fn process_line(
    line: line::Line,
    supported_directives: &[String],
) -> Result<Vec<line::Line>, io::Error> {
    transform_directives::transform_directives(line, supported_directives)
}