use bril_rs::{load_abstract_program, output_abstract_program};
use passes::tdce::trivial_dce;

fn main() {
    let mut program = load_abstract_program();
    for func in &mut program.functions {
        trivial_dce(func);
    }
    output_abstract_program(&program)
}
