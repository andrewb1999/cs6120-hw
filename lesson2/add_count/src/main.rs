use bril_rs::load_abstract_program;
use bril_rs::AbstractCode;
use bril_rs::AbstractInstruction;

fn is_add(instr : &AbstractInstruction) -> i32 {
    match instr {
        AbstractInstruction::Value { op, .. } if op == "add" => 1,
        _ => 0
    }
}

fn main() {
    let program = load_abstract_program();
    let mut num_adds = 0;
    for func in program.functions {
        for i in func.instrs {
            let add = match &i {
                AbstractCode::Instruction(instr) => is_add(instr),
                _ => 0
            };
            num_adds += add;
        }
    }
    if num_adds == 1 {
        println!("Program contains 1 add instruction")
    } else {
        println!("Program contains {} add instructions", num_adds)
    }
}
