use bril_rs::{AbstractInstruction, AbstractCode};

use crate::form_blocks::Block;
use crate::cfg::Cfg;
use std::collections::{HashSet, HashMap};
use std::hash::Hash;

trait Dataflow {
    type Item;

    fn merge(&self, sets : impl Iterator<Item=HashSet<Self::Item>>) -> HashSet<Self::Item>;

    fn transfer(&self, b : &Block, in_b : &HashSet<Self::Item>) -> HashSet<Self::Item>;

    fn is_reverse(&self) -> bool;

    fn init(&self) -> HashSet<Self::Item>;
}

fn union<T>(sets : impl Iterator<Item=HashSet<T>>) -> HashSet<T>
    where
    T : Eq + Hash + Clone {
    sets.into_iter().fold(HashSet::new(), |mut acc, p| {acc.extend(p); acc})
}

fn df_analysis<T>(cfg : &Cfg, df : impl Dataflow<Item=T>) 
    -> (HashMap<String, HashSet<T>>, HashMap<String, HashSet<T>>)
    where T : Eq + Hash + Clone {
    let forward = !df.is_reverse();
    let mut in_map : HashMap<String, HashSet<T>> = HashMap::new();
    let mut out_map : HashMap<String, HashSet<T>> = HashMap::new();

    let pred;
    let succ;

    if forward {
        pred = &cfg.pred;
        succ = &cfg.succ;
    } else {
        succ = &cfg.pred;
        pred = &cfg.succ;
    }

    if forward {
        let (name, _) = cfg.block_map.get_index(0).unwrap();
        in_map.insert(name.to_string(), df.init());
    } else {
        let (name, _) = cfg.block_map.last().unwrap();
        in_map.insert(name.to_string(), df.init());
    }

    for (name, _) in &cfg.block_map {
        out_map.insert(name.to_string(), df.init());
    }

    let mut worklist = Vec::new();
    worklist.extend(cfg.block_map.keys());

    while !worklist.is_empty() {
        let name = worklist.pop().unwrap();
        let b = cfg.block_map.get(name).unwrap();
        let preds : &Vec<String> = pred.get(name).unwrap();
        let out_p : Vec<HashSet<T>>
            = preds.into_iter().map(|p| out_map.get(p).unwrap().clone()).collect();
        let in_b = df.merge(out_p.into_iter());
        in_map.insert(name.to_string(), in_b);
        let out_b = df.transfer(b, in_map.get(name).unwrap());
        if out_map.get(name).unwrap() != &out_b {
            worklist.extend(succ.get(name).unwrap());
        }
        out_map.insert(name.to_string(), out_b);
    }
    if forward {
        (in_map, out_map)
    } else {
        (out_map, in_map)
    }
}

struct DefinedVars;

impl DefinedVars {
    fn get_def_vars(&self, b : &Block) -> HashSet<String> {
        let mut set = HashSet::new();
        for instr in &b.instrs {
            if let AbstractCode::Instruction(instr) = instr {
                if let AbstractInstruction::Constant {dest, ..}
                | AbstractInstruction::Value {dest, ..} = instr {
                    set.insert(dest.to_string());
                }
            }
        }
        set
    }
}

impl Dataflow for DefinedVars {
    type Item = String;

    fn merge(&self, sets : impl Iterator<Item=HashSet<Self::Item>>) -> HashSet<Self::Item> {
        union(sets)
    }

    fn transfer(&self, b : &Block, in_b : &HashSet<Self::Item>) -> HashSet<Self::Item> {
        let mut set : HashSet<String> = in_b.clone();
        set.extend(self.get_def_vars(b));
        set
    }

    fn is_reverse(&self) -> bool {
        false
    }

    fn init(&self) -> HashSet<Self::Item> {
        HashSet::new()
    }
}

pub fn declared_vars(cfg : &Cfg) {
    let (in_map, out_map) = df_analysis(cfg, DefinedVars);
    for name in cfg.block_map.keys() {
        let mut ins : Vec<&String> = in_map.get(name).unwrap().into_iter().collect();
        let mut outs : Vec<&String> = out_map.get(name).unwrap().into_iter().collect();
        ins.sort();
        outs.sort();
        println!("{name}:");
        println!("    in: {ins:?}");
        println!("    out: {outs:?}");
    }
}

struct LiveVars;

impl LiveVars {
    fn get_def_vars(&self, b : &Block) -> HashSet<String> {
        let mut set = HashSet::new();
        for instr in &b.instrs {
            if let AbstractCode::Instruction(instr) = instr {
                if let AbstractInstruction::Constant {dest, ..}
                | AbstractInstruction::Value {dest, ..} = instr {
                    set.insert(dest.to_string());
                }
            }
        }
        set
    }

    fn get_uses(&self, b : &Block) -> HashSet<String> {
        let mut defined = HashSet::new();
        let mut used = HashSet::new();
        for instr in &b.instrs{
            if let AbstractCode::Instruction(instr) = instr {
                if let AbstractInstruction::Value {args, ..}
                | AbstractInstruction::Effect {args, ..} = instr {
                    for a in args {
                        if !defined.contains(a) {
                            used.insert(a.to_string());
                        }
                    }
                }
                if let AbstractInstruction::Value {dest, ..}
                | AbstractInstruction::Constant {dest, ..} = instr {
                    defined.insert(dest);
                }
            }
        }
        used
    }
}

impl Dataflow for LiveVars {
    type Item = String;

    fn merge(&self, sets : impl Iterator<Item=HashSet<Self::Item>>) -> HashSet<Self::Item> {
        union(sets)
    }

    fn transfer(&self, b : &Block, out_b : &HashSet<Self::Item>) -> HashSet<Self::Item> {
        let mut set : HashSet<String> = HashSet::new();
        set.extend(self.get_uses(b));
        let diff : HashSet<String> 
            = out_b.clone().difference(&self.get_def_vars(b)).map(|s| s.to_string()).collect();
        set.extend(diff);
        set
    }

    fn is_reverse(&self) -> bool {
        true
    }

    fn init(&self) -> HashSet<Self::Item> {
        HashSet::new()
    }
}

pub fn live_vars(cfg : &Cfg) {
    let (in_map, out_map) = df_analysis(cfg, LiveVars);
    for name in cfg.block_map.keys() {
        let mut ins : Vec<&String> = in_map.get(name).unwrap().into_iter().collect();
        let mut outs : Vec<&String> = out_map.get(name).unwrap().into_iter().collect();
        ins.sort();
        outs.sort();
        println!("{name}:");
        println!("    in: {ins:?}");
        println!("    out: {outs:?}");
    }
}
