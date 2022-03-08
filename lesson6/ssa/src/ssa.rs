use std::collections::HashMap;

use bril_rs::AbstractCode;
use bril_rs::AbstractInstruction;
use bril_rs::AbstractType;
use bril_utils::cfg::*;
use bril_utils::dominators::*;
use bril_utils::form_blocks::*;
use bril_rs::AbstractFunction;

fn get_def_blocks(cfg : &Cfg) -> HashMap<String, HashMap<i32, Option<AbstractType>>> {
    let mut defs = HashMap::new();
    for (block_num, block) in &cfg.block_map {
        for instr in &block.instrs {
            if let AbstractCode::Instruction(instr) = instr {
                match instr {
                    AbstractInstruction::Value {dest, op_type, ..} 
                    | AbstractInstruction::Constant {dest, const_type: op_type, ..} => {
                        let op_type = op_type.clone().unwrap();
                        if !defs.contains_key(dest) {
                            defs.insert(dest.to_string(), HashMap::new());
                        }
                        let blocks = defs.get_mut(dest).unwrap();
                        blocks.insert(*block_num, Some(op_type));
                    },
                    _ => (),
                }
            }
        }
    }
    defs
}

#[derive(Eq)]
struct Phi {
    dest : String,
    vars : Vec<String>,
    labels : Vec<i32>,
}

impl Phi {
    fn new(dest : String) -> Self {
        Phi {dest, vars : vec![], labels : vec![]}
    }
}

impl PartialEq for Phi {
    fn eq(&self, other: &Self) -> bool {
        self.dest == other.dest && self.vars == other.vars && self.labels == other.labels
    }
}

fn get_phi_nodes(cfg : &mut Cfg) -> HashMap<i32, Vec<Phi>>{
    let mut defs = get_def_blocks(cfg);
    let frontier = get_dominance_frontier_num(cfg);

    let mut phi_nodes : HashMap<i32, Vec<Phi>> = HashMap::new();
    for (v, blocks) in defs.iter_mut() {
        for (d, op_type) in &blocks.clone() {
            for block in frontier.get(d).unwrap() {
                if !phi_nodes.contains_key(block) {
                    phi_nodes.insert(*block, vec![]);
                }

                let block_phi_nodes = phi_nodes.get(block).unwrap();

                if !block_phi_nodes.contains(&Phi::new(v.to_string())) {
                    let vec = phi_nodes.get_mut(block).unwrap();
                    vec.push(Phi::new(v.to_string()));
                    blocks.insert(*block, op_type.clone());
                    let phi = AbstractCode::Instruction(AbstractInstruction::Value {
                        dest: v.to_string(), args: vec![], labels: vec![], 
                        op_type: op_type.clone(), funcs: vec![], op: "phi".to_string()});
                    cfg.block_map.get_mut(block).unwrap().instrs.insert(0, phi);
                }
            }
        }
    }
    phi_nodes
}

fn fresh_name(curr_name: &String, i : &mut i32) -> String {
    let new_name = format!("{curr_name}.{i}");
    *i += 1;
    new_name
}

fn rename(cfg : &mut Cfg, block_num : i32, phis : &mut HashMap<i32, Vec<Phi>>,
    dom_tree : &DomTree, stacks : &mut HashMap<String, Vec<String>>, i : &mut i32) {
    let block = cfg.block_map.get_mut(&block_num).unwrap();
    
    for code in &mut block.instrs {
        if let AbstractCode::Instruction(instr) = code {
            match instr {
                AbstractInstruction::Value {args, ..}
                | AbstractInstruction::Effect {args, ..} => {
                    for a in args {
                        println!("{a}");
                        let new_name = stacks.get(a).unwrap().last().unwrap();
                        *a = new_name.to_string();
                    }           
                },
                _ => (),
            }

            match instr {
                AbstractInstruction::Value {dest, ..}
                | AbstractInstruction::Constant {dest, ..} => {
                    let name = fresh_name(dest, i);
                    stacks.get_mut(dest).unwrap().push(name.clone());
                    *dest = name;
                },
                _ => (),
            }
        }
    }

    for s in cfg.succ.get(&block_num).unwrap() {
        if let Some(phis) = phis.get_mut(s) {
            for p in phis.iter_mut() {
                let v = stacks.get(&p.dest).unwrap().last().unwrap();
                p.labels.push(*s);
                p.vars.push(v.to_string());
            }
        }
    }

    for b in &dom_tree.get_node(&block_num).unwrap().children {
        rename(cfg, *b as i32, phis, dom_tree, stacks, i);
    }
}

fn rename_vars(cfg : &mut Cfg, mut phis : HashMap<i32, Vec<Phi>>) {
    let (entry, _) = cfg.block_map.first().unwrap();
    let entry = *entry;
    let mut stack = HashMap::new();
    let defs = get_def_blocks(cfg);
    let dom_tree = form_dom_tree(cfg);
    
    for v in defs.keys() {
        println!("{v}");
        stack.insert(v.to_string(), vec![v.to_string()]);
    }
    
    let mut i : i32 = 0;
    rename(cfg, entry, &mut phis, &dom_tree, &mut stack, &mut i);
}

pub fn to_ssa(func : &mut AbstractFunction) {
    let blocks = form_blocks(func);
    let mut cfg = form_cfg(blocks);
    let phi_nodes = get_phi_nodes(&mut cfg);
    rename_vars(&mut cfg, phi_nodes);
    func.instrs = reassemble(cfg);
}

pub fn from_ssa(func : &mut AbstractFunction) {
    todo!()
}
