use crate::form_blocks::{Block, flatten_blocks};
use bril_rs::{AbstractCode, AbstractInstruction};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::iter::Iterator;

#[derive(Default, Clone)]
pub struct Cfg {
    pub succ : HashMap<String, Vec<String>>,
    pub pred : HashMap<String, Vec<String>>,
    pub block_map : IndexMap<String, Block>,
}

fn fresh<'a>(seed : String, mut names : impl Iterator<Item=&'a String>) -> String {
    let mut i : i32 = 1;
    loop {
        let name = seed.to_string() + &i.to_string();
        let contains = names.find(|a| a.to_string() == name);
        if let None = contains {
            break name;
        }
        i += 1;   
    }
}

fn form_block_map(blocks : Vec<Block>) -> IndexMap<String, Block> {
    let mut block_map = IndexMap::new();
    for mut block in blocks {
        let name;
        if let AbstractCode::Label {label, ..} = &block.instrs[0] {
            name = label.to_string();
            block.instrs.remove(0);
        } else {
            name = fresh("b".to_string(), block_map.keys());
        }
        block_map.insert(name, block);
    }
    block_map
}

fn add_terminators(mut block_map : IndexMap<String, Block>) -> IndexMap<String, Block> {
    for i in 0..block_map.len() {
        let (_, block) = block_map.get_index(i).unwrap();
        let last = block_map.len() - 1;
        let mut instr = None;
        if block.is_empty() {
            if i == last {
                instr = Some(AbstractInstruction::Effect {
                    op : "ret".to_string(), 
                    args : vec![], 
                    funcs : vec![],
                    labels : vec![]});
            } else {
                let (dest, _) = block_map.get_index(i + 1).unwrap();
                let dest = dest.to_string();
                instr = Some(AbstractInstruction::Effect {
                    op : "jmp".to_string(), 
                    args : vec![], 
                    funcs : vec![],
                    labels : vec![dest.to_string()]});
            }
        } else if let AbstractCode::Instruction(last_instr) = block.instrs.last().unwrap() {
            if i == last {
                instr = Some(AbstractInstruction::Effect {
                    op : "ret".to_string(), 
                    args : vec![], 
                    funcs : vec![],
                    labels : vec![]});
            } else if let AbstractInstruction::Effect {op, ..}
                | AbstractInstruction::Value {op, ..} = last_instr {
                let (dest, _) = block_map.get_index(i + 1).unwrap();
                let dest = dest.to_string();
                match op.as_str()  {
                    "br" | "jmp" | "ret" => (),
                    _ => {
                        instr = Some(AbstractInstruction::Effect {
                            op : "jmp".to_string(), 
                            args : vec![], 
                            funcs : vec![],
                            labels : vec![dest.to_string()]});
                    }
                }
            } else if let AbstractInstruction::Constant {..} = last_instr {
                let (dest, _) = block_map.get_index(i + 1).unwrap();
                let dest = dest.to_string();
                instr = Some(AbstractInstruction::Effect {
                    op : "jmp".to_string(), 
                    args : vec![], 
                    funcs : vec![],
                    labels : vec![dest.to_string()]});
            }
        }

        let (_, block) = block_map.get_index_mut(i).unwrap();
        if let Some(i) = instr {
            block.instrs.push(AbstractCode::Instruction(i));
        }
    }
    block_map
}

fn term_sucessors(instr : &AbstractCode) -> Option<&Vec<String>>{
    match instr {
        AbstractCode::Instruction(instr) => {
            match instr {
                AbstractInstruction::Effect {op, labels, ..}=> {
                    match op.as_str() {
                        "jmp" | "br" => Some(labels),
                        "ret" => None,
                        _ => panic!("Not a terminator"),
                    }
                }
                _ => panic!("Not a terminator")
            }
        }
        _ => panic!("Not a terminator")
    }
}

fn add_edges(block_map : IndexMap<String, Block>) -> Cfg {
    let mut cfg = Cfg::default();
    for name in block_map.keys() {
        cfg.pred.insert(name.to_string(), Vec::new());
        cfg.succ.insert(name.to_string(), Vec::new());
    }
    for (name, block) in &block_map {
        if let Some(successors) = term_sucessors(block.instrs.last().unwrap()) {
            for succ in successors {
                cfg.succ.get_mut(name).unwrap().push(succ.to_string());
                cfg.pred.get_mut(succ).unwrap().push(name.to_string());
            }
        }
    }
    cfg.block_map = block_map;
    cfg
}

fn add_entry(mut cfg : Cfg) -> Cfg {
    let (first_label, _) = cfg.block_map.get_index(0).unwrap();
    let blocks : Vec<Block> = cfg.block_map.clone().into_values().collect();

    let mut has_in_edge = false;

    for instr in flatten_blocks(blocks) {
        if let AbstractCode::Instruction(instr) = instr {
            match instr {
                AbstractInstruction::Effect {labels, ..}
                    if labels.contains(first_label) => has_in_edge = true,
                _ => (),
            }
        }
    }
    if has_in_edge {
        let new_label = fresh("entry".to_string(), cfg.block_map.keys());
        let old_map : IndexMap<String, Block> = cfg.block_map;
        cfg.block_map = IndexMap::new();
        cfg.block_map.insert(new_label, Block::default());
        cfg.block_map.extend(old_map);
    }
    cfg
}

pub fn form_cfg(blocks : Vec<Block>) -> Cfg {
    let block_map = add_terminators(form_block_map(blocks));
    let cfg = add_edges(block_map);
    add_entry(cfg)
}

pub fn reassemble(cfg : Cfg) -> Vec<AbstractCode> {
    let mut instrs = Vec::new();
    for (name, block) in cfg.block_map {
        instrs.push(AbstractCode::Label {label : name});
        instrs.extend(block.instrs);
    }
    instrs
}
