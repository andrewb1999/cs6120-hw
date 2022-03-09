use std::collections::HashMap;
use bril_rs::AbstractCode;
use bril_rs::AbstractInstruction;
use bril_rs::AbstractType;
use bril_utils::cfg::*;
use bril_utils::dominators::*;
use bril_utils::form_blocks::*;
use bril_rs::AbstractFunction;
use bril_utils::tdce::*;

type Defs = HashMap<String, HashMap<i32, Option<AbstractType>>>;

fn get_def_blocks(cfg : &Cfg) -> Defs {
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

fn var_defined_multiple_times(vars : Vec<String>) -> HashMap<String, bool> {
    let mut multiple_defs : HashMap<String, i32> = HashMap::new();
    for v in vars {
        if !multiple_defs.contains_key(&v) {
            multiple_defs.insert(v, 1);
        } else {
            *multiple_defs.get_mut(&v).unwrap() += 1;
        }
    }
    multiple_defs.into_iter().map(|(v, c)| (v, c > 1)).collect()
}

#[derive(Eq, Debug)]
struct Phi {
    dest : String,
    orig_dest : String,
    vars : Vec<String>,
    labels : Vec<i32>,
    op_type : Option<AbstractType>,
}

impl Phi {
    fn new(dest : String, op_type : Option<AbstractType>) -> Self {
        Phi {dest : dest.clone(), orig_dest : dest, vars : vec![], labels : vec![], op_type}
    }
}

impl PartialEq for Phi {
    fn eq(&self, other: &Self) -> bool {
        self.dest == other.dest && self.vars == other.vars && self.labels == other.labels
    }
}

fn get_phi_nodes(cfg : &mut Cfg, args : Vec<String>) -> HashMap<i32, Vec<Phi>>{
    let mut defs = get_def_blocks(&cfg);
    let multiple_defs = var_defined_multiple_times(
        args.into_iter().chain(defs.clone().into_keys()).collect());
    let frontier = get_dominance_frontier_num(cfg);

    let mut phi_nodes : HashMap<i32, Vec<Phi>> = HashMap::new();
    for (v, blocks) in defs.iter_mut() {
        for (d, op_type) in &blocks.clone() {
            for block in frontier.get(d).unwrap() {
                // println!("{v} : {d} : {block}");
                if !phi_nodes.contains_key(block) {
                    phi_nodes.insert(*block, vec![]);
                }

                let block_phi_nodes = phi_nodes.get(block).unwrap();

                if !block_phi_nodes.contains(&Phi::new(v.to_string(), op_type.clone())) 
                    // && *multiple_defs.get(v).unwrap() 
                {
                    let vec = phi_nodes.get_mut(block).unwrap();
                    vec.push(Phi::new(v.to_string(), op_type.clone()));
                    blocks.insert(*block, op_type.clone());
                }
            }
        }
    }
    phi_nodes
}

fn insert_phi_nodes(cfg : &mut Cfg, phi_nodes : HashMap<i32, Vec<Phi>>) {
    for (block, phis) in phi_nodes {
        for phi in phis {
            let labels : Vec<_> = phi.labels.iter().map(|l|
                cfg.name_map.get_by_left(l).unwrap().to_string()).collect();

            let phi = AbstractCode::Instruction(AbstractInstruction::Value {
                dest: phi.dest.to_string(), args: phi.vars, labels, 
                op_type: phi.op_type.clone(), funcs: vec![], op: "phi".to_string()});
            cfg.block_map.get_mut(&block).unwrap().instrs.insert(0, phi);
        }
    }
}

fn fresh_name(curr_name: &String, counters : &mut HashMap<String, i32>) -> String {
    if !counters.contains_key(curr_name) {
        counters.insert(curr_name.to_string(), 0);
    }
    let i = counters.get_mut(curr_name).unwrap();
    let new_name = format!("{curr_name}.{i}");
    *i += 1;
    new_name
}

fn rename(cfg : &mut Cfg, block_num : i32, phis : &mut HashMap<i32, Vec<Phi>>,
    dom_tree : &DomTree, stacks : &mut HashMap<String, Vec<String>>, 
    counters : &mut HashMap<String, i32>) {
    let block = cfg.block_map.get_mut(&block_num).unwrap();
    
    let stacks_backup = stacks.clone();

    if phis.contains_key(&block_num) {
        for phi in phis.get_mut(&block_num).unwrap() {
            let name = fresh_name(&phi.dest, counters);
            stacks.get_mut(&phi.dest).unwrap().push(name.clone());
            phi.dest = name;
        }
    }

    for code in &mut block.instrs {
        if let AbstractCode::Instruction(instr) = code {
            match instr {
                AbstractInstruction::Value {args, ..}
                | AbstractInstruction::Effect {args, ..} => {
                    for a in args {
                        let new_name = stacks.get(a).unwrap().last().unwrap();
                        *a = new_name.to_string();
                    }           
                },
                _ => (),
            }

            match instr {
                AbstractInstruction::Value {dest, ..}
                | AbstractInstruction::Constant {dest, ..} => {
                    let name = fresh_name(dest, counters);
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
                let v = stacks.get(&p.orig_dest).unwrap().last();
                match v {
                    Some(v) => {
                        p.vars.push(v.to_string());
                    },
                    None => {
                        p.vars.push("__undefined".to_string())
                    },
                }
                p.labels.push(block_num);
            }
        }
    }

    let mut children = dom_tree.get_node(&block_num).unwrap().children.clone();
    children.sort();
    for b in children {
        rename(cfg, b as i32, phis, dom_tree, stacks, counters);
    }

    *stacks = stacks_backup;
}

fn rename_vars(cfg : &mut Cfg, args : Vec<String>, phis : &mut HashMap<i32, Vec<Phi>>) {
    let (entry, _) = cfg.block_map.first().unwrap();
    let entry = *entry;
    let mut stack = HashMap::new();
    let dom_tree = form_dom_tree(cfg);
    let defs = get_def_blocks(cfg);
    
    for v in defs.keys() {
        stack.insert(v.to_string(), vec![]);
    }

    for v in &args {
        stack.insert(v.to_string(), vec![v.to_string()]);
    }
    
    rename(cfg, entry, phis, &dom_tree, &mut stack, &mut HashMap::new());
}

pub fn to_ssa(func : &mut AbstractFunction) {
    let blocks = form_blocks(func);
    let mut cfg = form_cfg(blocks);
    let args : Vec<_> = func.args.iter().map(|a| a.name.clone()).collect();
    let mut phi_nodes = get_phi_nodes(&mut cfg, args.clone());
    rename_vars(&mut cfg, args, &mut phi_nodes);
    insert_phi_nodes(&mut cfg, phi_nodes);
    func.instrs = reassemble(cfg);
    trivial_dce(func);
}

pub fn from_ssa(func : &mut AbstractFunction) {
    let blocks = form_blocks(func);
    let mut cfg = form_cfg(blocks);

    for block in cfg.block_map.clone().values() {
        for code in &block.instrs {
            if let AbstractCode::Instruction(instr) = code {
                if let AbstractInstruction::Value {op, dest, op_type, labels, args, ..} = instr {
                    if op == "phi" {
                        for (i, label) in labels.iter().enumerate() {
                            let var = args.get(i).unwrap();

                            let block_num = cfg.name_map.get_by_right(label).unwrap();
                            let (_, pred) = cfg.block_map.get_index_mut(*block_num as usize).unwrap();
                            pred.instrs.insert(pred.instrs.len() - 1, AbstractCode::Instruction(
                                AbstractInstruction::Value {
                                args: vec![var.to_string()],
                                dest: dest.to_string(),
                                funcs: vec![],
                                labels: vec![],
                                op: "id".to_string(),
                                op_type: op_type.clone(),
                            }));
                        }
                    }
                }
            }
        }
    }

    
    for block in cfg.block_map.values_mut() {
        for (i, code) in block.instrs.clone().iter().enumerate() {
            if let AbstractCode::Instruction(instr) = code {
                if let AbstractInstruction::Value {op, ..} = instr {
                    if op == "phi" {
                        block.instrs.remove(i);
                    }
                }
            }
        }
    }

    func.instrs = reassemble(cfg);
    trivial_dce(func);
}
