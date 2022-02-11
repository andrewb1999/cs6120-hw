use std::collections::{HashMap, HashSet};
use bril_rs::{AbstractFunction, AbstractCode, AbstractInstruction, ConstOps, Literal};
use crate::form_blocks::*;
// use crate::literal::Literal;

#[derive(Eq, Hash, Clone)]
struct ValueExpr {
    op_code : String,
    args : Vec<i32>,
}

impl ValueExpr {
    fn new(op_code : String, args : Vec<i32>) -> ValueExpr {
        ValueExpr {op_code, args}
    }
}

impl PartialEq for ValueExpr {
    fn eq(&self, other : &Self) -> bool {
        self.op_code == other.op_code &&
            self.args == other.args
    }
}

#[derive(Default)]
struct LvnTable {
    vector : Vec<(Option<ValueExpr>, String)>,
    map : HashMap<ValueExpr, (i32, String)>,
}

impl LvnTable {
    fn add_value(&mut self, value_expr : Option<ValueExpr>, home : &String) -> i32 {
        let val_num : i32 = self.vector.len().try_into().unwrap();
        self.vector.push((value_expr.clone(), home.to_string()));
        match value_expr {
            Some(expr) => {
                self.map.insert(expr, (val_num, home.to_string()));
            },
            None => (),
        }
        val_num
    }

    fn next_value(&self) -> i32 {
        self.vector.len().try_into().unwrap()
    }

    fn lookup_value(&self, val : i32) -> Option<(Option<ValueExpr>, String)> {
        match self.vector.get(usize::try_from(val).unwrap()) {
            Some(t) => Some(t.clone()),
            None => None
        }
    }

    fn lookup_expr(&self, value_expr : &ValueExpr, prop : bool) -> Option<(i32, String)> {
        if prop &&  value_expr.op_code == "id"{
            let val = value_expr.args[0];
            match self.lookup_value(val) {
                Some((_, var)) => Some((val, var.to_string())),
                None => None,
            }
        } else {
            match self.map.get(value_expr) {
                Some(t) => Some(t.clone()),
                None => None,
            }
        }
    }

    fn contains_expr(&self, value_expr : &ValueExpr, prop : bool) -> bool {
        if prop && value_expr.op_code == "id" {
            let val = value_expr.args[0];
            match self.vector.get(usize::try_from(val).unwrap()) {
                Some(_) => true,
                None => false,
            }
        } else {
            self.map.contains_key(value_expr)
        }
    }
}

fn copy_instr(instr : AbstractInstruction, home_var : &String) -> AbstractCode {
    let new_instr = match instr {
        AbstractInstruction::Value {op_type, dest, funcs, labels, ..} => {
            AbstractInstruction::Value {op : "id".to_string(), args : vec![home_var.to_string()],
                                        op_type, dest, funcs, labels}
        },
        _ => instr,
    };
    AbstractCode::Instruction(new_instr)
}

fn reform_instr(mut instr : AbstractInstruction, dest : Option<String>,
    table : &LvnTable, var2num : &HashMap<String, i32>) -> AbstractCode {
    match &mut instr {
        AbstractInstruction::Value {args, ..} | AbstractInstruction::Effect {args, ..} => {
            for i in 0..args.len() {
                let a = &args[i];
                let &num = var2num.get(a).unwrap();
                let (_, var) = table.lookup_value(num).unwrap();
                args[i] = var.to_string();
            }
        },
        _ => (),
    };
    let new_instr = match dest {
        Some(dest) => {
            match instr {
                AbstractInstruction::Value {args, op, op_type, labels, funcs, ..} => {
                    AbstractInstruction::Value{args, op, op_type, labels, funcs, dest}
                },
                AbstractInstruction::Constant {op, const_type, value, ..} => {
                    AbstractInstruction::Constant {op, const_type, value, dest}
                },
                x => x,
            }
        },
        None => instr,
    };
    AbstractCode::Instruction(new_instr)
}

fn overwritten_later(instrs : &Vec<AbstractCode>) -> Vec<bool> {
    let mut output = vec![true; instrs.len()];
    let mut seen = HashSet::new();
    for (i, code) in instrs.into_iter().enumerate().rev() {
        match code {
            AbstractCode::Instruction(instr) => {
                match instr {
                    AbstractInstruction::Value {dest, ..} | AbstractInstruction::Constant {dest, ..} => {
                        if !seen.contains(dest) {
                            output[i] = false;
                            seen.insert(dest);
                        }
                    },
                    _ => (),
                }
            },
            _ => (),
        }
    }
    output
}

fn block_inputs(instrs : &Vec<AbstractCode>) -> HashSet<String> {
    let mut read : HashSet<String> = HashSet::new();
    let mut written : HashSet<&String> = HashSet::new();
    for code in instrs {
        match code {
            AbstractCode::Instruction(instr) => {
                match instr {
                    AbstractInstruction::Value {args, ..} 
                    | AbstractInstruction::Effect {args, ..} => {
                        for a in args.iter().collect::<HashSet<&String>>().difference(&written) {
                            read.insert(a.to_string());
                        }
                    }
                    _ => (),
                }
                match instr {
                    AbstractInstruction::Value {dest, ..} 
                    | AbstractInstruction::Constant {dest, ..} => {
                        written.insert(dest);
                },
                    _ => (),
                }
            },
            _ => (),
        }
    }
    read
}

fn new_var(val : i32) -> String {
    format!("lvn.{}", val)
}

fn get_dest(table : &LvnTable, dest : &String, overwritten_later : bool) -> String {
    if overwritten_later {
        new_var(table.next_value())
    } else {
        dest.to_string()
    }
}

fn const_instr(instr : AbstractInstruction, value : Literal) -> AbstractCode {
    let new_instr = match instr {
        AbstractInstruction::Value {dest, op_type, ..} => 
            AbstractInstruction::Constant {op : ConstOps::Const, dest, const_type : op_type, value},
        AbstractInstruction::Constant {dest, const_type, ..} =>
            AbstractInstruction::Constant {op : ConstOps::Const, dest, const_type, value},
        _ => instr,
    };
    AbstractCode::Instruction(new_instr)
}

// fn apply_fold(op : &String, a : &Literal, b : Option<&Literal>) -> Option<Literal> {
//     let a_type : Type = 
//     match op {
//         _ => None,
//     }
// }

// fn const_fold(num2const : &mut HashMap<i32, Literal>, value_expr : &ValueExpr, fold : bool) -> Option<Literal> {
//     if fold {
//         let args = &value_expr.args;
//         let all_const = args.into_iter().map(|a| {
//             match num2const.get(&a) {
//                 Some(_) => true,
//                 None => false,
//             }
//         }).fold(true, |acc, b| acc && b);
//         if all_const {
//             apply_fold(&value_expr.op_code, num2const.get(&value_expr.args[0]).unwrap(), 
//                 num2const.get(&value_expr.args[1]))
//         } else {
//             None
//         }
//     } else {
//         None
//     }
// }

fn lvn_pass(block : Block, prop : bool, comm : bool, fold : bool) -> Block {
    let mut new_block = Block {instrs : Vec::new()};
    let mut table = LvnTable::default();
    let mut var2num : HashMap<String, i32> = HashMap::new();
    let mut num2const : HashMap<i32, Literal> = HashMap::new();
    for var in block_inputs(&block.instrs) {
        let num = table.add_value(None, &var);
        var2num.insert(var, num);
    }
    for (overwritten_later, code) in overwritten_later(&block.instrs).into_iter().zip(block.instrs.into_iter()) {
        match code {
            AbstractCode::Instruction(instr) => 
                match &instr {
                    AbstractInstruction::Value {op, args, dest, ..} if op != "call" => {
                        // args.into_iter().for_each(|a| {
                        //     match var2num.get(a) {
                        //         None => println!("{}", a),
                        //         _ => (),
                        //     }
                        // });
                        let mut arg_vals : Vec<i32> = args.into_iter()
                                                      .map(|a| *var2num.get(a).unwrap())
                                                      .collect();
                        match op.as_str() {
                            "add" | "mul" | "and" | "or" | "eq"
                                if comm => arg_vals.sort(),
                            _ => (),
                        }
                        let value_expr = ValueExpr::new(op.to_string(), arg_vals);
                        // let c = const_fold(&mut num2const, &value_expr, fold);
                        if table.contains_expr(&value_expr, prop) {
                            let (num, _) = table.lookup_expr(&value_expr, prop).unwrap();
                            var2num.insert(dest.to_string(), num);
                            // match c {
                            //     Some(constant) => new_block.instrs.push(const_instr(instr, constant)),
                            //     None => new_block.instrs.push(copy_instr(instr, &var)),
                            // }
                        } else {
                            let new_dest = get_dest(&table, dest, overwritten_later);
                            let num = table.add_value(Some(value_expr), &new_dest);
                            var2num.insert(dest.to_string(), num);
                            new_block.instrs.push(reform_instr(instr, Some(new_dest), &table, &var2num));
                        }
                    },
                    AbstractInstruction::Constant {dest, ..} => {
                        let new_dest = get_dest(&table, dest, overwritten_later);
                        let num = table.add_value(None, &new_dest);
                        var2num.insert(dest.to_string(), num);
                        new_block.instrs.push(reform_instr(instr, Some(new_dest), &table, &var2num));
                    },
                    _ => new_block.instrs.push(reform_instr(instr, None, &table, &var2num)),
                },
            AbstractCode::Label {..} => new_block.instrs.push(code),
        } 
    }
    new_block
}

pub fn local_value_numbering(func : &mut AbstractFunction, prop : bool, comm : bool, fold : bool) {
    let blocks = form_blocks(func);
    let new_blocks = blocks.into_iter()
                           .map(|b| lvn_pass(b, prop, comm, fold))
                           .collect();
    func.instrs = flatten_blocks(new_blocks);
}
