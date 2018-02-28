extern crate binder;
extern crate lexer;
extern crate parser;
extern crate typed_arena;
extern crate vm;

#[cfg(test)]
mod test;

use binder::{BindingKind, Bound, DeclarationKind};
use vm::value::{new_func, Function, Value};
use vm::vm::{Instruction, Symbol};
use std::ops::Deref;

pub fn emit_top(node: &Bound) -> Value {
    match node {
        &Bound::Module { ref statements, .. } => {
            Value::Function(new_func(emit_function(statements, true)))
        }
        _ => panic!(),
    }
}

pub fn emit_function(statements: &[Bound], is_module: bool) -> Function {
    let mut instrs = vec![];
    for statement in &statements[..statements.len() - 1] {
        if emit(statement, &mut instrs) {
            instrs.push(Instruction::Pop);
        }
    }
    if let Some(last_expression) = statements.last() {
        if emit(last_expression, &mut instrs) && is_module {
            instrs.push(Instruction::Pop);
        }
    }

    Function {
        arg_count: 0,
        name: None,
        instructions: instrs,
    }
}

pub fn emit(node: &Bound, out: &mut Vec<Instruction>) -> bool {
    fn emit_binary(
        left: &Bound,
        right: &Bound,
        out: &mut Vec<Instruction>,
        instruction: Instruction,
    ) -> bool {
        assert!(emit(left, out));
        assert!(emit(right, out));
        out.push(instruction);
        true
    }

    match node {
        &Bound::Integer { value, .. } => {
            out.push(Instruction::Push(Value::Integer(value)));
            true
        }
        &Bound::Float { value, .. } => {
            out.push(Instruction::Push(Value::Float(value)));
            true
        }
        &Bound::Add {
            ref left,
            ref right,
            ..
        } => emit_binary(left, right, out, Instruction::Add),
        &Bound::Sub {
            ref left,
            ref right,
            ..
        } => emit_binary(left, right, out, Instruction::Sub),
        &Bound::Mul {
            ref left,
            ref right,
            ..
        } => emit_binary(left, right, out, Instruction::Mul),
        &Bound::Div {
            ref left,
            ref right,
            ..
        } => emit_binary(left, right, out, Instruction::Div),
        &Bound::BlockExpr {
            ref statements,
            ref final_expression,
            ..
        } => {
            for statement in statements {
                if emit(statement, out) {
                    out.push(Instruction::Pop);
                }
            }
            assert!(emit(final_expression, out));
            true
        }
        &Bound::FieldAccess {
            ref target,
            field_name,
            ..
        } => {
            assert!(emit(target, out));
            out.push(Instruction::Push(Value::Symbol(Symbol(field_name.into()))));
            out.push(Instruction::MapGet);
            true
        }
        &Bound::Identifier {
            binding_kind:
                BindingKind::Module {
                    module_id,
                    ref symbol,
                },
            ..
        } => {
            let stringed = match symbol.deref() {
                &DeclarationKind::Named(s) => s.into(),
                &DeclarationKind::Generated(n, s) => format!("{}${}", s, n),
            };
            out.push(Instruction::Push(Value::Symbol(Symbol(module_id.into()))));
            out.push(Instruction::Push(Value::Symbol(Symbol(stringed))));
            out.push(Instruction::ModuleAdd);
            true
        }
        &Bound::VariableDecl {
            ref expression,
            ref location,
            ..
        } => {
            assert!(emit(expression, out));

            match location {
                &BindingKind::FunctionLocal(_local_idx) => unimplemented!(),
                &BindingKind::Argument(_arg_index) => unimplemented!(),
                &BindingKind::Upvar(_upvar_index) => unimplemented!(),
                &BindingKind::Module {
                    module_id,
                    ref symbol,
                } => {
                    let stringed = match symbol.deref() {
                        &DeclarationKind::Named(s) => s.into(),
                        &DeclarationKind::Generated(n, s) => format!("{}${}", s, n),
                    };
                    out.push(Instruction::Push(Value::Symbol(Symbol(module_id.into()))));
                    out.push(Instruction::Push(Value::Symbol(Symbol(stringed))));
                    out.push(Instruction::ModuleAdd);
                }
            }

            false
        }
        _ => unimplemented!(),
    }
}
