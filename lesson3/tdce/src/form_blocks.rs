use bril_rs::{AbstractInstruction, AbstractFunction, AbstractCode};

#[derive(Default)]
pub struct Block<'a> {
    pub instrs: Vec<&'a AbstractCode>,
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

fn get_code(func : &AbstractFunction) -> Vec<&AbstractCode> {
    let mut code = Vec::new();
    for instr in &func.instrs {
        code.push(instr)
    }
    code
}

pub fn form_blocks(func : &AbstractFunction) -> Vec<Block>{
    let instrs = get_code(func);
    let mut blocks = Vec::new();
    let mut cur_block : Block = Default::default();
    for i in instrs {
        match i {
            AbstractCode::Instruction(instr) => {
                let term = is_terminator(&instr);
                cur_block.instrs.push(i);
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
                cur_block.instrs.push(i)
            },
        }
    }
    if !cur_block.instrs.is_empty() {
        blocks.push(cur_block)
    }
    blocks
}

pub fn print_blocks(blocks : &Vec<Block>) {
    println!("----------------------");
    for block in blocks {
        for instr in &block.instrs {
            println!("{}", instr)
        }
        println!("----------------------");
    }
}

pub fn print_func_blocks(func : &AbstractFunction) {
    println!("----------------------");
    for block in form_blocks(func) {
        for instr in block.instrs {
            println!("{}", instr)
        }
        println!("----------------------");
    }
}

pub fn flatten_blocks(blocks : Vec<Block>) -> Vec<AbstractCode> {
    let mut instrs = Vec::new();
    for block in blocks {
        for instr in block.instrs {
            instrs.push(instr.clone());
        }
    }
    instrs
}
