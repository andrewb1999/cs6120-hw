use bril_utils::cfg::*;
use bril_utils::dominators::*;
use bril_utils::form_blocks::*;
use bril_rs::AbstractFunction;

fn insert_phi_nodes(func : &mut AbstractFunction) {

}

fn rename_vars(func : &mut AbstractFunction) {

}

pub fn to_ssa(func : &mut AbstractFunction) {
    let blocks = form_blocks(func);
    let cfg = form_cfg(blocks);
    let frontier = get_dominance_frontier(&cfg);
}

pub fn from_ssa(func : &mut AbstractFunction) {
    todo!()
}
