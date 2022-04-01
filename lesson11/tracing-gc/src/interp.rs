use std::fmt;
use std::hint::unreachable_unchecked;

use crate::basic_block::{BBFunction, BBProgram, BasicBlock};
use crate::error::{InterpError, PositionalInterpError};
use bril_rs::Instruction;

use fxhash::FxHashMap;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Debug)]
struct Scope {
    vars: Vec<Value>
}

impl Scope {
  #[inline(always)]
  pub fn new(size: u32) -> Self {
    Self {
      vars: vec![Value::default(); size as usize],
    }
  }
  #[inline(always)]
  pub fn get(&self, ident: &u32) -> &Value {
    // A bril program is well formed when, dynamically, every variable is defined before its use.
    // If this is violated, this will return Value::Uninitialized and the whole interpreter will come crashing down.
    self.vars.get(*ident as usize).unwrap()
  }
  #[inline(always)]
  pub fn set(&mut self, ident: u32, val: Value) {
    self.vars[ident as usize] = val;
  }
}

#[derive(Debug)]
struct Environment {
  env: Vec<Scope>
}

impl Environment {
  #[inline(always)]
  pub fn new(initial_scope: Scope) -> Self {
    Self {
      env: vec![initial_scope],
    }
  }

  #[inline(always)]
  pub fn get_current_scope_mut(&mut self) -> &mut Scope {
    self.env.last_mut().unwrap()
  }

  #[inline(always)]
  pub fn get_current_scope(&self) -> &Scope {
    self.env.last().unwrap()
  }

  #[inline(always)]
  pub fn append(&mut self, scope: Scope) {
    self.env.push(scope);
  }

  #[inline(always)]
  pub fn pop(&mut self) {
    self.env.pop();
  }
}

const HEAP_SIZE: usize = 1000000;
const INITIAL_GC_LIMIT: i64 = 16;
const GC_GROWTH_FACTOR: i64 = 2;

struct Heap {
  memory: Vec<Value>,
  size_map: FxHashMap<usize, i64>,
  base_ptr: usize,
  is_top: bool,
  gc_limit: i64,
}

impl Default for Heap {
    fn default() -> Self {
        Self { 
            memory: vec![Value::default(); HEAP_SIZE], 
            size_map: FxHashMap::default(),
            base_ptr: 0,
            is_top: true,
            gc_limit: INITIAL_GC_LIMIT,
        }
    }
}

impl Heap {
  #[inline(always)]
  fn should_run_gc(&mut self, amount : i64) -> bool {
      if amount + self.allocated_size() >= self.gc_limit {
        self.gc_limit *= GC_GROWTH_FACTOR;
        true
      } else {
        false
      }
  }

  fn flip(&mut self) {
      if self.is_top {
          self.base_ptr = HEAP_SIZE / 2;
      } else {
          self.base_ptr = 0;
      }
      self.is_top = !self.is_top;
  }

  fn clear(&mut self) {
      if self.is_top {
          for i in (HEAP_SIZE / 2)..HEAP_SIZE {
              *self.memory.get_mut(i).unwrap() = Value::default();
          }
      } else {
          for i in 0..(HEAP_SIZE / 2) {
              *self.memory.get_mut(i).unwrap() = Value::default();
          }
      }
  }

  fn process_field(&mut self, fld : &Value) -> Option<Value> {
      if let Value::Pointer(from_ref) = fld {
          let size = self.size_map.remove(&from_ref.base);
          if let Some(s) = size {
              let to_ref = self.alloc(s).unwrap();
              for i in 0..s {
                  let from = self.memory.get(i as usize + from_ref.base).unwrap().clone();
                  let to = self.memory.get_mut(i as usize + to_ref.base).unwrap();
                  *to = from;
              }
              self.size_map.insert(to_ref.base, s);
              return Some(Value::Pointer(Pointer {base: to_ref.base, offset: from_ref.offset}))
          }
      }
      None
  }

  #[inline(always)]
  fn gc(&mut self, value_store: &mut Environment) {
      self.flip();
      let mut scan = self.base_ptr;
      for scope in &mut value_store.env {
          for root in &mut scope.vars {
              if let Some(ptr) = self.process_field(root) {
                  *root = ptr;
              }
          }
      }
      while scan != self.base_ptr {
        let elem = self.memory.get(scan).unwrap();
        scan = scan + *self.size_map.get(&scan).unwrap() as usize;
        if let Value::Pointer(p) = elem {
            let base = p.base;
            let size = self.size_map.get(&p.base);
            if let Some(s) = size {
                for i in base..(base + *s as usize) {
                    let fld = self.memory.get(i).unwrap().clone();
                    if let Some(ptr) = self.process_field(&fld) {
                        let fld = self.memory.get_mut(i).unwrap();
                        *fld = ptr;
                    }
                }
            }
        }
      }
      self.clear();
  }

  const fn allocated_size(&self) -> i64 {
      if self.is_top {
          self.base_ptr as i64
      } else {
          (self.base_ptr - HEAP_SIZE / 2) as i64
      }
  }

  #[inline(always)]
  fn alloc(&mut self, amount: i64) -> Result<Pointer, InterpError> {
    if amount < 0 || amount > (HEAP_SIZE / 2) as i64 - self.allocated_size() {
      return Err(InterpError::CannotAllocSize(amount));
    }

    let base = self.base_ptr;
    self.size_map.insert(base, amount);
    self.base_ptr += amount as usize;
    Ok(Pointer { base, offset: 0 })
  }

  #[inline(always)]
  fn free(&mut self, _key: &Pointer) -> Result<(), InterpError> {
      panic!("Cannot free in GC interpreter");
  }

  #[inline(always)]
  fn write(&mut self, key: &Pointer, val: Value) -> Result<(), InterpError> {
    let ptr : usize = key.base + key.offset as usize;
    match self.memory.get_mut(ptr) {
      Some(loc) if key.offset >= 0 => {
        *loc = val;
        Ok(())
      }
      Some(_) | None => Err(InterpError::InvalidMemoryAccess(key.base, key.offset)),
    }
  }

  #[inline(always)]
  fn read(&self, key: &Pointer) -> Result<&Value, InterpError> {
    let ptr : usize = key.base + key.offset as usize;
    self
    .memory
      .get(ptr)
      .ok_or(InterpError::InvalidMemoryAccess(key.base, key.offset))
      .and_then(|val| match val {
        Value::Uninitialized => Err(InterpError::UsingUninitializedMemory),
        _ => Ok(val),
      })
  }
}

#[inline(always)]
fn get_value<'a>(vars: &'a Environment, index: usize, args: &[u32]) -> &'a Value {
  vars.get_current_scope().get(&args[index])
}

#[inline(always)]
fn get_arg<'a, T>(vars: &'a Environment, index: usize, args: &[u32]) -> T
where
  T: From<&'a Value>,
{
  T::from(vars.get_current_scope().get(&args[index]))
}

#[derive(Debug, Clone)]
enum Value {
  Int(i64),
  Bool(bool),
  Float(f64),
  Pointer(Pointer),
  Uninitialized,
}

impl Default for Value {
  fn default() -> Self {
    Self::Uninitialized
  }
}

#[derive(Debug, Clone, PartialEq)]
struct Pointer {
  base: usize,
  offset: i64,
}

impl Pointer {
  const fn add(&self, offset: i64) -> Self {
    Self {
      base: self.base,
      offset: self.offset + offset,
    }
  }
}

impl fmt::Display for Value {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Value::Int(i) => write!(f, "{i}"),
      Value::Bool(b) => write!(f, "{b}"),
      Value::Float(v) => write!(f, "{v}"),
      Value::Pointer(p) => write!(f, "{p:?}"),
      // This is safe because Uninitialized is only used in relation to memory and immediately errors if this value is returned. Otherwise this value can not appear in the code
      Value::Uninitialized => unsafe { unreachable_unchecked() },
    }
  }
}

impl From<&bril_rs::Literal> for Value {
  #[inline(always)]
  fn from(l: &bril_rs::Literal) -> Self {
    match l {
      bril_rs::Literal::Int(i) => Self::Int(*i),
      bril_rs::Literal::Bool(b) => Self::Bool(*b),
      bril_rs::Literal::Float(f) => Self::Float(*f),
    }
  }
}

impl From<bril_rs::Literal> for Value {
  #[inline(always)]
  fn from(l: bril_rs::Literal) -> Self {
    match l {
      bril_rs::Literal::Int(i) => Self::Int(i),
      bril_rs::Literal::Bool(b) => Self::Bool(b),
      bril_rs::Literal::Float(f) => Self::Float(f),
    }
  }
}

impl From<&Value> for i64 {
  #[inline(always)]
  fn from(value: &Value) -> Self {
    if let Value::Int(i) = value {
      *i
    } else {
      // This is safe because we type check the program beforehand
      unsafe { unreachable_unchecked() }
    }
  }
}

impl From<&Value> for bool {
  #[inline(always)]
  fn from(value: &Value) -> Self {
    if let Value::Bool(b) = value {
      *b
    } else {
      // This is safe because we type check the program beforehand
      unsafe { unreachable_unchecked() }
    }
  }
}

impl From<&Value> for f64 {
  #[inline(always)]
  fn from(value: &Value) -> Self {
    if let Value::Float(f) = value {
      *f
    } else {
      // This is safe because we type check the program beforehand
      unsafe { unreachable_unchecked() }
    }
  }
}

impl<'a> From<&'a Value> for &'a Pointer {
  #[inline(always)]
  fn from(value: &'a Value) -> Self {
    if let Value::Pointer(p) = value {
      p
    } else {
      // This is safe because we type check the program beforehand
      unsafe { unreachable_unchecked() }
    }
  }
}

// todo do this with less function arguments
#[inline(always)]
fn execute_value_op<'a, T: std::io::Write>(
  prog: &'a BBProgram,
  op: &bril_rs::ValueOps,
  dest: u32,
  args: &[u32],
  labels: &[String],
  funcs: &[String],
  out: &mut T,
  value_store: &mut Environment,
  heap: &mut Heap,
  last_label: Option<&String>,
  instruction_count: &mut u32,
) -> Result<(), InterpError> {
  use bril_rs::ValueOps::*;
  match *op {
    Add => {
      let arg0 = get_arg::<i64>(value_store, 0, args);
      let arg1 = get_arg::<i64>(value_store, 1, args);
      value_store.get_current_scope_mut().set(dest, Value::Int(arg0.wrapping_add(arg1)));
    }
    Mul => {
      let arg0 = get_arg::<i64>(value_store, 0, args);
      let arg1 = get_arg::<i64>(value_store, 1, args);
      value_store.get_current_scope_mut().set(dest, Value::Int(arg0.wrapping_mul(arg1)));
    }
    Sub => {
      let arg0 = get_arg::<i64>(value_store, 0, args);
      let arg1 = get_arg::<i64>(value_store, 1, args);
      value_store.get_current_scope_mut().set(dest, Value::Int(arg0.wrapping_sub(arg1)));
    }
    Div => {
      let arg0 = get_arg::<i64>(value_store, 0, args);
      let arg1 = get_arg::<i64>(value_store, 1, args);
      value_store.get_current_scope_mut().set(dest, Value::Int(arg0.wrapping_div(arg1)));
    }
    Eq => {
      let arg0 = get_arg::<i64>(value_store, 0, args);
      let arg1 = get_arg::<i64>(value_store, 1, args);
      value_store.get_current_scope_mut().set(dest, Value::Bool(arg0 == arg1));
    }
    Lt => {
      let arg0 = get_arg::<i64>(value_store, 0, args);
      let arg1 = get_arg::<i64>(value_store, 1, args);
      value_store.get_current_scope_mut().set(dest, Value::Bool(arg0 < arg1));
    }
    Gt => {
      let arg0 = get_arg::<i64>(value_store, 0, args);
      let arg1 = get_arg::<i64>(value_store, 1, args);
      value_store.get_current_scope_mut().set(dest, Value::Bool(arg0 > arg1));
    }
    Le => {
      let arg0 = get_arg::<i64>(value_store, 0, args);
      let arg1 = get_arg::<i64>(value_store, 1, args);
      value_store.get_current_scope_mut().set(dest, Value::Bool(arg0 <= arg1));
    }
    Ge => {
      let arg0 = get_arg::<i64>(value_store, 0, args);
      let arg1 = get_arg::<i64>(value_store, 1, args);
      value_store.get_current_scope_mut().set(dest, Value::Bool(arg0 >= arg1));
    }
    Not => {
      let arg0 = get_arg::<bool>(value_store, 0, args);
      value_store.get_current_scope_mut().set(dest, Value::Bool(!arg0));
    }
    And => {
      let arg0 = get_arg::<bool>(value_store, 0, args);
      let arg1 = get_arg::<bool>(value_store, 1, args);
      value_store.get_current_scope_mut().set(dest, Value::Bool(arg0 && arg1));
    }
    Or => {
      let arg0 = get_arg::<bool>(value_store, 0, args);
      let arg1 = get_arg::<bool>(value_store, 1, args);
      value_store.get_current_scope_mut().set(dest, Value::Bool(arg0 || arg1));
    }
    Id => {
      let src = get_value(value_store, 0, args).clone();
      value_store.get_current_scope_mut().set(dest, src);
    }
    Fadd => {
      let arg0 = get_arg::<f64>(value_store, 0, args);
      let arg1 = get_arg::<f64>(value_store, 1, args);
      value_store.get_current_scope_mut().set(dest, Value::Float(arg0 + arg1));
    }
    Fmul => {
      let arg0 = get_arg::<f64>(value_store, 0, args);
      let arg1 = get_arg::<f64>(value_store, 1, args);
      value_store.get_current_scope_mut().set(dest, Value::Float(arg0 * arg1));
    }
    Fsub => {
      let arg0 = get_arg::<f64>(value_store, 0, args);
      let arg1 = get_arg::<f64>(value_store, 1, args);
      value_store.get_current_scope_mut().set(dest, Value::Float(arg0 - arg1));
    }
    Fdiv => {
      let arg0 = get_arg::<f64>(value_store, 0, args);
      let arg1 = get_arg::<f64>(value_store, 1, args);
      value_store.get_current_scope_mut().set(dest, Value::Float(arg0 / arg1));
    }
    Feq => {
      let arg0 = get_arg::<f64>(value_store, 0, args);
      let arg1 = get_arg::<f64>(value_store, 1, args);
      value_store.get_current_scope_mut().set(dest, Value::Bool(arg0 == arg1));
    }
    Flt => {
      let arg0 = get_arg::<f64>(value_store, 0, args);
      let arg1 = get_arg::<f64>(value_store, 1, args);
      value_store.get_current_scope_mut().set(dest, Value::Bool(arg0 < arg1));
    }
    Fgt => {
      let arg0 = get_arg::<f64>(value_store, 0, args);
      let arg1 = get_arg::<f64>(value_store, 1, args);
      value_store.get_current_scope_mut().set(dest, Value::Bool(arg0 > arg1));
    }
    Fle => {
      let arg0 = get_arg::<f64>(value_store, 0, args);
      let arg1 = get_arg::<f64>(value_store, 1, args);
      value_store.get_current_scope_mut().set(dest, Value::Bool(arg0 <= arg1));
    }
    Fge => {
      let arg0 = get_arg::<f64>(value_store, 0, args);
      let arg1 = get_arg::<f64>(value_store, 1, args);
      value_store.get_current_scope_mut().set(dest, Value::Bool(arg0 >= arg1));
    }
    Call => {
      let callee_func = prog
        .get(&funcs[0])
        .ok_or_else(|| InterpError::FuncNotFound(funcs[0].clone()))?;

      make_func_args(callee_func, args, value_store);

      let res = execute(prog, callee_func, out, value_store, heap, instruction_count)?.unwrap();
      value_store.pop();

      value_store.get_current_scope_mut().set(
        dest,
        res,
      );
    }
    Phi => {
      if last_label.is_none() {
        return Err(InterpError::NoLastLabel);
      } else {
        let arg = labels
          .iter()
          .position(|l| l == last_label.unwrap())
          .ok_or_else(|| InterpError::PhiMissingLabel(last_label.unwrap().to_string()))
          .map(|i| value_store.get_current_scope_mut().get(args.get(i).unwrap()))?
          .clone();
        value_store.get_current_scope_mut().set(dest, arg);
      }
    }
    Alloc => {
      let arg0 = get_arg::<i64>(value_store, 0, args);
      if heap.should_run_gc(arg0) {
        heap.gc(value_store);
      }
      let res = heap.alloc(arg0)?;
      value_store.get_current_scope_mut().set(dest, Value::Pointer(res))
    }
    Load => {
      let arg0 = get_arg::<&Pointer>(value_store, 0, args);
      let res = heap.read(arg0)?;
      value_store.get_current_scope_mut().set(dest, res.clone())
    }
    PtrAdd => {
      let arg0 = get_arg::<&Pointer>(value_store, 0, args);
      let arg1 = get_arg::<i64>(value_store, 1, args);
      let res = Value::Pointer(arg0.add(arg1));
      value_store.get_current_scope_mut().set(dest, res)
    }
  }
  Ok(())
}

// Returns a map from function parameter names to values of the call arguments
// that are bound to those parameters.
fn make_func_args<'a>(
  callee_func: &'a BBFunction,
  args: &[u32],
  vars: &mut Environment,
) {
  // todo: Having to allocate a new environment on each function call probably makes small function calls very heavy weight. This could be interesting to profile and see if old environments can be reused instead of being deallocated and reallocated. Maybe there is another way to sometimes avoid this allocation.
  let mut new_scope = Scope::new(callee_func.num_of_vars);

  args
    .iter()
    .zip(callee_func.args_as_nums.iter())
    .for_each(|(arg_name, expected_arg)| {
      let arg = vars.get_current_scope_mut().get(arg_name);
      new_scope.set(*expected_arg, arg.clone());
    });
    
  vars.append(new_scope);
}

// todo do this with less function arguments
#[inline(always)]
fn execute_effect_op<'a, T: std::io::Write>(
  prog: &'a BBProgram,
  func: &BBFunction,
  op: &bril_rs::EffectOps,
  args: &[u32],
  funcs: &[String],
  curr_block: &BasicBlock,
  out: &mut T,
  value_store: &mut Environment,
  heap: &mut Heap,
  next_block_idx: &mut Option<usize>,
  instruction_count: &mut u32,
) -> Result<Option<Value>, InterpError> {
  use bril_rs::EffectOps::*;
  match op {
    Jump => {
      *next_block_idx = Some(curr_block.exit[0]);
    }
    Branch => {
      let bool_arg0 = get_arg::<bool>(value_store, 0, args);
      let exit_idx = if bool_arg0 { 0 } else { 1 };
      *next_block_idx = Some(curr_block.exit[exit_idx]);
    }
    Return => match &func.return_type {
      Some(_) => {
        let arg0 = get_value(value_store, 0, args);
        return Ok(Some(arg0.clone()));
      }
      None => return Ok(None),
    },
    Print => {
      writeln!(
        out,
        "{}",
        args
          .iter()
          .map(|a| value_store.get_current_scope_mut().get(a).to_string())
          .collect::<Vec<String>>()
          .join(" ")
      )
      .map_err(|e| InterpError::IoError(Box::new(e)))?;
      out.flush().map_err(|e| InterpError::IoError(Box::new(e)))?;
    }
    Nop => {}
    Call => {
      let callee_func = prog
        .get(&funcs[0])
        .ok_or_else(|| InterpError::FuncNotFound(funcs[0].clone()))?;

      make_func_args(callee_func, args, value_store);

      execute(prog, callee_func, out, value_store, heap, instruction_count)?;

      value_store.pop();
    }
    Store => {
      let arg0 = get_arg::<&Pointer>(value_store, 0, args);
      let arg1 = get_value(value_store, 1, args);
      heap.write(arg0, arg1.clone())?
    }
    Free => {
      let arg0 = get_arg::<&Pointer>(value_store, 0, args);
      heap.free(arg0)?
    }
    Speculate | Commit | Guard => unimplemented!(),
  }
  Ok(None)
}

fn execute<'a, T: std::io::Write>(
  prog: &'a BBProgram,
  func: &'a BBFunction,
  out: &mut T,
  value_store: &mut Environment,
  heap: &mut Heap,
  instruction_count: &mut u32,
) -> Result<Option<Value>, PositionalInterpError> {
  // Map from variable name to value.
  let mut last_label;
  let mut current_label = None;
  let mut curr_block_idx = 0;
  let mut result = None;

  loop {
    let curr_block = &func.blocks[curr_block_idx];
    let curr_instrs = &curr_block.instrs;
    let curr_numified_instrs = &curr_block.numified_instrs;
    // WARNING!!! We can add the # of instructions at once because you can only jump to a new block at the end. This may need to be changed if speculation is implemented
    *instruction_count += curr_instrs.len() as u32;
    last_label = current_label;
    current_label = curr_block.label.as_ref();

    let mut next_block_idx = if curr_block.exit.len() == 1 {
      Some(curr_block.exit[0])
    } else {
      None
    };

    for (code, numified_code) in curr_instrs.iter().zip(curr_numified_instrs.iter()) {
      match code {
        Instruction::Constant {
          op: bril_rs::ConstOps::Const,
          dest: _,
          const_type,
          value,
          pos: _,
        } => {
          // Integer literals can be promoted to Floating point
          if const_type == &bril_rs::Type::Float {
            match value {
              bril_rs::Literal::Int(i) => {
                value_store.get_current_scope_mut().set(numified_code.dest.unwrap(), Value::Float(*i as f64))
              }
              bril_rs::Literal::Float(f) => {
                value_store.get_current_scope_mut().set(numified_code.dest.unwrap(), Value::Float(*f))
              }
              // this is safe because we type check this beforehand
              bril_rs::Literal::Bool(_) => unsafe { unreachable_unchecked() },
            }
          } else {
            value_store.get_current_scope_mut().set(numified_code.dest.unwrap(), Value::from(value));
          };
        }
        Instruction::Value {
          op,
          dest: _,
          op_type: _,
          args: _,
          labels,
          funcs,
          pos,
        } => {
          execute_value_op(
            prog,
            op,
            numified_code.dest.unwrap(),
            &numified_code.args,
            labels,
            funcs,
            out,
            value_store,
            heap,
            last_label,
            instruction_count,
          )
          .map_err(|e| e.add_pos(*pos))?;
        }
        Instruction::Effect {
          op,
          args: _,
          labels: _,
          funcs,
          pos,
        } => {
          result = execute_effect_op(
            prog,
            func,
            op,
            &numified_code.args,
            funcs,
            curr_block,
            out,
            value_store,
            heap,
            &mut next_block_idx,
            instruction_count,
          )
          .map_err(|e| e.add_pos(*pos))?;
        }
      }
    }
    if let Some(idx) = next_block_idx {
      curr_block_idx = idx;
    } else {
      return Ok(result);
    }
  }
}

fn parse_args(
  mut env: Environment,
  args: &[bril_rs::Argument],
  args_as_nums: &[u32],
  inputs: &[String],
) -> Result<Environment, InterpError> {
  if args.is_empty() && inputs.is_empty() {
    Ok(env)
  } else if inputs.len() != args.len() {
    Err(InterpError::BadNumFuncArgs(args.len(), inputs.len()))
  } else {
    args
      .iter()
      .zip(args_as_nums.iter())
      .enumerate()
      .try_for_each(|(index, (arg, arg_as_num))| match arg.arg_type {
        bril_rs::Type::Bool => {
          match inputs.get(index).unwrap().parse::<bool>() {
            Err(_) => {
              return Err(InterpError::BadFuncArgType(
                bril_rs::Type::Bool,
                (*inputs.get(index).unwrap()).to_string(),
              ))
            }
            Ok(b) => env.get_current_scope_mut().set(*arg_as_num, Value::Bool(b)),
          };
          Ok(())
        }
        bril_rs::Type::Int => {
          match inputs.get(index).unwrap().parse::<i64>() {
            Err(_) => {
              return Err(InterpError::BadFuncArgType(
                bril_rs::Type::Int,
                (*inputs.get(index).unwrap()).to_string(),
              ))
            }
            Ok(i) => env.get_current_scope_mut().set(*arg_as_num, Value::Int(i)),
          };
          Ok(())
        }
        bril_rs::Type::Float => {
          match inputs.get(index).unwrap().parse::<f64>() {
            Err(_) => {
              return Err(InterpError::BadFuncArgType(
                bril_rs::Type::Float,
                (*inputs.get(index).unwrap()).to_string(),
              ))
            }
            Ok(f) => env.get_current_scope_mut().set(*arg_as_num, Value::Float(f)),
          };
          Ok(())
        }
        // this is safe because there is no possible way to pass a pointer as an argument
        bril_rs::Type::Pointer(..) => unsafe { unreachable_unchecked() },
      })?;
    Ok(env)
  }
}

/// The entrance point to the interpreter. It runs over a ```prog```:[`BBProgram`] starting at the "main" function with ```input_args``` as input. Print statements output to ```out``` which implements [std::io::Write]. You also need to include whether you want the interpreter to count the number of instructions run with ```profiling```. This information is outputted to [std::io::stderr]
// todo we could probably output the profiling thing to a user defined location. If the program can output to a file, you should probably also be allowed to output this debug info to a file as well.
pub fn execute_main<T: std::io::Write>(
  prog: &BBProgram,
  mut out: T,
  input_args: &[String],
  profiling: bool,
) -> Result<(), PositionalInterpError> {
  let main_func = prog
    .get("main")
    .ok_or_else(|| PositionalInterpError::new(InterpError::NoMainFunction))?;

  if main_func.return_type.is_some() {
    return Err(InterpError::NonEmptyRetForFunc(main_func.name.clone()))
      .map_err(|e| e.add_pos(main_func.pos));
  }

  let scope = Scope::new(main_func.num_of_vars);
  let env = Environment::new(scope);
  let mut heap = Heap::default();

  let mut value_store = parse_args(env, &main_func.args, &main_func.args_as_nums, input_args)
    .map_err(|e| e.add_pos(main_func.pos))?;

  let mut instruction_count = 0;

  execute(
    prog,
    main_func,
    &mut out,
    &mut value_store,
    &mut heap,
    &mut instruction_count,
  )?;

  // if !heap.is_empty() {
  //   return Err(InterpError::MemLeak).map_err(|e| e.add_pos(main_func.pos));
  // }

  if profiling {
    eprintln!("total_dyn_inst: {instruction_count}");
  }

  Ok(())
}
