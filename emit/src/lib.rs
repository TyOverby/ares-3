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

            instructions.push(Instruction::MapEmpty);
            instructions.push(Instruction::Ret);

            Value::Function(new_func(Function {
                instructions,
                upvars: vec![],
                name: None,
                args_count: 0,
                upvars_count: 0,
                locals_count: 0,
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
        &Bound::DebugCall { arg, .. } => {
            assert!(emit(arg, out, current_function));
            out.push(Instruction::Debug);
            false
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
        &Bound::Identifier {
            ref binding_kind, ..
        } => {
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
        &Bound::FunctionCall {
            ref target,
            ref args,
            ..
        } => {
            assert!(emit(target, out, current_function));
            for arg in args {
                assert!(emit(arg, out, current_function));
            }
            out.push(Instruction::Call(args.len() as u32));
            true
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

            let mut instrs = vec![];
            assert!(emit(body, &mut instrs, Some(fn_info)));
            instrs.push(Instruction::Ret);

            let function_value = Value::Function(new_func(Function {
                instructions: instrs,
                upvars: vec![],
                name: Some(name.into()),
                args_count: params.len() as u32,
                upvars_count: upvars.len() as u32,
                locals_count: locals.len() as u32,
            }));

            for (_, &(ref upvar, _)) in upvars {
                if let Some(ref fi) = current_function {
                    fi.emit_binding_kind_getter(upvar, out);
                } else {
                    fallback_emit_binding_kind_getter(upvar, out);
                }
            }

            out.push(Instruction::Push(function_value));
            out.push(Instruction::BuildFunction);

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
