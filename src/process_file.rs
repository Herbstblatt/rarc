use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::iter;

#[derive(Debug)]
enum Body {
    Directive {
        name: String,
        args: Vec<String>
    },
    Instr {
        name: String,
        args: Vec<String>    
    }
}

impl Body {
    fn data_ref(&self) -> (&str, &[String]) {
        match self {
            Body::Directive { name, args } => (name, args),
            Body::Instr { name, args } => (name, args)
        }
    }
}

#[derive(Debug, Default)]
struct Line {
    pub label: Option<String>,
    pub body: Option<Body>,
    pub comment: Option<String>,
    raw_content: String,
    ident: String
}

impl Line {
    pub fn new(from: String) -> Line {
        let mut line = Line {
            label: None,
            body: None,
            comment: None,
            raw_content: from.clone(),
            ident: from.chars().take_while(|x| x.is_whitespace()).collect()
        };

        let mut args = from.split_whitespace();
        let mut curr_arg = args.next();

        if let Some(arg_content) = curr_arg {
            if arg_content.ends_with(':') {
                line.label = Some(
                    arg_content.strip_suffix(':').expect("Line ends with :").to_string()
                );
                curr_arg = args.next();
            }
        }

        if let Some(mut arg_content) = curr_arg {
            if arg_content.starts_with('#') {
                arg_content = arg_content.strip_prefix('#').expect("Content starts with #");
                line.comment = Some(
                    iter::once(arg_content)
                         .chain(args)
                         .collect::<Vec<_>>()
                         .join(" ")
                );
                return line
            }

            let rest: Vec<String> = args.by_ref()
                                        .take_while(|x| !x.starts_with('#'))
                                        .map(|x| x.strip_suffix(',').unwrap_or(x).to_string())
                                        .collect();
            if arg_content.starts_with('.') {
                line.body = Some(
                    Body::Directive { name: arg_content.to_string(), args: rest }
                )
            } else {
                line.body = Some(
                    Body::Instr { name: arg_content.to_string(), args: rest }
                )
            }

            line.comment = Some(
                args.collect::<Vec<_>>().join(" ")
            ).filter(|s| !s.is_empty());
        }

        line
    }

    pub fn emit(&self, writer: &mut BufWriter<File>) -> io::Result<()> {
        writer.write_all(self.ident.as_bytes())?;

        let mut tokens: Vec<String> = Vec::new();

        if let Some(label) = &self.label {
            tokens.push(label.clone() + ":");
        }

        if let Some(body) = &self.body {
            let (name, args) = body.data_ref();
            tokens.push(name.to_string());
            if !args.is_empty() {
                tokens.push(args.join(", "));
            }
        }

        if let Some(comment) = &self.comment {
            tokens.push("#".to_owned());
            tokens.push(comment.clone());
        }

        let rendered = tokens.join(" ");

        writer.write_all(rendered.as_bytes())?;
        writer.write_all(b"\n")?;
        Ok(())
    }

    pub fn process(self, supported_directives: &[String]) -> Vec<Line> {
        let Some(body) = self.body.as_ref() else {
            return vec![self];
        };
        if let Body::Instr { .. } = body {
            return vec![self]
        }

        let (name, args) = body.data_ref();
        if supported_directives.iter().any(|d| d == name) {
            return vec![self];
        }

        match name {
            ".bss" => {
                return vec![Line {
                    body: Some(
                        Body::Directive { name: ".data".to_owned(), args: args.to_vec() }
                    ),
                    ..self
                }];
            },
            ".section" => {
                let section_name = args.get(0).expect("Section has a name");
                if section_name.starts_with(".rodata") {
                    return vec![Line {
                        body: Some(
                            Body::Directive { name: ".data".to_owned(), args: vec![] }
                        ),
                        ..self
                    }];
                }
            },
            ".zero" => {
                let mut arg: i32 = args.get(0)
                                .expect("Should have an argument")
                                .parse()
                                .expect("Argument should be an int");

                let mut new_lines: Vec<Line> = Vec::new();
                let mut extend_iter = iter::once(self).chain(
                    iter::repeat_with(|| Line::default())
                );

                for _ in 0..(arg/4) {
                    new_lines.push(Line {
                        body: Some(
                            Body::Directive { name: ".word".to_owned(), args: vec!["0".to_owned()] }
                        ),
                        ..extend_iter.next().unwrap()
                    });
                }
                arg = arg % 4;
                
                for _ in 0..(arg/2) {
                    new_lines.push(Line {
                        body: Some(
                            Body::Directive { name: ".half".to_owned(), args: vec!["0".to_owned()] }
                        ),
                        ..extend_iter.next().unwrap()
                    });
                }
                arg = arg % 2;

                for _ in 0..arg {
                    new_lines.push(Line {
                        body: Some(
                            Body::Directive { name: ".half".to_owned(), args: vec!["0".to_owned()] }
                        ),
                        ..extend_iter.next().unwrap()
                    });
                }
            },
            ".p2align" => {
                let arg: i32 = args.get(0)
                                .expect("Should have an argument")
                                .parse()
                                .expect("Argument should be an int");
                if arg > 3 {
                    return vec![];
                }
                return vec![Line {
                    body: Some(
                        Body::Directive { name: ".align".to_owned(), args: vec![arg.to_string()] }
                    ),
                    ..self
                }];
            },
            _ => {}
        }

        return vec![];
    }
}

pub fn process_file(
    reader: BufReader<File>,
    mut writer: BufWriter<File>,
    supported_directives: &[String],
) -> Result<(), Box<dyn Error>> {
    for line in reader.lines() {
        let line = Line::new(line?);
        dbg!(&line);
        for res_line in line.process(supported_directives) {
            res_line.emit(&mut writer)?;
        }
    }
    writer.flush()?;
    Ok(())
}