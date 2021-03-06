// use llvm_ir::Module;
// use llvm_ir::function::Function;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::FunctionValue;

use std::path::Path;

use llvm_ir::instruction::{Instruction, InlineAssembly};
use llvm_ir::operand::Operand;

use either::Either;

use crate::circuit::{OpType, Circuit};


pub trait ModuleExtension {
    fn get_func_by_name(&self, name: &str) -> Vec<&llvm_ir::Function>;

}

impl ModuleExtension for llvm_ir::Module {
    fn get_func_by_name(&self, name: &str) -> Vec<&llvm_ir::Function> {
	self.functions
	    .iter()
	    .filter(|f| {
		f.function_attributes.contains(
		    &llvm_ir::function::FunctionAttribute::StringAttribute {
			kind: name.to_string(),
			value: String::new(),
		    },
		)
	    })
	    .collect()
    }
}


pub trait FunctionExtension {
    fn get_attr_by_name(&self, name: &str) -> Option<String>;
    fn get_instr_by_name(&self, name: &str) -> Option<&llvm_ir::Instruction>;
}

impl FunctionExtension for llvm_ir::Function {
    fn get_attr_by_name(&self, name: &str) -> Option<String> {
	for attr in &self.function_attributes {
	    match attr {
		llvm_ir::function::FunctionAttribute::StringAttribute { kind, value } => {
                    if kind.to_string().eq(name) {
                        Some(value.to_string());
                    }
                },
                _ => continue,
	    }
	}
	None
    }

    fn get_instr_by_name(&self, name: &str) -> Option<&llvm_ir::Instruction> {
	for block in &self.basic_blocks {
	    // println!("{:?}", block);
	    for instr in &block.instrs {
		// println!("{:?}", instr);
		// match instr.try_get_result() {
		//     Some(idestname) => {
		// 	println!("idestname {:?}", idestname);
		// 	if idestname.to_string().eq(&name.to_string()) {
		// 	    return Some(instr)
		// 	} 
		//     }
		//     None => continue
		// }
		match instr {
		    llvm_ir::Instruction::Call(call) => {
			match call.get_func_name() {
			    Some(func_name) => {
				if func_name.as_string().eq(name) {
				    return Some(instr)
				}
			    },
			    None => continue,
			}
			// println!("There is a match {:?}", call.function);
			// match &call.dest {
			//     Some(idestname) => {
			// 	println!("idestname {:?}", idestname);
			// 	if idestname.to_string().eq(&name.to_string()) {
			// 	    println!("Got it !");
			// 	   return Some(instr); 
			// 	}
			//     },
			//     None => continue

			// }
		    },
		    _ => continue,
		}
	    }
	}
	None
    }
    
}

pub trait CallExtension {
    fn get_func_name(&self) -> Option<llvm_ir::Name>;
    fn get_qubit_index(&self) -> Option<&u64>;
}

impl CallExtension for llvm_ir::instruction::Call {
    fn get_func_name(&self) -> Option<llvm_ir::Name> {
	match self.function.clone().right()? {
            llvm_ir::Operand::ConstantOperand(c) =>
		match c.as_ref() {
                    llvm_ir::constant::Constant::GlobalReference { name, ty: _ } => Some(name.clone()),
                    _ => None,
		},
            _ => None,
        }
    }
    fn get_qubit_index(&self) -> Option<&u64> {
	match self.arguments.as_slice() {
	    [first, ..] => {
		match &first.0 {
		    llvm_ir::Operand::ConstantOperand(const_op) => {
			match const_op.as_ref() {
			    llvm_ir::constant::Constant::IntToPtr (p) => {
				match p.operand.as_ref() {
				    llvm_ir::Constant::Int { bits: _, value } => Some(&value),
				    _ => None,
				    
				}
			    },
			    _ => None,
			}
		    },
		    _ => None,
		}
	    },
	    _ => unreachable!(),
        }
    }
}

pub trait NameExtension {
    fn as_string(&self) -> String;
}

impl NameExtension for llvm_ir::Name {
    fn as_string(&self) -> String {
	match &self {
	    llvm_ir::Name::Name(name) => name.to_string(),
	    llvm_ir::Name::Number(number) => number.to_string(),
	}
    }
}


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

    println!("{:?}", args);
}


// pub fn parse_bitcode_file(file_path: &Path) -> Result<Module, String> {
//     let context: Context = Context::create();
//     // let module: Module = match Module::from_bc_path(file_path) {
//     // 	Ok(module) => return Ok(module),
//     // 	Err(r) => return Err(format!("Parsing {} has failed.", file_path).to_string()),
//     // };
//     let module: Module = match Module::parse_bitcode_from_path(&file_path, &context) {
// 	Ok(module) => return Ok(module),
// 	Err(_) => return Err(format!("Parsing {} has failed.", file_path.display()).to_string()),
//     };
// }

pub fn parse_bitcode_file<'ctx>(file_path: &Path, context: &'ctx Context) -> Result<Module<'ctx>, String> {
    Module::parse_bitcode_from_path(&file_path, &context).map_err(|err| format!("Parsing {} has failed.", file_path.display()))
}


#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::BufReader;
    use std::path::Path;

    use llvm_ir::Module;
    use llvm_ir::instruction::Instruction;
    use llvm_ir::types::Typed;
    

    use crate::circuit;

    use super::*;

    #[test]
    fn test_get_overall_function_by_name() {
	let file_path = Path::new("example_files/SimpleGroverBaseProfile.bc");
	let module = Module::from_bc_path(file_path).expect("File not found.");

	let func_name = "Microsoft__Quantum__Samples__SimpleGrover__SearchForMarkedInput__Interop";
	let func = module.get_func_by_name(func_name).expect("Function not found.");

	assert_eq!(func.name, func_name);
    }
    
    #[test]
    fn test_get_first_instruction_by_name () {

	let file_path = Path::new("example_files/SimpleGroverBaseProfile.bc");
	let module = Module::from_bc_path(file_path).expect("File not found.");

	let func_name = "Microsoft__Quantum__Samples__SimpleGrover__SearchForMarkedInput__Interop";
	let func = module.get_func_by_name(func_name).expect("Function not found.");

	let first_instruction_name = "__quantum__qis__h__body";

	let first_instruction = func.get_instr_by_name(first_instruction_name).expect("Instruction not found.");
	
	println!("{:?}", first_instruction);
	match first_instruction {
	    llvm_ir::Instruction::Call(call) => {
		match call.get_func_name() {
		    Some(func_name) => {
			assert_eq!(func_name.as_string(), first_instruction_name.to_string())
		    },
		    _ => (),
		}
		
		// assert_eq!(call.get_in_name().expect("Call not found").as_string(), func_name);
	    },
	    _ => (),
	}
    }

    #[test]
    fn test_get_qubit_index() {
	
	let file_path = Path::new("example_files/SimpleGroverBaseProfile.bc");
	let module = Module::from_bc_path(file_path).expect("File not found.");

	let func_name = "Microsoft__Quantum__Samples__SimpleGrover__SearchForMarkedInput__Interop";
	let func = module.get_func_by_name(func_name).expect("Function not found.");

	let first_instruction_name = "__quantum__qis__h__body";

	let first_instruction = func.get_instr_by_name(first_instruction_name).expect("Instruction not found.");

	match first_instruction {
	    llvm_ir::Instruction::Call(call) => {
		let index = call.get_qubit_index();
		assert!(index.is_none())
	    },
	    _ => (),
	}

	let second_instruction_name = "__quantum__qis__x__body";
	let second_instruction = func.get_instr_by_name(second_instruction_name).expect("Instruction not ofund.");

	match second_instruction {
	    llvm_ir::Instruction::Call(call) => {
		let index = call.get_qubit_index().expect("Index not found.");
		assert_eq!(*index, 2);
	    },
	    _ => (),
	}
	
    }
    
    
    fn parse_simple_instruction() {

	let file_path = Path::new("example_files/SimpleGroverBaseProfile.bc");
	
	let module = Module::from_bc_path(file_path).expect("File not found.");
	let first_function = &module.functions[0];
	let first_basicblock = &first_function.basic_blocks[0];

	let instructions = &first_basicblock.instrs;

	println!("Instruction {:?}", instructions[1]);

	to_command(&instructions[0]);

	let first_instruction = &first_basicblock.instrs[1];
	let call = match_call(first_instruction);

	println!("Call {:?}", call.unwrap());
	let call_function = &call.unwrap().function;
	println!("Call_function {:?}", call_function);

	let operand = match_function(call_function).unwrap();
	let const_ref = match_operand(operand).unwrap();
	let global_ref = &*const_ref.as_ref().to_string();
	// println!("{:?}", global_ref);
	// println!("{:?}", global_ref.to_string());

	let op: Vec<&str> = global_ref.split("__").collect();
	print!("Op {:?}", op[3]);

	let params = &call.unwrap().arguments;
	println!("Params0 {:?}", params[0].0);

	let param_const_ref = match_operand(&params[0].0).unwrap();

	println!("Param_Const_Ref {}", param_const_ref);


	let stuff: &str = &*param_const_ref.to_string();

	println!("Stuff {}", stuff);

	if stuff.contains("Qubit") {
		println!("Qubit!");
	}

	if stuff.contains("null") {
		println!("null !")

	}
    }



    fn test_generate_simple_circuit() {
	// A register of qubits for the circuit
	let register = circuit::Register("q".to_string(), vec![0]);

	// Two clones for the implicit permutation
	let register1 = register.clone();
	let register2 = register.clone();

	// Filling out the qubit register while creating an empty bit register
	let circuit_qubits = vec![register];
	let circuit_bits: Vec<circuit::Register> = vec![];

	// Filling out the op type for simple H gate 
	let optype = circuit::OpType::H;
	let op_register = circuit::Register("q".to_string(), vec![0]);
	let op_args = vec![op_register];
	let op = circuit::Operation{
	    op_type: optype,
	    n_qb: None,
	    params: None,
	    op_box: None,
	    signature: None,
	    conditional: None
	};

	// Filling out the commands
	let command = circuit::Command{op: op, args: op_args, opgroup: None};
	let commands = vec![command];

	// Defining the global phase and implicit permutation
	let phase = "0.0".to_string();
	let implicit_permutation = vec![circuit::Permutation(register1, register2)];

	// Creating the circuit with all previously defined parameters
	let circuit = circuit::Circuit{
	    name: None,
	    phase: phase,
	    commands: commands,
	    qubits: circuit_qubits,
	    bits: circuit_bits,
	    implicit_permutation: implicit_permutation
	};


	let circuit_json_str = serde_json::to_string(&circuit).unwrap();
	// println!("{:?}", c_json.unwrap());

	let file_path = Path::new("example_files/simple_H_pytket_circuit.json");
	let file = File::open(file_path).expect("File not found.");
	let reader = BufReader::new(file);
	let pytket_circuit: circuit::Circuit = serde_json::from_reader(reader).expect("Error while reading.");
	// serde_json::to_writer(&File::create("./data.json").unwrap(), &c);
	let pytket_circuit_str: String = serde_json::to_string(&pytket_circuit).unwrap();

	assert_eq!(circuit_json_str, pytket_circuit_str);
    }   

}
