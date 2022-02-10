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




fn match_call(instruction: &Instruction) -> Option<&llvm_ir::instruction::Call> {
    match instruction {
	Instruction::Call(call) => Some(call),
	_ => None,
    }
}

fn match_function(func: &Either<InlineAssembly, Operand>) -> Option<&llvm_ir::operand::Operand> {
    match func {
	Either::Right(operand) => Some(operand),
	_ => None,
    }
}

fn match_operand(operand: &Operand) -> Option<&llvm_ir::constant::ConstantRef> {
    match operand {
	Operand::ConstantOperand(const_ref) => Some(const_ref),
	_ => None,
    }
}

// fn match_globalref(const_ref: &Constant) -> Option<&llvm_ir::Constant> {
//     match const_ref {
// 	llvm_ir::Constant::GlobalReference(global_ref) => Some(global_ref),
// 	_ => None,
//     }
// }


// fn match_globalref()


fn match_to_optype(qir_optype: &str) -> Option<OpType> {

    match qir_optype {
	"h" => Some(OpType::H),
	"cnot" => Some(OpType::CX),
	"mz" => Some(OpType::Measure),
	_ => None,
    }

}

fn to_command(instruction: &Instruction) {
    let mut func_signature = String::new();
    if let Instruction::Call(call) = instruction {
	// println!("{}", call);
	if let Either::Right(operand) = &call.function {
	    // println!("{}", operand);
	    func_signature = operand.to_string();
	}
    }

    let split_signature: Vec<&str> = func_signature.split("__").collect();
    // println!("{:?}", split_signature);

    let optype = match_to_optype(split_signature[3]).unwrap();

    println!("{:?}", optype);

    let mut args = String::new();
    if let Instruction::Call(call) = instruction {
	// println!("{}", call);
	let arguments = &call.arguments;
	if let Operand::ConstantOperand(operand) = &arguments[0].0 {
	    // println!("{}", operand);
	    args = operand.to_string();
	}
    }

    println!("{:?}", args)
}

fn main() {
    let circ_s = r#"{"bits": [["c", [0]], ["c", [1]]], "commands": [{"args": [["q", [0]]], "op": {"type": "H"}}, {"args": [["q", [0]], ["q", [1]]], "op": {"type": "CX"}}, {"args": [["q", [0]], ["c", [0]]], "op": {"type": "Measure"}}, {"args": [["q", [1]], ["c", [1]]], "op": {"type": "Measure"}}], "implicit_permutation": [[["q", [0]], ["q", [0]]], [["q", [1]], ["q", [1]]]], "phase": "0.0", "qubits": [["q", [0]], ["q", [1]]]}"#;
    let p: circuit::Circuit = serde_json::from_str(circ_s).unwrap();

    // dbg!(p);
    // println!("{:?}", module.functions)
    // write_circ_to_file(&p, "dump.ll");
    // dbg!(get_ir_string(&p));

    let module = Module::from_bc_path("./example_files/SimpleGroverBaseProfile.bc").unwrap();
    let first_function = &module.functions[0];
    let first_basicblock = &first_function.basic_blocks[0];

    let instructions = &first_basicblock.instrs;

    println!("{:?}", instructions[1]);

    to_command(&instructions[0]);

    
    // let first_instruction = &first_basicblock.instrs[1];
    // let call = match_call(first_instruction);
    
    // println!("{:?}", call.unwrap());
    // let call_function = &call.unwrap().function;
    // println!("{:?}", call_function);

    // let operand = match_function(call_function).unwrap();
    // let const_ref = match_operand(operand).unwrap();
    // let global_ref = &*const_ref.as_ref().to_string();
    // // println!("{:?}", global_ref);
    // // println!("{:?}", global_ref.to_string());
    
    // let op: Vec<&str> = global_ref.split("__").collect();
    // print!("{:?}", op[3]);
    
    // let params = &call.unwrap().arguments;
    // println!("{:?}", params[0].0);

    // let param_const_ref = match_operand(&params[0].0).unwrap();

    // println!("{}", param_const_ref);


    // let stuff: &str = &*param_const_ref.to_string();

    // println!("{}", stuff);
    
    // if stuff.contains("Qubit") {
    // 	println!("Qubit!");
    // }

    // if stuff.contains("null") {
    // 	println!("null !")
    // }

    // let register = circuit::Register("q".to_string(), vec![0]);
    // let register1 = register.clone();
    // let register2 = register.clone();
    // let circuit_qubits = vec![register];
    // println!("{:?}", circuit_qubits);
    // let circuit_bits: Vec<Register> = vec![];


    // let optype = circuit::OpType::H;
    // let op_register = circuit::Register("q".to_string(), vec![0]);
    // let op_args = vec![op_register];
    // let op = circuit::Operation{op_type: optype, n_qb: None, params: None, op_box: None, signature: None, conditional: None};
    // let command = circuit::Command{op: op, args: op_args, opgroup: None};
    // let commands = vec![command];

    // let phase = "0.0".to_string();
    
    // let implicit_permutation = vec![circuit::Permutation(register1, register2)];
    
    // let c = circuit::Circuit{
    // 	name: None,
    // 	phase: phase,
    // 	commands: commands,
    // 	qubits: circuit_qubits,
    // 	bits: circuit_bits,
    // 	implicit_permutation: implicit_permutation
    // };


    // let c_json = serde_json::to_string(&c);
    // println!("{:?}", c_json.unwrap());

    // serde_json::to_writer(&File::create("./data.json").unwrap(), &c);
   
}
