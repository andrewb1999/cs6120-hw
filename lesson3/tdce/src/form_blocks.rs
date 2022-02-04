use bril_rs::{AbstractInstruction, AbstractProgram, AbstractCode};

#[derive(Default)]
pub struct Block {
    pub instrs: Vec<AbstractInstruction>,
}

fn get_op(instr : &AbstractInstruction) -> String {
    match instr {
        AbstractInstruction::Constant {..} => String::from("const"),
        AbstractInstruction::Value {op, ..} => op.to_string(),
        AbstractInstruction::Effect {op, ..} => op.to_string(),
    }
}

fn is_terminator(instr : &AbstractInstruction) -> bool {
    let op = get_op(instr);
    if op == "br" || op == "jmp" || op == "ret" {
        true
    } else {
        false
    }
}

fn get_code(program : AbstractProgram) -> Vec<AbstractCode> {
    let mut code = Vec::new();
    for func in program.functions {
        for instr in func.instrs {
            code.push(instr)
        }
    }
    code
}

pub fn form_blocks<'a>(program : AbstractProgram) -> Vec<Block>{
    let instrs = get_code(program);
    let mut blocks = Vec::new();
    let mut cur_block : Block = Default::default();
    for i in instrs {
        match i {
            AbstractCode::Instruction(instr) => {
                let term = is_terminator(&instr);
                cur_block.instrs.push(instr);
                if term {
                    blocks.push(cur_block);
                    cur_block = Default::default();
                }
            },
            AbstractCode::Label {..} => {
                if !cur_block.instrs.is_empty() {
                    blocks.push(cur_block);
                    cur_block = Default::default()
                }
            },
        }
    }
    if !cur_block.instrs.is_empty() {
        blocks.push(cur_block)
    }
    blocks
}
