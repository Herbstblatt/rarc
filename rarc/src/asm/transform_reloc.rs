use crate::asm::line::{Line, Body};

fn parse_hi_reloc(arg: &str) -> Option<&str> {
    arg.strip_prefix("%hi(")?.strip_suffix(')')
}

fn parse_lo_mem_reloc(arg: &str) -> Option<(&str, &str)> {
    let rest = arg.strip_prefix("%lo(")?;
    let split_idx = rest.find(")(")?;
    let symbol = &rest[..split_idx];
    let reg_with_suffix = &rest[(split_idx + 2)..];
    let reg = reg_with_suffix.strip_suffix(')')?;
    Some((symbol, reg))
}

pub fn normalize_hi_lo_pairs(lines: &mut [Line]) {
    for idx in 0..lines.len().saturating_sub(1) {
        let Some(curr_body) = lines[idx].body.as_ref() else {
            continue;
        };
        let (curr_name, curr_args) = curr_body.data_ref();
        if curr_name != "lui" || curr_args.len() != 2 {
            continue;
        }

        let reg = curr_args[0].clone();
        let Some(symbol) = parse_hi_reloc(&curr_args[1]).map(str::to_owned) else {
            continue;
        };

        let Some(next_body) = lines[idx + 1].body.as_mut() else {
            continue;
        };
        let (_, next_args) = next_body.data_ref_mut();

        let mut replaced = false;
        for arg in next_args.iter_mut() {
            if let Some((lo_symbol, lo_reg)) = parse_lo_mem_reloc(arg) {
                if lo_symbol == symbol && lo_reg == reg {
                    *arg = format!("0({reg})");
                    replaced = true;
                }
            }
        }

        if replaced {
            if let Some(curr_body_mut) = lines[idx].body.as_mut() {
                *curr_body_mut = Body::Instr {
                    name: "la".to_owned(),
                    args: vec![reg, symbol],
                };
            }
        }
    }
}
