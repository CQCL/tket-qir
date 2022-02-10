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
    let first_instruction = &first_basicblock.instrs[0];
    // let function_call = &first_instruction;
    // match first_instruction {
    // 	Instruction::Call(call) => println!("{:?}", call),
    // 	_ => (),
    // }
    // println!("{:?}", first_instruction);
    let call = match_call(first_instruction);
    println!("{:?}", call.unwrap());
    let call_function = &call.unwrap().function;
    println!("{:?}", call_function);

    let operand = match_function(call_function).unwrap();
    let const_ref = match_operand(operand).unwrap();
    let global_ref = &*const_ref.as_ref().to_string();
    // println!("{:?}", global_ref);
    // println!("{:?}", global_ref.to_string());
    
    let op: Vec<&str> = global_ref.split("__").collect();
    print!("{:?}", op[3]);

    
    let params = &call.unwrap().arguments;
    println!("{:?}", params[0].0);

    let param_const_ref = match_operand(&params[0].0).unwrap();

    println!("{}", param_const_ref);


    let stuff: &str = &*param_const_ref.to_string();

    println!("{}", stuff);
    
    if stuff.contains("Qubit") {
	println!("Qubit!");
    }

    if stuff.contains("null") {
	println!("null !")
    }

    /* println!("{:?}", global_ref) */
    // let global_ref = match_constref(const_ref);

    // let module = Module::from_bc_path("./dump.bc").unwrap();
   
    // let path = Path::new("./example_files/SimpleGroverBaseProfile.bc");
    // let context: Context = Context::create();
    // // let module = module::load_template("./example_files/SimpleGroverBaseProfile.bc", &context).unwrap();
    // let module = load_file(&path, &context).unwrap();
    // // let first_func = module.get_first_function();
    // // println!("{:?}", first_func);
    // // let module = parse_file("./example_files/SimpleGroverBaseProfile.bc").unwrap();
    // // let struct_names = module.types.all_struct_names();
    // // for names in struct_names {
    // // 	println!("{}", names);
    // // }
    // let generator = CodeGenerator::new(&context, module).unwrap();
    // let module = generator.module;
    // // println!("{:?}", module.get_first_function().unwrap());
    // let first_function = module.get_first_function().unwrap();
    // // println!("{:?}", first_function);
    // let first_basicblock = first_function.get_first_basic_block().unwrap();
    // // println!("{:?}", first_basicblock);
    // let first_instruction = first_basicblock.get_first_instruction().unwrap();
    // let num_operands = first_instruction.get_num_operands();
    // println!("{:?}", num_operands);
    // for operand in 0..num_operands {
    // 	println!("{:?}", first_instruction.get_operand(operand));
    // }
    // let next_instruction = first_instruction.get_next_instruction().unwrap();
    // println!("{:?}", next_instruction)
    
    // inner_functionvalue = InnerFunctionValue(module.get_first_fun/* c */tion().unwrap())
    // println!("{:?}",first_function.get_next_function().unwrap());
    // let intrinsics = Intrinsics::new(&module);
    // println!("{:?}", intrinsics.h_ins);
    // let intrinsics = Intrinsics::new(&generator.module);
    // println!("{:?}", intrinsics.h);
    // let types = Types::new(generator.context, &generator.module);
    // println!("{}", types.qubit.size_of())
   
}
