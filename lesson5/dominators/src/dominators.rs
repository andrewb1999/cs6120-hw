use crate::cfg::*;
use std::collections::HashMap;

pub fn find_dominators(cfg : &Cfg) -> HashMap<String, Vec<String>> {
    let mut dom = HashMap::new();
    let all_blocks : Vec<String> = cfg.block_map.clone().into_keys()
    for (name, block) in cfg.block_map {
        
    }
    dom
}
