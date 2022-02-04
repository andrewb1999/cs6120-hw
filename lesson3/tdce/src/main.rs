pub mod form_blocks;
pub use form_blocks::*;

use std::collections::HashSet;
use bril_rs::AbstractCode;
use bril_rs::{load_abstract_program, output_abstract_program};
use bril_rs::AbstractInstruction;
use bril_rs::AbstractProgram;

// fn trivial_global_dce_pass(prog : &AbstractProgram) -> bool {
//     let mut used = HashSet::new();
//     for func in &prog.functions {
//         for code in &func.instrs {
//             match code {
//                 AbstractCode::Instruction(instr) =>
//                     match instr {
//                         AbstractInstruction::Value {args, ..} => used.extend(args),
//                         AbstractInstruction::Effect {args, ..} => used.extend(args),
//                         _ => (),
//                     },
//                 _ => (),
//             }
//         }
//     }
    
//     for func in &prog.functions{
//         for code in &func.instrs {
//             match code {
//                 AbstractCode::Instruction(instr) =>
//                     match instr {
//                         AbstractInstruction::Constant {dest, ..} 
//                         if !used.contains(dest) => 
                            
//                     }
//             }
//         }
//     }
//     false
// }

fn main() {
    let program = load_abstract_program();
    let blocks = form_blocks(program);
    for block in blocks {
        for instr in block.instrs {
            println!("{}", instr)
        }
        println!("-----------")
    }
    // while trivial_global_dce_pass(&program) {}
}
