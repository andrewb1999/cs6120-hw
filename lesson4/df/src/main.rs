use df::cfg::*;
use df::form_blocks::*;
use df::df::*;
use bril_rs::load_abstract_program;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long)]
    live : bool,

    #[clap(short, long)]
    decl : bool,
}


fn main() {
    let args = Args::parse();
    let mut program = load_abstract_program();
    for func in &mut program.functions {
        let blocks = form_blocks(&func);
        let cfg = form_cfg(blocks);
        if args.live {
            live_vars(&cfg);
        } else {
            declared_vars(&cfg);
        }
    }
}
