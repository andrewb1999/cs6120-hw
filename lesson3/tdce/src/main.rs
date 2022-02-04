pub mod form_blocks;
pub use form_blocks::*;

use std::collections::HashSet;
use bril_rs::*;

fn trivial_global_dce_pass(func : &mut AbstractFunction) -> bool {
    let mut blocks = form_blocks(&func);
    let mut used = HashSet::new();
    for block in &blocks {
        for code in &block.instrs {
            match code {
                AbstractCode::Instruction(instr) =>
                    match instr {
                        AbstractInstruction::Value {args, ..} => used.extend(args),
                        AbstractInstruction::Effect {args, ..} => used.extend(args),
                        _ => (),
                    },
                _ => (),
            }
        }
    }

    let mut done = false;
    
    for i in 0..blocks.len() {
        let block = &blocks[i];
        let mut new_block = Block { instrs: Vec::new() };
        for code in &block.instrs {
            match code {
                AbstractCode::Instruction(instr) =>
                    match instr {
                        AbstractInstruction::Constant {dest, ..} | AbstractInstruction::Value {dest, ..} => {
                            if !used.contains(dest) {
                                done = true
                            } else {
                                new_block.instrs.push(code)
                            }
                        },
                        AbstractInstruction::Effect {..} => new_block.instrs.push(code),
                    }
                AbstractCode::Label {..} => new_block.instrs.push(code),
            }
        }
        blocks[i] = new_block;
    }
    func.instrs = flatten_blocks(blocks);
    done
}

fn trivial_dce(func : &mut AbstractFunction) {
    while trivial_global_dce_pass(func) {}
}

fn main() {
    let mut program = load_abstract_program();
    for func in &mut program.functions {
        trivial_dce(func);
    }
    output_abstract_program(&program)
}
