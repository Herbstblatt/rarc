use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};

pub fn process_file(
    reader: BufReader<File>,
    mut writer: BufWriter<File>,
    _supported_directives: &[String],
) -> io::Result<()> {
    for line in reader.lines() {
        writer.write_all(line?.as_bytes())?;
        writer.write_all(b"\n")?;
    }
    writer.flush()?;
    Ok(())
}