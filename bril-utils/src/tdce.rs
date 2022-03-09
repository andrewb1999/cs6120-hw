use crate::form_blocks::*;
use std::collections::{HashSet, HashMap};
use bril_rs::*;

pub fn trivial_global_dce_pass(func : &mut AbstractFunction) -> bool {
    let mut blocks = form_blocks(&func);
    let mut used = HashSet::new();
    for block in &blocks {
        for code in &block.instrs {
            match code {
                AbstractCode::Instruction(instr) =>
                    match instr {
                        AbstractInstruction::Value {args, ..} => used.extend(args.clone()),
                        AbstractInstruction::Effect {args, ..} => used.extend(args.clone()),
                        _ => (),
                    },
                _ => (),
            }
        }
    }

    let mut not_done = false;
    
    for block in &mut blocks {
        let mut new_block = Block { instrs: Vec::new() };
        for code in &block.instrs {
            match code {
                AbstractCode::Instruction(instr) =>
                    match instr {
                        AbstractInstruction::Constant {dest, ..} | AbstractInstruction::Value {dest, ..} => {
                            if !used.contains(dest) {
                                not_done = true;
                            } else {
                                new_block.instrs.push(code.clone())
                            }
                        },
                        AbstractInstruction::Effect {..} => new_block.instrs.push(code.clone()),
                    }
                AbstractCode::Label {..} => new_block.instrs.push(code.clone()),
            }
        }
        *block = new_block;
    }
    func.instrs = flatten_blocks(blocks);
    not_done
}

pub fn locally_killed_instrs_pass(func : &mut AbstractFunction) -> bool {
    let mut blocks = form_blocks(&func);
    let mut not_done = false;
    for block in &mut blocks {
        let mut to_remove : Vec<usize> = Vec::new();
        let mut last_def = HashMap::new();
        for i in 0..block.instrs.len() {
            let code = &block.instrs[i];
            match code {
                AbstractCode::Instruction(instr) => {
                    match instr {
                        AbstractInstruction::Value {args, ..} | AbstractInstruction::Effect {args, ..} => {
                            for a in args {
                                last_def.remove(&a);
                            }
                        },
                        _ => (),
                    }
                    match instr {
                        AbstractInstruction::Value {dest, ..} | AbstractInstruction::Constant {dest, ..} => {
                            if last_def.contains_key(&dest) {
                                to_remove.push(*last_def.get(&dest).unwrap());
                                not_done = true;
                            }
                            last_def.insert(dest, i);
                        },
                        _ => (),
                    }
                }
                _ => (),
            }
        }
        for i in to_remove {
            block.instrs.remove(i);
        }
    }
    func.instrs = flatten_blocks(blocks);
    not_done
}

pub fn trivial_dce(func : &mut AbstractFunction) {
    while trivial_global_dce_pass(func) || locally_killed_instrs_pass(func) {}
}
