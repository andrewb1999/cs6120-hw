use bril_rs::{load_abstract_program, output_abstract_program, AbstractProgram, AbstractFunction};
use bril_rs::{AbstractInstruction, AbstractCode, AbstractType};
use serde::{Serialize, Deserialize};
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
struct TraceItem {
    instr: AbstractInstruction,
    line_num: i64,
}

fn read_trace() -> Vec<TraceItem> {
    let file = fs::read_to_string("/tmp/trace.txt").expect("Could not read trace file");
    let trace : Vec<TraceItem> = serde_json::from_str(&file).expect("Could not convert trace into instrs");
    trace
}

fn convert_trace_to_straight_line(trace : Vec<AbstractInstruction>) -> Vec<AbstractCode> {
    let mut trace : Vec<AbstractCode> = std::iter::once(
    AbstractCode::Instruction(AbstractInstruction::Effect {
        op: "speculate".to_string(),
        args: vec![],
        labels: vec![],
        funcs: vec![]
    })).chain(trace.into_iter().filter(|i| {
        if let AbstractInstruction::Effect {op, ..} = &i {
            if op == "jmp" {
                return false;
            }
        }
        return true;
    }).map(|i| {
        if let AbstractInstruction::Effect {op, args, ..} = &i {
            if op == "br" {
                return AbstractCode::Instruction(AbstractInstruction::Effect {
                    op: "guard".to_string(),
                    args: vec![args[0].clone()],
                    funcs: vec![],
                    labels: vec!["abort".to_string()]
                });
            }
        }
        return AbstractCode::Instruction(i);
    })).collect();
    trace.push(AbstractCode::Instruction(AbstractInstruction::Effect {
        op: "commit".to_string(),
        args: vec![],
        labels: vec![],
        funcs: vec![]
    }));
    trace.push(AbstractCode::Instruction(AbstractInstruction::Effect {
        op: "jmp".to_string(),
        args: vec![],
        labels: vec!["traceend".to_string()],
        funcs: vec![]
    }));
    trace.push(AbstractCode::Label {label: "abort".to_string()});
    let call_locs = trace.iter().enumerate().filter(|(x, i)| {
        if let AbstractCode::Instruction(instr) = i {
            if let AbstractInstruction::Value {op, ..} | AbstractInstruction::Effect {op, ..} = instr {
                return op == "call";
            }
        }
        return false;
    });

    trace
}

fn insert_trace(program: &mut AbstractFunction, trace: Vec<AbstractCode>) {
    program.instrs = trace.into_iter().chain(program.instrs.drain(0..).into_iter()).collect();
}

fn add_abort(mut code: Vec<AbstractCode>) -> Vec<AbstractCode> {
    code.push(AbstractCode::Instruction(
        AbstractInstruction::Effect {
            op: "jmp".to_string(),
            args: vec![],
            labels: vec!["exit".to_string()],
            funcs: vec![]
        })
    );
    code.push(AbstractCode::Label{ label: "abort".to_string() });
    code.push(AbstractCode::Instruction(
        AbstractInstruction::Constant {
            dest: "fourtytwo".to_string(),
            op: bril_rs::ConstOps::Const,
            const_type: Some(AbstractType::Primitive("int".to_string())),
            value: bril_rs::Literal::Int(42)
        })
    );
    code.push(AbstractCode::Instruction(
        AbstractInstruction::Effect {
            op: "print".to_string(),
            args: vec!["fourtytwo".to_string()],
            labels: vec![],
            funcs: vec![]
        })
    );
    code.push(AbstractCode::Label{ label: "exit".to_string() });
    code
}

fn cutoff_trace(trace: Vec<TraceItem>) -> Vec<TraceItem> {
    let mut new_trace = vec![];
    for t in trace {
        if let AbstractInstruction::Effect { op, .. } | AbstractInstruction::Value {op, ..}= &t.instr {
            if op == "print" || op == "call" || op == "store" || op == "alloc" || op == "free" {
                new_trace.push(t);
                break;
            } else {
                new_trace.push(t);
            }
        } else {
            new_trace.push(t);
        }
    }
    new_trace
}

fn insert_trace_end_label(program: &mut AbstractProgram, trace: &mut Vec<TraceItem>) {
    let loc = trace.last().unwrap().line_num;
    let main_fn = program.functions.iter_mut().filter(|x| x.name == "main").last().unwrap();
    main_fn.instrs.insert(loc as usize, AbstractCode::Label{label: "traceend".to_string()});
    trace.pop();
    // for i in &main_fn.instrs {
    //     println!("{i}")
    // }
}

fn strip_trace_line_num(trace: Vec<TraceItem>) -> Vec<AbstractInstruction> {
    trace.into_iter().map(|x| x.instr).collect()
}

fn main() {
    let mut program = load_abstract_program();
    let mut trace = cutoff_trace(read_trace());
    insert_trace_end_label(&mut program, &mut trace);
    let trace = strip_trace_line_num(trace);
    let code = convert_trace_to_straight_line(trace);
    let mut main_fn = program.functions.iter_mut().filter(|x| x.name == "main").last().unwrap();
    insert_trace(&mut main_fn, code);
    output_abstract_program(&program);
    // println!("{program}")
}
