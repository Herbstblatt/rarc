use std::collections::HashSet;
use std::sync::LazyLock;

pub const SUPPORTED_INSTRUCTIONS: &[&str] = &[
    "add", "addi", "and", "andi", "auipc", "b", "beq", "beqz", "bge", "bgeu", "bgez", "bgt", "bgtu",
    "bgtz", "ble", "bleu", "blez", "blt", "bltu", "bltz", "bne", "bnez", "call", "csrc", "csrci",
    "csrr", "csrrc", "csrrci", "csrrs", "csrrsi", "csrrw", "csrrwi", "csrs", "csrsi", "csrw",
    "csrwi", "div", "divu", "ebreak", "ecall", "fabs.d", "fabs.s", "fadd.d", "fadd.s", "fclass.d",
    "fclass.s", "fcvt.d.s", "fcvt.d.w", "fcvt.d.wu", "fcvt.s.d", "fcvt.s.w", "fcvt.s.wu",
    "fcvt.w.d", "fcvt.w.s", "fcvt.wu.d", "fcvt.wu.s", "fdiv.d", "fdiv.s", "fence", "fence.i",
    "feq.d", "feq.s", "fge.d", "fge.s", "fgt.d", "fgt.s", "fld", "fle.d", "fle.s", "flt.d", "flt.s",
    "flw", "flwd", "fmadd.d", "fmadd.s", "fmax.d", "fmax.s", "fmin.d", "fmin.s", "fmsub.d",
    "fmsub.s", "fmul.d", "fmul.s", "fmv.d", "fmv.s", "fmv.s.x", "fmv.w.x", "fmv.x.s", "fmv.x.w",
    "fneg.d", "fneg.s", "fnmadd.d", "fnmadd.s", "fnmsub.d", "fnmsub.s", "frcsr", "frflags", "frrm",
    "frsr", "fscsr", "fsd", "fsflags", "fsflagsi", "fsgnj.d", "fsgnj.s", "fsgnjn.d", "fsgnjn.s",
    "fsgnjx.d", "fsgnjx.s", "fsqrt.d", "fsqrt.s", "fsrm", "fsrmi", "fssr", "fsub.d", "fsub.s",
    "fsw", "j", "jal", "jalr", "jr", "la", "lb", "lbu", "lh", "lhu", "li", "lui", "lw", "mul",
    "mulh", "mulhsu", "mulhu", "mv", "neg", "nop", "not", "or", "ori", "rdcycle", "rdcycleh",
    "rdinstret", "rdinstreth", "rdtime", "rdtimeh", "rem", "remu", "ret", "sb", "seqz", "sgt",
    "sgtu", "sgtz", "sh", "sll", "slli", "slt", "slti", "sltiu", "sltu", "sltz", "snez", "sra",
    "srai", "srl", "srli", "sub", "sw", "tail", "uret", "wfi", "xor", "xori",
];

static SUPPORTED_INSTRUCTION_SET: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| SUPPORTED_INSTRUCTIONS.iter().copied().collect());

pub fn is_supported_instruction(name: &str) -> bool {
    SUPPORTED_INSTRUCTION_SET.contains(name)
}
