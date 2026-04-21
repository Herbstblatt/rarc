use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

use crate::asm::line::Line;
use crate::asm::transform::process_line;

pub fn process_file(
    reader: BufReader<File>,
    mut writer: BufWriter<File>,
    supported_directives: &[String],
) -> Result<(), Box<dyn Error>> {
    for line in reader.lines() {
        let line = Line::new(line?);
        for processed in process_line(line, supported_directives) {
            processed.emit(&mut writer)?;
        }
    }
    writer.flush()?;
    Ok(())
}
