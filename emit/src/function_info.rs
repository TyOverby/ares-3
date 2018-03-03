use super::*;
use std::ops::Deref;

// Function Local layouts
// [args]
// [upvars]
// [locals]
// [..scratch space..]
#[derive(Clone, Copy)]
pub struct FunctionInfo {
    pub args_count: u32,
    pub upvars_count: u32,
    pub locals_count: u32,
}

fn fallback_emit_binding_kind_prelude(binding_kind: &BindingKind, out: &mut Vec<Instruction>) {
    match binding_kind {
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
        }
        _ => panic!(),
    }
}

pub fn fallback_emit_binding_kind_getter(binding_kind: &BindingKind, out: &mut Vec<Instruction>) {
    fallback_emit_binding_kind_prelude(binding_kind, out);
    out.push(Instruction::ModuleGet);
}

pub fn fallback_emit_binding_kind_setter(binding_kind: &BindingKind, out: &mut Vec<Instruction>) {
    fallback_emit_binding_kind_prelude(binding_kind, out);
    out.push(Instruction::ModuleAdd);
}

impl FunctionInfo {
    pub fn emit_binding_kind_getter(&self, binding_kind: &BindingKind, out: &mut Vec<Instruction>) {
        match binding_kind {
            &BindingKind::Argument(arg_index) => {
                out.push(Instruction::GetFromStackPosition(arg_index));
            }
            &BindingKind::Upvar(upvar_index) => {
                out.push(Instruction::GetFromStackPosition(
                    self.args_count + upvar_index,
                ));
            }
            &BindingKind::FunctionLocal(local_idx) => {
                out.push(Instruction::GetFromStackPosition(
                    self.args_count + self.upvars_count + local_idx,
                ));
            }
            &BindingKind::Module { .. } => {
                fallback_emit_binding_kind_getter(binding_kind, out);
            }
        }
    }

    pub fn emit_binding_kind_setter(&self, binding_kind: &BindingKind, out: &mut Vec<Instruction>) {
        match binding_kind {
            &BindingKind::Argument(arg_index) => {
                out.push(Instruction::SetToStackPosition(arg_index));
            }
            &BindingKind::Upvar(upvar_index) => {
                out.push(Instruction::SetToStackPosition(
                    self.args_count + upvar_index,
                ));
            }
            &BindingKind::FunctionLocal(local_idx) => {
                out.push(Instruction::SetToStackPosition(
                    self.args_count + self.upvars_count + local_idx,
                ));
            }
            &BindingKind::Module { .. } => {
                fallback_emit_binding_kind_setter(binding_kind, out);
            }
        }
    }
}
