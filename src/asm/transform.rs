use std::iter;

use crate::asm::line::{Body, Line};

pub(crate) fn process_line(line: Line, supported_directives: &[String]) -> Vec<Line> {
    let Some(body) = line.body.as_ref() else {
        return vec![line];
    };
    if let Body::Instr { .. } = body {
        return vec![line];
    }

    let (name, args) = body.data_ref();
    if supported_directives.iter().any(|d| d == name) {
        return vec![line];
    }

    match name {
        ".bss" => {
            vec![Line {
                body: Some(Body::Directive {
                    name: ".data".to_owned(),
                    args: args.to_vec(),
                }),
                ..line
            }]
        }
        ".section" => {
            let section_name = args.first().expect("Section has a name");
            if section_name.starts_with(".rodata") {
                return vec![Line {
                    body: Some(Body::Directive {
                        name: ".data".to_owned(),
                        args: vec![],
                    }),
                    ..line
                }];
            }
            vec![]
        }
        ".zero" => {
            let mut arg: i32 = args
                .first()
                .expect("Should have an argument")
                .parse()
                .expect("Argument should be an int");

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

            new_lines
        }
        ".p2align" => {
            let arg: i32 = args
                .first()
                .expect("Should have an argument")
                .parse()
                .expect("Argument should be an int");
            if arg > 3 {
                return vec![];
            }
            vec![Line {
                body: Some(Body::Directive {
                    name: ".align".to_owned(),
                    args: vec![arg.to_string()],
                }),
                ..line
            }]
        }
        _ => vec![],
    }
}
