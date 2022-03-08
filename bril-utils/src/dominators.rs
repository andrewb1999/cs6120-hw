use crate::cfg::*;
use std::collections::{HashMap, HashSet};

fn post_order_rec(cfg : &Cfg, node : i32, current : &mut Vec<i32>, visited : &mut HashSet<i32>) {
    visited.insert(node);
    let succ = cfg.succ.get(&node).unwrap();
    for s in succ {
        if !visited.contains(s) {
            post_order_rec(cfg, *s, current, visited);
        }
    }
    current.push(node);
}

fn get_reverse_post_order(cfg : &Cfg) -> Vec<i32> {
    let vertices = &mut vec![];
    post_order_rec(cfg, *cfg.block_map.first().unwrap().0, vertices, &mut HashSet::new());
    vertices.into_iter().rev().map(|a| *a).collect()
}

pub fn find_dominators_num(cfg : &Cfg) -> HashMap<i32, HashSet<i32>> {
    let mut dom : HashMap<i32, HashSet<i32>> = HashMap::new();
    let all_blocks : HashSet<i32> = cfg.block_map.keys().copied().collect();
    let (&first_num, _) = cfg.block_map.get_index(0).unwrap();
    for num in cfg.block_map.keys() {
        dom.insert(*num, all_blocks.clone());
    }
    let mut entry_blocks = HashSet::new();
    entry_blocks.insert(first_num);
    dom.insert(first_num, entry_blocks);
    let mut changed = true;
    let vertices = get_reverse_post_order(cfg);
    while changed {
        changed = false;
        for v in &vertices {
            let preds = cfg.pred.get(v).unwrap();
            let mut new_dom : HashSet<i32> = 
                if preds.len() == 0 {
                    HashSet::new()
                } else {
                    preds.into_iter().fold(all_blocks.clone(), 
                    |acc, p| acc.intersection(dom.get(p).unwrap()).copied().collect())
                };
            new_dom.insert(*v);
            changed |= &new_dom != dom.get(v).unwrap();
            dom.insert(*v, new_dom);
        }
    }
    dom
}

fn convert_doms_to_string(cfg : &Cfg, dom : HashMap<i32, HashSet<i32>>) -> HashMap<String, HashSet<String>> {
    let mut dominators = HashMap::new();
    for (v, doms) in dom {
        let name = cfg.name_map.get_by_left(&v).unwrap().clone();
        let name_doms = doms.into_iter().map(|d|
            cfg.name_map.get_by_left(&d).unwrap().to_string()).collect();
        dominators.insert(name, name_doms);
    }
    dominators
}

pub fn find_dominators(cfg : &Cfg) -> HashMap<String, HashSet<String>> {
    let dom = find_dominators_num(cfg);
    convert_doms_to_string(cfg, dom)
}

fn get_strict_doms(dom : &HashMap<i32, HashSet<i32>>) -> HashMap<i32, HashSet<i32>> {
    let mut dom = dom.clone();
    dom.iter_mut().for_each(|(v, d)| {d.remove(&v);});
    dom
}

fn find_immediate_doms(cfg : &Cfg) -> HashMap<i32, Option<i32>> {
    let mut idom = HashMap::new();
    let dom = find_dominators_num(cfg);
    let sdom = get_strict_doms(&dom);
    for (v, doms) in &sdom {
        let mut v_idom = None;
        for a in doms {
            let mut imm = true;
            for d in doms {
                imm &= !sdom.get(d).unwrap().contains(a);
            }
            if imm {
                v_idom = Some(*a);
            }
        }
        idom.insert(*v, v_idom);
    }
    idom
}

#[derive(Default, Debug)]
pub struct DomTree {
    arena : Vec<DomNode>,
    num_to_id : HashMap<i32, usize>,
}

#[derive(Default, Debug, Clone)]
pub struct DomNode {
    pub idx : usize,
    pub label : String,
    pub parent : Option<usize>,
    pub children : Vec<usize>,
}

impl DomNode {
    pub fn new(idx: usize, label: String) -> Self {
        Self {idx, label, parent: None, children: vec![]}
    }
}

impl DomTree {
    pub fn new_node(&mut self, label : String, num : i32) -> usize {
        let idx = self.arena.len();
        self.num_to_id.insert(num, idx);
        self.arena.push(DomNode::new(idx, label));
        idx
    }

    pub fn get_node(&self, v : &i32) -> Option<&DomNode> {
        if let Some(node_id) = self.num_to_id.get(v) {
            self.arena.get(*node_id)
        } else {
            None
        }
    }

    fn get_num_id(&self, v : &i32) -> Option<&usize> {
        self.num_to_id.get(v)
    }

    pub fn add_child(&mut self, parent : &i32, child : &i32) {
        let child_idx = *self.get_num_id(child).unwrap();
        let parent_idx = *self.get_num_id(parent).unwrap();
        let child_node = self.arena.get_mut(child_idx).unwrap();
        child_node.parent = Some(parent_idx);
        let parent_node = self.arena.get_mut(parent_idx).unwrap();
        parent_node.children.push(child_idx);
    }
}

pub fn form_dom_tree(cfg : &Cfg) -> DomTree {
    let idom = find_immediate_doms(cfg);
    let mut tree = DomTree::default();
    let mut idom : Vec<_> = idom.into_iter().collect();
    idom.sort_by_key(|(v,_)| *v);
    for (v, _) in &idom {
        let label = cfg.name_map.get_by_left(v).unwrap();
        tree.new_node(label.to_string(), *v);
    }
    for (v, parent) in idom {
        if let Some(p) = parent {
            tree.add_child(&p, &v);
        }
    }
    tree
}

pub fn get_dominance_frontier_num(cfg : &Cfg) -> HashMap<i32, HashSet<i32>> {
    let dom = find_dominators_num(cfg);
    let sdom = get_strict_doms(&dom);
    let mut frontier : HashMap<i32, HashSet<i32>> = HashMap::new();
    for (a, _) in &dom {
        let mut set_a = HashSet::new();
        for (b, d) in &sdom {
            if !d.contains(a) {
                for p in cfg.pred.get(b).unwrap() {
                    if dom.get(p).unwrap().contains(&a) {
                        set_a.insert(*b);
                    }
                }
            }
        }
        frontier.insert(*a, set_a);
    }
    frontier
}

pub fn get_dominance_frontier(cfg : &Cfg) -> HashMap<String, HashSet<String>> {
    let frontier = get_dominance_frontier_num(cfg);
    convert_doms_to_string(cfg, frontier)
}

pub fn print_dominance_frontier(frontier : &HashMap<String, HashSet<String>>) {
    println!("Frontier");
    println!("--------------------------------");
    let mut frontier = frontier.into_iter().collect::<Vec<_>>();
    frontier.sort_by_key(|(n, _)| *n);
    for (node, front) in  frontier {
        print!("{node}: ");
        let mut front = front.into_iter().collect::<Vec<_>>();
        front.sort();
        println!("{front:?}");
    }
}

pub fn print_dominator_tree(tree : &DomTree) {
    println!("Tree");
    println!("--------------------------------");
    for node in &tree.arena {
        println!("{node:?}")
    }
}

pub fn print_dominators(doms : &HashMap<String, HashSet<String>>) {
    let mut doms : Vec<_> = doms.into_iter().collect();
    doms.sort_by_key(|d| d.0);
    println!("Dominators");
    println!("--------------------------------");
    for (name, dom) in doms {
        println!("{name}: {dom:?}")
    }
}
