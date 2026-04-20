use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

pub fn process_directive(
    line: &String,
    writer: &mut BufWriter<File>,
    supported_directives: &[String]
) -> Result<bool, Box<dyn Error>> {
    let ident: String = line.chars().take_while(|c| c.is_whitespace()).collect();
    let mut args = line.split_whitespace();
    let directive_name = args.next().expect("Line is not empty");
    if supported_directives.iter().any(|d| d == directive_name) {
        return Ok(false);
    }

    match directive_name {
        ".bss" => {
            writer.write_all(ident.as_bytes())?;
            writer.write_all(b".data\n")?;
        },
        ".section" => {
            let section_name = args.next().expect("Section has a name");
            if section_name.starts_with(".rodata") {
                writer.write_all(ident.as_bytes())?;
                writer.write_all(b".data\n")?;
            }
        },
        ".zero" => {
            let mut arg: i32 = args.next()
                               .expect("Should have an argument")
                               .parse()
                               .expect("Argument should be an int");
            for _ in 0..(arg/4) {
                writer.write_all(ident.as_bytes())?;
                writer.write_all(b".word 0\n")?;
            }
            arg = arg % 4;
            
            for _ in 0..(arg/2) {
                writer.write_all(ident.as_bytes())?;
                writer.write_all(b".half 0\n")?;
            }
            arg = arg % 2;

            for _ in 0..arg {
                writer.write_all(ident.as_bytes())?;
                writer.write_all(b".byte 0\n")?;
            }
        },
        ".p2align" => {
            let arg = args.next()
                               .expect("Should have an argument");
            let arg: i32 = arg.strip_suffix(',')
                               .unwrap_or(arg)
                               .parse()?;
            if arg > 3 {
                return Err("Unsupported alignment".into());
            }
            writeln!(writer, "{}.align {}", ident, arg)?;
        },
        _ => {}
    }
    Ok(true)
}

pub fn process_file(
    reader: BufReader<File>,
    mut writer: BufWriter<File>,
    supported_directives: &[String],
) -> Result<(), Box<dyn Error>> {
    for line in reader.lines() {
        let line = line?;
        let skip: bool = {
            let mut iter = line.chars().skip_while(|c| c.is_whitespace());
            if let Some(sym) = iter.next() && sym == '.' {
                let last = iter.take_while(|c| !c.is_whitespace()).last();
                if let Some(sym) = last && sym != ':' {
                    process_directive(&line, &mut writer, supported_directives)?
                } else {
                    false
                }
            } else {
                false
            }
        };

        if !skip {
            writer.write_all(line.as_bytes())?;
            writer.write_all(b"\n")?;
        }
    }
    writer.flush()?;
    Ok(())
}