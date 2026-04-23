use std::iter;
use std::io;
use std::collections::HashMap;

use crate::asm::line::{Body, Line};
use crate::asm::symbol::Symbol;
use crate::asm::supported_instructions::is_supported_instruction;

fn parse_i32_arg(arg: Option<&String>, err_msg: String) -> Result<i32, io::Error> {
    arg.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, err_msg.clone()))?
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, err_msg))
}

fn parse_comm_align_pow2(comm_align: i32, raw_line: &str) -> Result<i32, io::Error> {
    if comm_align <= 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Directive .comm alignment must be a positive integer: '{}'",
                raw_line
            ),
        ));
    }

    if (comm_align & (comm_align - 1)) != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Directive .comm alignment must be a power of two: '{}'",
                raw_line
            ),
        ));
    }

    Ok(comm_align.trailing_zeros() as i32)
}

fn zero_fill_chunks(mut size: i32) -> Vec<(&'static str, String)> {
    let mut chunks: Vec<(&'static str, String)> = Vec::new();

    for _ in 0..(size / 4) {
        chunks.push((".word", "0".to_owned()));
    }
    size %= 4;

    for _ in 0..(size / 2) {
        chunks.push((".half", "0".to_owned()));
    }
    size %= 2;

    for _ in 0..size {
        chunks.push((".byte", "0".to_owned()));
    }

    chunks
}

fn emit_zero_fill_from_line(line: Line, size: i32) -> Vec<Line> {
    let ident = line.ident.clone();
    let mut extend_iter = iter::once(line).chain(iter::repeat_with(Line::default));

    zero_fill_chunks(size)
        .into_iter()
        .map(|(name, arg)| Line {
            body: Some(Body::Directive {
                name: name.to_owned(),
                args: vec![arg],
            }),
            ident: ident.clone(),
            ..extend_iter.next().expect("infinite iterator")
        })
        .collect()
}

fn emit_zero_fill_lines(ident: &str, size: i32) -> Vec<Line> {
    zero_fill_chunks(size)
        .into_iter()
        .map(|(name, arg)| Line {
            ident: ident.to_owned(),
            body: Some(Body::Directive {
                name: name.to_owned(),
                args: vec![arg],
            }),
            ..Line::default()
        })
        .collect()
}

fn handle_bss(line: Line, args: &[String]) -> Vec<Line> {
    vec![Line {
        body: Some(Body::Directive {
            name: ".data".to_owned(),
            args: args.to_vec(),
        }),
        ..line
    }]
}

fn handle_section(line: Line, args: &[String]) -> Result<Vec<Line>, io::Error> {
    let section_name = args.first().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Directive .section requires an argument: '{}'", line.raw_content),
        )
    })?;
    if section_name.starts_with(".rodata") {
        return Ok(vec![Line {
            body: Some(Body::Directive {
                name: ".data".to_owned(),
                args: vec![],
            }),
            ..line
        }]);
    }
    Ok(vec![])
}

fn parse_zero_size(args: &[String], raw_line: &str) -> Result<i32, io::Error> {
    let size: i32 = args
        .first()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Directive .zero requires an integer argument: '{}'", raw_line),
            )
        })?
        .parse()
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Directive .zero argument must be an integer: '{}'", raw_line),
            )
        })?;

    if size < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Directive .zero size must be non-negative: '{}'", raw_line),
        ));
    }

    Ok(size)
}

fn handle_zero(line: Line, args: &[String]) -> Result<Vec<Line>, io::Error> {
    let size = parse_zero_size(args, &line.raw_content)?;
    Ok(emit_zero_fill_from_line(line, size))
}

fn handle_p2align(line: Line, args: &[String]) -> Result<Vec<Line>, io::Error> {
    let arg: i32 = args
        .first()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Directive .p2align requires an integer argument: '{}'", line.raw_content),
            )
        })?
        .parse()
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Directive .p2align argument must be an integer: '{}'", line.raw_content),
            )
        })?;
    if arg > 3 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unsupported .p2align value {} in line: '{}'", arg, line.raw_content),
        ));
    }
    Ok(vec![Line {
        body: Some(Body::Directive {
            name: ".align".to_owned(),
            args: vec![arg.to_string()],
        }),
        ..line
    }])
}

fn handle_comm(
    line: Line,
    args: &[String],
    symbols: &HashMap<String, Symbol>,
) -> Result<Vec<Line>, io::Error> {
    let symbol = args.first().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Directive .comm requires a symbol name: '{}'", line.raw_content),
        )
    })?;

    let size = parse_i32_arg(
        args.get(1),
        format!("Directive .comm requires an integer size: '{}'", line.raw_content),
    )?;
    if size < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Directive .comm size must be non-negative: '{}'", line.raw_content),
        ));
    }

    let align_pow2 = if let Some(align_arg) = args.get(2) {
        let comm_align = parse_i32_arg(
            Some(align_arg),
            format!(
                "Directive .comm alignment must be an integer: '{}'",
                line.raw_content
            ),
        )?;
        Some(parse_comm_align_pow2(comm_align, &line.raw_content)?)
    } else {
        None
    };

    let ident = line.ident.clone();
    let mut new_lines = vec![Line {
        body: Some(Body::Directive {
            name: ".data".to_owned(),
            args: vec![],
        }),
        ..line
    }];

    let is_local = symbols.get(symbol).map(|s| s.is_local).unwrap_or(false);
    if !is_local {
        new_lines.push(Line {
            ident: ident.clone(),
            body: Some(Body::Directive {
                name: ".globl".to_owned(),
                args: vec![symbol.clone()],
            }),
            ..Line::default()
        });
    }

    if let Some(pow2) = align_pow2 {
        new_lines.push(Line {
            ident: ident.clone(),
            body: Some(Body::Directive {
                name: ".align".to_owned(),
                args: vec![pow2.to_string()],
            }),
            ..Line::default()
        });
    }

    new_lines.push(Line {
        ident: ident.clone(),
        label: Some(symbol.clone()),
        ..Line::default()
    });

    new_lines.extend(emit_zero_fill_lines(&ident, size));

    Ok(new_lines)
}

pub(crate) fn transform_directives(
    line: Line,
    supported_directives: &[String],
    symbols: &HashMap<String, Symbol>,
) -> Result<Vec<Line>, io::Error> {
    let Some(body) = line.body.as_ref() else {
        return Ok(vec![line]);
    };
    if let Body::Instr { .. } = body {
        let (name, _) = body.data_ref();
        if !is_supported_instruction(name) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Clang emitted unsupported instruction '{}' in line: '{}'",
                    name, line.raw_content
                ),
            ));
        }
        return Ok(vec![line]);
    }

    let (name, args) = body.data_ref();
    let name = name.to_owned();
    let args = args.to_vec();

    if supported_directives.iter().any(|d| d == &name) {
        return Ok(vec![line]);
    }

    match name.as_str() {
        ".comm" => handle_comm(line, &args, symbols),
        ".bss" => Ok(handle_bss(line, &args)),
        ".section" => handle_section(line, &args),
        ".zero" => handle_zero(line, &args),
        ".p2align" => handle_p2align(line, &args),
        _ => Ok(vec![]),
    }
}

#[cfg(test)]
mod tests {
    use super::transform_directives;
    use crate::asm::line::{Body, Line};
    use crate::asm::symbol::Symbol;
    use std::collections::HashMap;

    fn supported_directives() -> Vec<String> {
        vec![
            ".ascii".to_owned(),
            ".asciz".to_owned(),
            ".byte".to_owned(),
            ".data".to_owned(),
            ".double".to_owned(),
            ".end_macro".to_owned(),
            ".eqv".to_owned(),
            ".extern".to_owned(),
            ".float".to_owned(),
            ".globl".to_owned(),
            ".half".to_owned(),
            ".include".to_owned(),
            ".macro".to_owned(),
            ".space".to_owned(),
            ".string".to_owned(),
            ".text".to_owned(),
            ".word".to_owned(),
        ]
    }

    fn directive_parts(line: &Line) -> (&str, &[String]) {
        let body = line.body.as_ref().expect("line should have a body");
        match body {
            Body::Directive { name, args } => (name.as_str(), args.as_slice()),
            Body::Instr { .. } => panic!("expected directive body"),
        }
    }

    #[test]
    fn section_rodata_is_moved_to_data() {
        let symbols = HashMap::<String, Symbol>::new();
        let input = Line::new("\t.section .rodata.str1.1, \"aMS\", @progbits, 1".to_owned());

        let out = transform_directives(input, &supported_directives(), &symbols)
            .expect(".section rewrite should succeed");

        assert_eq!(out.len(), 1);
        let (name, args) = directive_parts(&out[0]);
        assert_eq!(name, ".data");
        assert!(args.is_empty());
    }

    #[test]
    fn comm_transforms_into_data_symbol_and_zero_fill() {
        let symbols = HashMap::<String, Symbol>::new();
        let input = Line::new("\t.comm global_buf, 6, 4".to_owned());

        let out = transform_directives(input, &supported_directives(), &symbols)
            .expect(".comm rewrite should succeed");

        assert_eq!(out.len(), 6);
        assert_eq!(directive_parts(&out[0]).0, ".data");

        let (globl_name, globl_args) = directive_parts(&out[1]);
        assert_eq!(globl_name, ".globl");
        assert_eq!(globl_args, ["global_buf"]);

        let (align_name, align_args) = directive_parts(&out[2]);
        assert_eq!(align_name, ".align");
        assert_eq!(align_args, ["2"]);

        assert_eq!(out[3].label.as_deref(), Some("global_buf"));

        let (fill0_name, fill0_args) = directive_parts(&out[4]);
        assert_eq!(fill0_name, ".word");
        assert_eq!(fill0_args, ["0"]);

        let (fill1_name, fill1_args) = directive_parts(&out[5]);
        assert_eq!(fill1_name, ".half");
        assert_eq!(fill1_args, ["0"]);
    }

    #[test]
    fn p2align_is_rewritten_to_align() {
        let symbols = HashMap::<String, Symbol>::new();
        let input = Line::new("\t.p2align 3".to_owned());

        let out = transform_directives(input, &supported_directives(), &symbols)
            .expect(".p2align rewrite should succeed");

        assert_eq!(out.len(), 1);
        let (name, args) = directive_parts(&out[0]);
        assert_eq!(name, ".align");
        assert_eq!(args, ["3"]);
    }
}
