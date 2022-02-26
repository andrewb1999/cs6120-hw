use crate::cfg::*;
use std::collections::{HashMap, HashSet};

fn get_paths_to_node(cfg : &Cfg, entry_node : i32, node : i32, path : &Vec<i32>) -> Vec<Vec<i32>>{
    let mut path = path.clone();
    path.push(entry_node);
    if entry_node == node {
        vec![path]
    } else {
        let mut paths = vec![];
        for succ in cfg.succ.get(&entry_node).unwrap() {
            if !path.contains(succ) {
                let new_path = get_paths_to_node(cfg, *succ, node, &path);
                paths.extend(new_path)
            }
        }
        paths
    }
}

fn dominates(cfg : &Cfg, entry_node : i32, node_a : i32, node_b : i32) -> bool {
    let paths = get_paths_to_node(cfg, entry_node, node_b, &vec![]);
    for path in paths {
        if !path.contains(&node_a) {
            return false;
        }
    }
    true
}

pub fn validate_dominators(cfg : &Cfg, dom : HashMap<i32, HashSet<i32>>) {
    let (entry_node, _) = cfg.block_map.first().unwrap();
    for (node, doms) in dom {
        for d in doms {
            assert!(dominates(cfg, *entry_node, d, node))
        }
    }
}
