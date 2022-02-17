use df::cfg::*;
use df::form_blocks::*;
use bril_rs::{load_abstract_program, output_abstract_program};

fn main() {
    let mut program = load_abstract_program();
    for func in &mut program.functions {
        let blocks = form_blocks(&func);
        let cfg = form_cfg(blocks);
        func.instrs = reassemble(cfg);
    }
    println!("{}", program);
}
