use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::iter;

#[derive(Debug)]
pub(crate) enum Body {
    Directive { name: String, args: Vec<String> },
    Instr { name: String, args: Vec<String> },
}

impl Body {
    pub(crate) fn data_ref(&self) -> (&str, &[String]) {
        match self {
            Body::Directive { name, args } => (name, args),
            Body::Instr { name, args } => (name, args),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Line {
    pub(crate) label: Option<String>,
    pub(crate) body: Option<Body>,
    pub(crate) comment: Option<String>,
    pub(crate) raw_content: String,
    pub(crate) ident: String,
}

impl Line {
    pub(crate) fn new(from: String) -> Line {
        let mut line = Line {
            label: None,
            body: None,
            comment: None,
            raw_content: from.clone(),
            ident: from.chars().take_while(|x| x.is_whitespace()).collect(),
        };

        let mut args = from.split_whitespace();
        let mut curr_arg = args.next();

        if let Some(arg_content) = curr_arg {
            if arg_content.ends_with(':') {
                line.label = Some(
                    arg_content
                        .strip_suffix(':')
                        .expect("Line ends with :")
                        .to_string(),
                );
                curr_arg = args.next();
            }
        }

        if let Some(mut arg_content) = curr_arg {
            if arg_content.starts_with('#') {
                arg_content = arg_content
                    .strip_prefix('#')
                    .expect("Content starts with #");
                line.comment = Some(iter::once(arg_content).chain(args).collect::<Vec<_>>().join(" "));
                return line;
            }

            let rest: Vec<String> = args
                .by_ref()
                .take_while(|x| !x.starts_with('#'))
                .map(|x| x.strip_suffix(',').unwrap_or(x).to_string())
                .collect();
            if arg_content.starts_with('.') {
                line.body = Some(Body::Directive {
                    name: arg_content.to_string(),
                    args: rest,
                })
            } else {
                line.body = Some(Body::Instr {
                    name: arg_content.to_string(),
                    args: rest,
                })
            }

            line.comment = Some(args.collect::<Vec<_>>().join(" ")).filter(|s| !s.is_empty());
        }

        line
    }

    pub(crate) fn emit(&self, writer: &mut BufWriter<File>) -> io::Result<()> {
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
}
