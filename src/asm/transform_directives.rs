use std::iter;
use std::io;

use crate::asm::line::{Body, Line};

pub(crate) fn transform_directives(
    line: Line,
    supported_directives: &[String],
) -> Result<Vec<Line>, io::Error> {
    let Some(body) = line.body.as_ref() else {
        return Ok(vec![line]);
    };
    if let Body::Instr { .. } = body {
        return Ok(vec![line]);
    }

    let (name, args) = body.data_ref();
    if supported_directives.iter().any(|d| d == name) {
        return Ok(vec![line]);
    }

    match name {
        ".bss" => Ok(vec![Line {
                body: Some(Body::Directive {
                    name: ".data".to_owned(),
                    args: args.to_vec(),
                }),
                ..line
            }]),
        ".section" => {
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
        ".zero" => {
            let mut arg: i32 = args
                .first()
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Directive .zero requires an integer argument: '{}'", line.raw_content),
                    )
                })?
                .parse()
                .map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Directive .zero argument must be an integer: '{}'", line.raw_content),
                    )
                })?;

            let ident = line.ident.clone();
            let mut new_lines: Vec<Line> = Vec::new();
            let mut extend_iter = iter::once(line).chain(iter::repeat_with(Line::default));

            for _ in 0..(arg / 4) {
                new_lines.push(Line {
                    body: Some(Body::Directive {
                        name: ".word".to_owned(),
                        args: vec!["0".to_owned()],
                    }),
                    ident: ident.clone(),
                    ..extend_iter.next().expect("infinite iterator")
                });
            }
            arg %= 4;

            for _ in 0..(arg / 2) {
                new_lines.push(Line {
                    body: Some(Body::Directive {
                        name: ".half".to_owned(),
                        args: vec!["0".to_owned()],
                    }),
                    ident: ident.clone(),
                    ..extend_iter.next().expect("infinite iterator")
                });
            }
            arg %= 2;

            for _ in 0..arg {
                new_lines.push(Line {
                    body: Some(Body::Directive {
                        name: ".half".to_owned(),
                        args: vec!["0".to_owned()],
                    }),
                    ident: ident.clone(),
                    ..extend_iter.next().expect("infinite iterator")
                });
            }

            Ok(new_lines)
        }
        ".p2align" => {
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
        _ => Ok(vec![]),
    }
}
