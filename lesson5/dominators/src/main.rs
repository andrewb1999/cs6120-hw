use dominators::cfg::*;
use dominators::form_blocks::*;
use dominators::dominators::*;
use dominators::test::validate_dominators;
use bril_rs::load_abstract_program;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long)]
    dominators : bool,

    #[clap(short, long)]
    tree : bool,

    #[clap(short, long)]
    frontier : bool,

    #[clap(short, long)]
    validate : bool,
}

fn main() {
    let args = Args::parse();
    let mut program = load_abstract_program();
    for func in &mut program.functions {
        let blocks = form_blocks(&func);
        let cfg = form_cfg(blocks);
        let mut pred_sorted : Vec<_> = cfg.pred.iter().collect();
        pred_sorted.sort_by_key(|a| a.0);
        let mut succ_sorted : Vec<_> = cfg.succ.iter().collect();
        succ_sorted.sort_by_key(|a| a.0);
        if args.tree {
           let dom_tree = form_dom_tree(&cfg);
            print_dominator_tree(&dom_tree);
        } else if args.frontier {
            let frontier = get_dominance_frontier(&cfg);
            print_dominance_frontier(&frontier);
        } else if args.validate {
            let doms = find_dominators_num(&cfg);
            validate_dominators(&cfg, doms);
            println!("Dominators are correct!");
        } else {
            let doms = find_dominators(&cfg);
            print_dominators(&doms);
        }
    }
}
