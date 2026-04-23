use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Seek, Write};

use crate::asm::line::Line;
use crate::asm::process_line;
use crate::asm::symbol::Symbol;
use crate::asm::supported_instructions::is_supported_instruction;
use crate::asm::transform_reloc;

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
    let mut labels: HashMap<String, Symbol> = HashMap::new();

    for line in reader.lines() {
        let line = Line::new(line?);
        if let Some(label) = &line.label {
            labels.entry(label.clone())
                  .or_insert_with(|| Symbol::new(label.clone()));
        }

        if let Some(body) = line.body {
            let (name, args) = body.into_data();
            if name == ".local" {
                for local_name in args {
                    let local_symbol = labels
                        .entry(local_name.clone())
                        .or_insert_with(|| Symbol::new(local_name));
                    local_symbol.is_local = true;
                }
            }
        }
    }

    emit_header(writer)?;
    if labels.contains_key("main") {
        labels.insert("main".to_owned(), Symbol::new("__rarc_original_main".to_owned()));
        emit_main(writer)?;
    }
    for (label, replacement) in labels.iter_mut() {
        if is_supported_instruction(label) {
            *replacement = Symbol::new("__rarc_original_".to_owned() + label);
        }
    }

    reader.get_mut().seek(std::io::SeekFrom::Start(0))?;
    let mut normalized_lines: Vec<Line> = Vec::new();
    for line in reader.lines() {
        let line = Line::new(line?);
        for processed in process_line(line, supported_directives, &labels)? {
            normalized_lines.push(processed);
        }
    }

    transform_reloc::normalize_hi_lo_pairs(&mut normalized_lines);

    for line in normalized_lines {
        line.emit(writer)?;
    }

    writer.flush()?;
    Ok(())
}
