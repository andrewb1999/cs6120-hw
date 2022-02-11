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

fn const_instr(instr : AbstractInstruction, new_dest : Option<String>, value : Literal) -> AbstractCode {
    let (dest, const_type) = match instr {
        AbstractInstruction::Value {dest, op_type, ..} => 
            if let Some(new_dest) = new_dest {
                (new_dest, op_type)
            } else {
                (dest, op_type)
            },
        AbstractInstruction::Constant {dest, const_type, ..} =>
            if let Some(new_dest) = new_dest {
                (new_dest, const_type)
            } else {
                (dest, const_type)
            },
        _ => panic!("Cannot convert effect into const"),
    };
    let new_instr = AbstractInstruction::Constant {op : ConstOps::Const, dest, const_type, value};
    AbstractCode::Instruction(new_instr)
}

fn binop_int(a : &Literal, b : &Literal, func : &dyn Fn(&i64, &i64) -> i64) -> Option<Literal> {
    match a {
        Literal::Int(x) => {
            if let Literal::Int(y) = b {
                Some(Literal::Int(func(x, y)))
            } else {
                None
            }
        },
        _ => None,
    }
}

fn binop_float(a : &Literal, b : &Literal, func : &dyn Fn(&f64, &f64) -> f64) -> Option<Literal> {
    match a {
        Literal::Float(x) => {
            if let Literal::Float(y) = b {
                Some(Literal::Float(func(x, y)))
            } else {
                None
            }
        },
        _ => None,
    }
}

fn binop_bool(a : &Literal, b : &Literal, func : &dyn Fn(&bool, &bool) -> bool) -> Option<Literal> {
    match a {
        Literal::Bool(x) => {
            if let Literal::Bool(y) = b {
                Some(Literal::Bool(func(x, y)))
            } else {
                None
            }
        },
        _ => None,
    }
}

fn apply_fold(op : &String, a : &Literal, b : Option<&Literal>) -> Option<Literal> {
    match op.to_string().as_str() {
        "add"  => binop_int(a, b.unwrap(), &|a, b| a + b),
        "mul"  => binop_int(a, b.unwrap(), &|a, b| a * b),
        "sub"  => binop_int(a, b.unwrap(), &|a, b| a - b),
        "div"  => binop_int(a, b.unwrap(), &|a, b| a / b),
        "addf" => binop_float(a, b.unwrap(), &|a, b| a + b),
        "mulf" => binop_float(a, b.unwrap(), &|a, b| a * b),
        "subf" => binop_float(a, b.unwrap(), &|a, b| a - b),
        "divf" => binop_float(a, b.unwrap(), &|a, b| a / b),
        "eq"   => binop_bool(a, b.unwrap(), &|a, b| a == b),
        "lt"   => binop_bool(a, b.unwrap(), &|a, b| a < b),
        "gt"   => binop_bool(a, b.unwrap(), &|a, b| a > b),
        "le"   => binop_bool(a, b.unwrap(), &|a, b| a <= b),
        "ge"   => binop_bool(a, b.unwrap(), &|a, b| a >= b),
        "and"  => binop_bool(a, b.unwrap(), &|a, b| *a && *b),
        "or"   => binop_bool(a, b.unwrap(), &|a, b| *a || *b),
        "not"  => 
            match a {
                Literal::Bool(x) => Some(Literal::Bool(!x)),
                _ => None,
            },
        _ => None,
    }
}

fn const_fold(num2const : &mut HashMap<i32, Literal>, value_expr : &ValueExpr, 
    num : i32, fold : bool) -> Option<Literal> {
    if fold {
        let args = &value_expr.args;
        let all_const = args.into_iter().map(|a| {
            match num2const.get(&a) {
                Some(_) => true,
                None => {false},
            }
        }).fold(true, |acc, b| acc && b);
        if all_const {
            let c = apply_fold(&value_expr.op_code, num2const.get(&value_expr.args[0]).unwrap(),
                num2const.get(&value_expr.args[1]));
            if let Some(c) = c {
                num2const.insert(num, c.clone());
                Some(c)
            } else {
                None
            }
        } else {
            let any_const = args.into_iter().map(|a| {
                match num2const.get(&a) {
                    Some(_) => true,
                    None => {false},
                }
            }).fold(false, |acc, b| acc || b);
            match value_expr.op_code.as_str() {
                "eq" | "le" | "ge" 
                    if value_expr.args[0] == value_expr.args[0] => Some(Literal::Bool(true)),
                "and" | "or"
                    if any_const => {
                        let const_num = if num2const.contains_key(&value_expr.args[0]) {
                            value_expr.args[0]
                        } else {
                            value_expr.args[1]
                        };
                        if let Literal::Bool(b) = num2const.get(&const_num).unwrap() {
                            let bool_literal = (value_expr.op_code == "and" && !b)
                                || (value_expr.op_code == "or" && *b);
                            Some(Literal::Bool(bool_literal))
                        } else {
                            None
                        } 
                    },
                _ => None,
            }
        }
    } else {
        None
    }
}

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
                        args.into_iter().for_each(|a| {
                            match var2num.get(a) {
                                None => println!("{}", a),
                                _ => (),
                            }
                        });
                        let mut arg_vals : Vec<i32> = args.into_iter()
                                                      .map(|a| *var2num.get(a).unwrap())
                                                      .collect();
                        match op.as_str() {
                            "add" | "mul" | "and" | "or" | "eq"
                                if comm => arg_vals.sort(),
                            _ => (),
                        }
                        let value_expr = ValueExpr::new(op.to_string(), arg_vals);
                        if table.contains_expr(&value_expr, prop) {
                            let (num, var) = table.lookup_expr(&value_expr, prop).unwrap();
                            let c = const_fold(&mut num2const, &value_expr, num, fold);
                            var2num.insert(dest.to_string(), num);
                            match c {
                                Some(constant) => new_block.instrs.push(const_instr(instr, None, constant)),
                                None => new_block.instrs.push(copy_instr(instr, &var)),
                            }
                        } else {
                            let new_dest = get_dest(&table, dest, overwritten_later);
                            let num = table.next_value();
                            let c = const_fold(&mut num2const, &value_expr, num, fold);
                            let num = table.add_value(Some(value_expr), &new_dest);
                            var2num.insert(dest.to_string(), num);
                            match c {
                                Some(constant) => {
                                    var2num.insert(new_dest.to_string(), num);
                                    new_block.instrs.push(const_instr(instr, Some(new_dest.to_string()), constant));
                                },
                                None => new_block.instrs.push(
                                    reform_instr(instr, Some(new_dest), &table, &var2num)),
                            }
                        }
                    },
                    AbstractInstruction::Constant {dest, value, ..} => {
                        let new_dest = get_dest(&table, dest, overwritten_later);
                        let num = table.add_value(None, &new_dest);
                        var2num.insert(dest.to_string(), num);
                        num2const.insert(num, value.clone());
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
