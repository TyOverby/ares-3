extern crate colored;
extern crate linenoise;
extern crate repl;
extern crate vm;

use colored::*;
use vm::value::new_func;
use vm::value::Function;
use repl::{ReplOutKind, StorableModuleBinder};

fn main() {
    linenoise::set_multiline(3);

    let mut vm = vm::vm::Vm::new();
    let mut storable_mod_binder = StorableModuleBinder {
        name: "repl-module".into(),
        definitions: Default::default(),
    };

    let mut buildup = String::new();
    loop {
        let pre_string = if buildup.len() > 0 {
            "----> "
        } else {
            "ares> "
        };

        while let Some(input) = linenoise::input(&format!("{}", pre_string.cyan())) {
            buildup.push_str(&input);
            buildup.push('\n');
            match repl::run(&buildup, &mut vm, storable_mod_binder.clone()) {
                Ok((ReplOutKind::Expression(s), new_mod)) => {
                    storable_mod_binder = new_mod;
                    let s_green = format!("{:?}", s).green();
                    println!("{}", s_green);
                }
                Ok((ReplOutKind::Statement(_), new_mod)) => {
                    storable_mod_binder = new_mod;
                }
                Err(s) => println!("{}", s.red()),
            }
            buildup.clear();
        }

        if buildup.is_empty() {
            break;
        }


        buildup.clear();
    }
}
