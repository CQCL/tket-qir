#![allow(warnings, unused)]
mod array1d;
mod basic_values;
mod circuit;

mod emit;
mod parse;

use emit::{get_ir_string, write_circ_to_file};
use llvm_ir::Function;
use llvm_ir::Module;
use llvm_ir::instruction;
use llvm_ir::instruction::Instruction;
use llvm_ir::operand::Operand;
use llvm_ir::instruction::InlineAssembly;
use llvm_ir::constant::ConstantRef;
use llvm_ir::constant::Constant;


use parse::parse_bitcode_file;

use std::path::Path;

use inkwell::context::Context;
use inkwell::values::FunctionValue;

use qirlib::intrinsics::Intrinsics;
use qirlib::module;
use qirlib::codegen::CodeGenerator;
use qirlib::types::Types;
use qirlib::module::load_file;

use either::Either;

use crate::circuit::OpType;
use crate::circuit::Register;

use std::fs::{File, OpenOptions};

use std::io::Write;


// struct InnerFunctionValue<'a>(FunctionValue<'a>);

// impl Iterator for InnerFunctionValue {
//     type Item<'a> = FunctionValue<'a>;
    
//     fn next(&mut self) -> Option<Self::Item> {
// 	Some(self.get_next_function())
//     }
// }

// fn match_globalref(const_ref: &Constant) -> Option<&llvm_ir::Constant> {
//     match const_ref {
// 	llvm_ir::Constant::GlobalReference(global_ref) => Some(global_ref),
// 	_ => None,
//     }
// }


// fn match_globalref()


fn main() {
    let circ_s = r#"{"bits": [["c", [0]], ["c", [1]]], "commands": [{"args": [["q", [0]]], "op": {"type": "H"}}, {"args": [["q", [0]], ["q", [1]]], "op": {"type": "CX"}}, {"args": [["q", [0]], ["c", [0]]], "op": {"type": "Measure"}}, {"args": [["q", [1]], ["c", [1]]], "op": {"type": "Measure"}}], "implicit_permutation": [[["q", [0]], ["q", [0]]], [["q", [1]], ["q", [1]]]], "phase": "0.0", "qubits": [["q", [0]], ["q", [1]]]}"#;
    let p: circuit::Circuit = serde_json::from_str(circ_s).unwrap();

    // dbg!(p);
    // println!("{:?}", module.functions)
    // write_circ_to_file(&p, "dump.ll");
    // dbg!(get_ir_string(&p));

    

   
}
