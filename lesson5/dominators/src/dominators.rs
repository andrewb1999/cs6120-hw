use crate::cfg::*;
use std::collections::{HashMap, HashSet};
use indextree::{Arena, NodeId, Node};

fn get_reverse_post_order(cfg : &Cfg) -> Vec<i32> {
    let vertices = cfg.block_map.keys().map(|a| *a).collect();
    vertices
}

fn find_dominators_num(cfg : &Cfg) -> HashMap<i32, HashSet<i32>> {
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

fn find_immediate_strict_doms(cfg : &Cfg) -> HashMap<i32, Option<i32>> {
    let mut idom = HashMap::new();
    let dom = find_dominators_num(cfg);
    let sdom = get_strict_doms(&dom);
    for (v, doms) in &sdom {
        let mut v_idom = None;
        for a in doms {
            let mut imm = true;
            for d in doms {
                imm &= !dom.get(d).unwrap().contains(a);
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
    tree : Arena<String>,
    num_to_id : HashMap<i32, NodeId>,
}

impl DomTree {
    pub fn new_node(&mut self, label : String, num : i32) {
        let node_id = self.tree.new_node(label);
        self.num_to_id.insert(num, node_id);
    }

    pub fn get_node(&self, v : &i32) -> Option<&Node<String>> {
        if let Some(node_id) = self.num_to_id.get(v) {
            self.tree.get(*node_id)
        } else {
            None
        }
    }

    fn get_node_id(&self, v : &i32) -> Option<&NodeId> {
        self.num_to_id.get(v)
    }

    pub fn add_child(&mut self, parent : &i32, child : &i32) {
        let child_node = *self.get_node_id(child).unwrap();
        let parent_node = self.get_node_id(parent).unwrap();
        parent_node.append(child_node, &mut self.tree);
    }
}

pub fn form_dom_tree(cfg : &Cfg) -> DomTree {
    let imm_strict_dom = find_immediate_strict_doms(cfg);
    let mut tree = DomTree::default();
    for (v, _) in &imm_strict_dom {
        let label = cfg.name_map.get_by_left(v).unwrap();
        tree.new_node(label.to_string(), *v);
    }
    for (v, parent) in imm_strict_dom {
        if let Some(p) = parent {
            tree.add_child(&p, &v);
        }
    }
    tree
}

pub fn get_dominance_frontier(cfg : &Cfg) {
    let dom = find_dominators_num(cfg);
    let sdom = get_strict_doms(&dom);
    let mut frontier : HashMap<i32, HashSet<i32>> = HashMap::new();
    for (a, _) in dom {
        frontier.insert(a, HashSet::new());
        for (b, d) in &sdom {
            if !d.contains(b) {

            }
        }
    }
}

pub fn print_dominator_tree(tree : &DomTree) {
    println!("{:?}", tree.tree)
}

pub fn print_dominators(doms : &HashMap<String, HashSet<String>>) {
    let mut doms : Vec<_> = doms.into_iter().collect();
    doms.sort_by_key(|d| d.0);
    for (name, dom) in doms {
        println!("{name}: {dom:?}")
    }
}
