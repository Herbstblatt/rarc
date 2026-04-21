use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Seek, Write};

use crate::asm::line::Line;
use crate::asm::process_line;

fn emit_header(writer: &mut BufWriter<File>) -> io::Result<()> {
    writer.write_all(b"# This file was generated automatically by the rarc tool. If this line has any lines above, DO NOT MODIFY THEM.\n")?;
    writer.write_all(b"\n")?;
    Ok(())
}

fn emit_main(writer: &mut BufWriter<File>) -> io::Result<()> {
    let content = [
        "\t.text\n",
        "\t.globl main\n",
        "main:\n",
        "\tcall __rarc_original_main\n",
        "\tli a7, 93\n",
        "\tecall\n",
        "\n"
    ];
    for line in content {
        writer.write_all(line.as_bytes())?;
    }

    Ok(())
}

pub fn process_file(
    reader: &mut BufReader<File>,
    writer: &mut BufWriter<File>,
    supported_directives: &[String],
) -> Result<(), Box<dyn Error>> {
    let mut labels: HashMap<String, String> = HashMap::new();

    for line in reader.lines() {
        let line = Line::new(line?);
        if let Some(label) = &line.label {
            labels.insert(label.clone(), label.clone());
        }
    }

    emit_header(writer)?;
    if labels.contains_key("main") {
        labels.insert("main".to_owned(), "__rarc_original_main".to_owned());
        emit_main(writer)?;
    }

    reader.get_mut().seek(std::io::SeekFrom::Start(0))?;
    for line in reader.lines() {
        let line = Line::new(line?);
        for processed in process_line(line, supported_directives, &labels)? {
            processed.emit(writer)?;
        }
    }

    writer.flush()?;
    Ok(())
}
