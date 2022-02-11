use bril_rs::{load_abstract_program, output_abstract_program};
use passes::{lvn::local_value_numbering, tdce::trivial_dce};
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long)]
    prop : bool,

    #[clap(short, long)]
    comm : bool,

    #[clap(short, long)]
    fold : bool,

    #[clap(short, long)]
    no_dce : bool,
}

fn main() {
    let args = Args::parse();
    let mut program = load_abstract_program();
    for f in &mut program.functions {
        local_value_numbering(f, args.prop, args.comm, args.fold);
        if !args.no_dce {
            trivial_dce(f);
        }
    }
    output_abstract_program(&program)
}
