use bril_rs::{load_abstract_program, output_abstract_program};
use ssa::ssa::{to_ssa, from_ssa};
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long)]
    to_ssa : bool,

    #[clap(short, long)]
    from_ssa : bool,

    #[clap(short, long)]
    roundtrip : bool,
}

fn main() {
    let args = Args::parse();
    let mut program = load_abstract_program();
    for func in &mut program.functions {
        if args.from_ssa {
            from_ssa(func);
        } else if args.roundtrip {
            to_ssa(func);
            from_ssa(func);
        } else {
            to_ssa(func);
        }
    }
}
