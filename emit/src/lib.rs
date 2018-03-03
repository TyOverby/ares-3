extern crate binder;
extern crate lexer;
extern crate parser;
extern crate typed_arena;
extern crate vm;

#[cfg(test)]
mod test;
mod function_info;

use binder::{BindingKind, Bound, DeclarationKind};
use vm::value::{new_func, Function, Value};
use vm::vm::{Instruction, Symbol};
use function_info::*;

pub fn emit_top(node: &Bound) -> Value {
    match node {
        &Bound::Module { ref statements, .. } => {
            let mut instructions = vec![];
            for statement in statements {
                if emit(statement, &mut instructions, None) {
                    instructions.push(Instruction::Pop);
                }
            }

            Value::Function(new_func(Function {
                instructions,
                name: None,
                arg_count: 0,
            }))
        }
        _ => panic!(),
    }
}

pub fn emit(
    node: &Bound,
    out: &mut Vec<Instruction>,
    current_function: Option<FunctionInfo>,
) -> bool {
    fn emit_binary(
        left: &Bound,
        right: &Bound,
        out: &mut Vec<Instruction>,
        instruction: Instruction,
        current_function: Option<FunctionInfo>,
    ) -> bool {
        assert!(emit(left, out, current_function));
        assert!(emit(right, out, current_function));
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
        } => emit_binary(left, right, out, Instruction::Add, current_function),
        &Bound::Sub {
            ref left,
            ref right,
            ..
        } => emit_binary(left, right, out, Instruction::Sub, current_function),
        &Bound::Mul {
            ref left,
            ref right,
            ..
        } => emit_binary(left, right, out, Instruction::Mul, current_function),
        &Bound::Div {
            ref left,
            ref right,
            ..
        } => emit_binary(left, right, out, Instruction::Div, current_function),
        &Bound::BlockExpr {
            ref statements,
            ref final_expression,
            ..
        } => {
            for statement in statements {
                if emit(statement, out, current_function) {
                    out.push(Instruction::Pop);
                }
            }
            assert!(emit(final_expression, out, current_function));
            true
        }
        &Bound::FieldAccess {
            ref target,
            field_name,
            ..
        } => {
            assert!(emit(target, out, current_function));
            out.push(Instruction::Push(Value::Symbol(Symbol(field_name.into()))));
            out.push(Instruction::MapGet);
            true
        }
        &Bound::Identifier { ref binding_kind, .. } => {
            if let Some(ref fi) = current_function {
                fi.emit_binding_kind_getter(binding_kind, out);
            } else {
                fallback_emit_binding_kind_getter(binding_kind, out);
            }
            true
        }
        &Bound::VariableDecl {
            ref expression,
            ref location,
            ..
        } => {
            assert!(emit(expression, out, current_function));

            if let Some(ref fi) = current_function {
                fi.emit_binding_kind_setter(location, out);
            } else {
                fallback_emit_binding_kind_setter(location, out);
            }

            false
        }
        &Bound::FunctionDecl {
            ref params,
            ref body,
            ref locals,
            ref upvars,
            ref location,
            name,
            ..
        } => {
            let fn_info = FunctionInfo {
                args_count: params.len() as u32,
                upvars_count: upvars.len() as u32,
                locals_count: locals.len() as u32,
            };

            if !upvars.is_empty() {
                panic!();
            }

            let mut instrs = vec![];
            assert!(emit(body, &mut instrs, Some(fn_info)));

            let function_value = Value::Function(new_func(Function {
                instructions: instrs,
                name: Some(name.into()),
                arg_count: params.len(),
            }));

            out.push(Instruction::Push(function_value));

            if let Some(ref fi) = current_function {
                fi.emit_binding_kind_setter(location, out);
            } else {
                fallback_emit_binding_kind_setter(location, out);
            }

            false
        }
        other => unimplemented!("emit({:?}) is not implemented", other),
    }
}
