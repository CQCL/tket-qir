// use llvm_ir::Module;
// use llvm_ir::function::Function;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::FunctionValue;

use std::path::Path;

use llvm_ir::instruction::{Instruction, InlineAssembly};
use llvm_ir::operand::Operand;

use either::Either;

use crate::circuit::{OpType, Operation, Circuit, Command, Register, Permutation};


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
    fn get_nb_qubits(&self) -> i64;
    fn get_nb_bits(&self) -> i64;
}

impl FunctionExtension for llvm_ir::Function {
    fn get_attr_by_name(&self, name: &str) -> Option<String> {
	for attr in &self.function_attributes {
	    match attr {
		llvm_ir::function::FunctionAttribute::StringAttribute { kind, value } => {
                    if kind.eq(name) {
                        return Some(value.to_string());
                    }
                },
                _ => continue,
	    }
	}
	None
    }

    fn get_instr_by_name(&self, name: &str) -> Option<&llvm_ir::Instruction> {
	for block in &self.basic_blocks {
	    for instr in &block.instrs {
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
		    },
		    _ => continue,
		}
	    }
	}
	None
    }

    fn get_nb_qubits(&self) -> i64 {
	self.get_attr_by_name("requiredQubits").expect("No qubits found.").parse::<i64>().unwrap()
    }
    fn get_nb_bits(&self) -> i64 {
	if let Some(bits) = self.get_attr_by_name("requiredResults") {
	    bits.parse::<i64>().unwrap()
	} else {
	    0
	}
    }
}

pub trait BasicBlockExtension {
    fn to_circuit(&self, nb_qubits: i64, nb_bits: i64) -> Circuit;
}

impl BasicBlockExtension for llvm_ir::BasicBlock {
    fn to_circuit(&self, nb_qubits: i64, nb_bits: i64) -> Circuit {
	// Classical and quantum registers for the circuit
	let mut circuit_qubit_register: Vec<Register> = vec![];
	let mut circuit_bit_register: Vec<Register> = vec![];
	for qubit in 0..nb_qubits {
	    circuit_qubit_register.push(Register("q".to_string(), vec![qubit]))
	}
	for bit in 0..nb_bits {
	    circuit_bit_register.push(Register("c".to_string(), vec![bit]))
	}
	let commands: Vec<Command> = self.instrs
	    .iter()
	    .map(|i| i.get_call().to_command().expect("Command not found."))
	    .collect();
    
	// Defining the global phase and implicit permutation
	let phase = "0.0".to_string();

	// Two clones for the implicit permutation
	let register0 = Register("q".to_string(), vec![0]);
	let implicit_permutation = vec![Permutation(register0.clone(), register0.clone())];

	// Creating the circuit with all previously defined parameters
	let circuit = Circuit{
	    name: None,
	    phase: phase,
	    commands: commands,
	    qubits: circuit_qubit_register,
	    bits: circuit_bit_register,
	    implicit_permutation: implicit_permutation
	};
	return circuit;
    }
} 

pub trait InstructionExtension {
    fn get_call(&self) -> &llvm_ir::instruction::Call;
}

impl InstructionExtension for llvm_ir::Instruction {
    fn get_call(&self) -> &llvm_ir::instruction::Call {
	match &self {
	    llvm_ir::Instruction::Call(call) => call,
	    _ => unreachable!(),
	}
    }
}


pub trait CallExtension {
    fn get_func_name(&self) -> Option<llvm_ir::Name>;
    fn get_qubit_indices(&self) -> Vec<i64>;
    fn get_bit_indices(&self) -> Option<Vec<i64>>;
    fn get_optype(&self) -> Option<OpType>;
    fn get_params(&self) -> Option<Vec<String>>; 
    fn get_operation(&self) -> Option<Operation>;
    fn to_command(&self) -> Option<Command>;
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
    fn get_qubit_indices(&self) -> Vec<i64> {

	let mut qubit_indices: Vec<i64> = vec![];

	self.arguments
	    .iter()
	    .for_each(|arg| if let llvm_ir::Operand::ConstantOperand(const_op) = &arg.0 {
		 match const_op.as_ref() {
		     llvm_ir::constant::Constant::IntToPtr (p) => {
			 if let llvm_ir::types::Type::PointerType { pointee_type: pt , addr_space: _} = p.to_type.as_ref() {
			     if let llvm_ir::types::Type::NamedStructType { name: name } = pt.as_ref() {
				 if name.eq("Qubit") {
				     let operand_ref = p.operand.as_ref();
				     if let llvm_ir::Constant::Int { bits: _, value } = operand_ref {
					 qubit_indices.push(*value as i64)
				     }
				 }				 
			     }
			 }
		     },
		     llvm_ir::constant::Constant::Null (p) => {

			 if let llvm_ir::types::Type::PointerType { pointee_type: pt , addr_space: _} = p.as_ref() {
			     if let llvm_ir::types::Type::NamedStructType { name: name } = pt.as_ref() {
				 if name.eq("Qubit") {
				     qubit_indices.push(0)
				 }				 
			     }
			 }

		     },
		     _ => (),
		 }
	    });
	return qubit_indices
    }

    fn get_bit_indices(&self) -> Option<Vec<i64>> {
	
	let mut bit_indices: Vec<i64> = vec![];

	self.arguments
	    .iter()
	    .for_each(|arg| if let llvm_ir::Operand::ConstantOperand(const_op) = &arg.0 {
		// println!("const_op {:?}", const_op);
		match const_op.as_ref() {
		     llvm_ir::constant::Constant::IntToPtr (p) => {
			 // println!("p {:?}", p);
			 if let llvm_ir::types::Type::PointerType { pointee_type: pt , addr_space: _} = p.to_type.as_ref() {
			     if let llvm_ir::types::Type::NamedStructType { name: name } = pt.as_ref() {
				 if name.eq("Result") {
				     let operand_ref = p.operand.as_ref();
				     if let llvm_ir::Constant::Int { bits: _, value } = operand_ref {
					 bit_indices.push(*value as i64)
				     }

				 }				 
			     }
			 }
		     },
		    llvm_ir::constant::Constant::Null (p) => {
			// println!("null p {:?}", p);

			if let llvm_ir::types::Type::PointerType { pointee_type: pt , addr_space: _} = p.as_ref() {
			    // println!("pt {:?}", pt);
			    if let llvm_ir::types::Type::NamedStructType { name: name } = pt.as_ref() {
				if name.eq("Result") {
				    bit_indices.push(0)
				}				 
			    }
			}
		    },
		     _ => (),
		 }
	    });

	if !bit_indices.is_empty() {
	    return Some(bit_indices)
	}
	None
    }

    
    fn get_optype(&self) -> Option<OpType> {
	let func_name = self.get_func_name().expect("No name found.").as_string();
	if func_name.contains("__h__") {
	   Some(OpType::H)
	}
	else if func_name.contains("__x__") {
	    Some(OpType::X)
	}
	else if func_name.contains("__y__") {
	    Some(OpType::Y)
	}
	else if func_name.contains("__z__") {
	    Some(OpType::Z)
	}
	else if func_name.contains("__t__body") {
	    Some(OpType::T)
	}
	else if func_name.contains("__t__adj") {
	    Some(OpType::Tdg)
	}
	else if func_name.contains("__cnot__") {
	    Some(OpType::CX)
	}
	else if func_name.contains("__rx__") {
	    Some(OpType::Rx)
	}
	else if func_name.contains("__ry__") {
	    Some(OpType::Ry)
	}
	else if func_name.contains("__rz__") {
	    Some(OpType::Rz)
	}
	else if func_name.contains("__mz__") {
	    Some(OpType::Measure)
	}
	else {
	    None
	}
    }
    fn get_params(&self) -> Option<Vec<String>> {
	
	// println!("Args {:?}", self.arguments);
	let mut params: Vec<String> = vec![];

	self.arguments
	    .iter()
	    .for_each(|arg| if let llvm_ir::Operand::ConstantOperand(const_op) = &arg.0 {
		let const_op_ref = const_op.as_ref();
		if let llvm_ir::constant::Constant::Float (f) = const_op_ref {
		    match f {
			llvm_ir::constant::Float::Double(d) => {
			    let param = *d;
			    params.push(param.to_string());
			},
			_ => ()
		    }
		}
	    });

	if !params.is_empty() {
	    return Some(params)
	}
	None
    }
    fn get_operation(&self) -> Option<Operation> {
	let op_type = self.get_optype();
	let params = self.get_params();
	match op_type {
	    Some(optype) => {
		let op = Operation{
		    op_type: optype,
		    n_qb: None,
		    params: params,
		    op_box: None,
		    signature: None,
		    conditional: None
		};
		Some(op)
	    },
	    None => None,
	}
    }
    fn to_command(&self) -> Option<Command> {
	let op = self.get_operation().expect("Op not found.");
	let qubit_index = self.get_qubit_index();
	// let op_register = Register("q".to_string(), qubit_index);
	// let op_args = vec![op_register];

	let op_args: Vec<Register> = qubit_index
	    .iter()
	    .map(|qi| Register("q".to_string(), vec![*qi]))
	    .collect();



	
	// Filling out the commands
	let command = Command{op: op, args: op_args, opgroup: None};
	return Some(command);
	None
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


// fn match_call(instruction: &Instruction) -> Option<&llvm_ir::instruction::Call> {
//     match instruction {
// 	Instruction::Call(call) => Some(call),
// 	_ => None,
//     }
// }

// fn match_function(func: &Either<InlineAssembly, Operand>) -> Option<&llvm_ir::operand::Operand> {
//     match func {
// 	Either::Right(operand) => Some(operand),
// 	_ => None,
//     }
// }

// fn match_operand(operand: &Operand) -> Option<&llvm_ir::constant::ConstantRef> {
//     match operand {
// 	Operand::ConstantOperand(const_ref) => Some(const_ref),
// 	_ => None,
//     }
// }

// fn match_to_optype(qir_optype: &str) -> Option<OpType> {

//     match qir_optype {
// 	"h" => Some(OpType::H),
// 	"cnot" => Some(OpType::CX),
// 	"mz" => Some(OpType::Measure),
// 	_ => None,
//     }

// }

// fn to_command(instruction: &Instruction) {
//     let mut func_signature = String::new();
//     if let Instruction::Call(call) = instruction {
// 	// println!("{}", call);
// 	if let Either::Right(operand) = &call.function {
// 	    // println!("{}", operand);
// 	    func_signature = operand.to_string();
// 	}
//     }

//     let split_signature: Vec<&str> = func_signature.split("__").collect();
//     // println!("{:?}", split_signature);

//     let optype = match_to_optype(split_signature[3]).unwrap();

//     println!("{:?}", optype);

//     let mut args = String::new();
//     if let Instruction::Call(call) = instruction {
// 	// println!("{}", call);
// 	let arguments = &call.arguments;
// 	if let Operand::ConstantOperand(operand) = &arguments[0].0 {
// 	    // println!("{}", operand);
// 	    args = operand.to_string();
// 	}
//     }

//     println!("{:?}", args);
// }


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
    use serde_json::to_vec_pretty;
    

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
		assert_eq!(index, vec![0])
	    },
	    _ => (),
	}

	let second_instruction_name = "__quantum__qis__x__body";
	let second_instruction = func.get_instr_by_name(second_instruction_name).expect("Instruction not found.");

	match second_instruction {
	    llvm_ir::Instruction::Call(call) => {
		let index = call.get_qubit_index();
		assert_eq!(index, vec![2]);
	    },
	    _ => (),
	}

	let third_instruction_name = "__quantum__qis__cnot__body";
	let third_instruction_call = func.get_instr_by_name(third_instruction_name).expect("Instruction not found.").get_call();

	println!("{:?}", third_instruction_call.get_qubit_index())
	
    }

    #[test]
    fn test_get_optype() {
	
	let file_path = Path::new("example_files/SimpleGroverBaseProfile.bc");
	let module = Module::from_bc_path(file_path).expect("File not found.");

	let func_name = "Microsoft__Quantum__Samples__SimpleGrover__SearchForMarkedInput__Interop";
	let func = module.get_func_by_name(func_name).expect("Function not found.");

	let first_instruction = &func.basic_blocks[0].instrs[0];

	// println!("{:?}", first_instruction);

	match first_instruction {
	    llvm_ir::Instruction::Call(call) => {
		match call.get_optype() {
		    Some(optype) => {
			assert_eq!(optype, OpType::H)
		    },
		    _ => (),
		}
		
		// assert_eq!(call.get_in_name().expect("Call not found").as_string(), func_name);
	    },
	    _ => (),
	}	

    }

    #[test]
    fn test_get_op() {
	
	let file_path = Path::new("example_files/SimpleGroverBaseProfile.bc");
	let module = Module::from_bc_path(file_path).expect("File not found.");

	let func_name = "Microsoft__Quantum__Samples__SimpleGrover__SearchForMarkedInput__Interop";
	let func = module.get_func_by_name(func_name).expect("Function not found.");

	let first_call = &func.basic_blocks[0].instrs[0].get_call();

	let op = first_call.get_operation().expect("No op found.");

	let expected_op = Operation{
	    op_type: OpType::H,
	    n_qb: None,
	    params: None,
	    op_box: None,
	    signature: None,
	    conditional: None
	};

	assert_eq!(op, expected_op);
    }

    #[test]
    fn test_to_command() {
	
	let file_path = Path::new("example_files/SimpleGroverBaseProfile.bc");
	let module = Module::from_bc_path(file_path).expect("File not found.");

	let func_name = "Microsoft__Quantum__Samples__SimpleGrover__SearchForMarkedInput__Interop";
	let func = module.get_func_by_name(func_name).expect("Function not found.");

	let first_call = &func.basic_blocks[0].instrs[0].get_call();

	let command = first_call.to_command().expect("No command found.");

	println!("{:?}", command);

	// Filling out the op type for simple H gate 
	// let optype = circuit::OpType::H;
	let op_register = circuit::Register("q".to_string(), vec![0]);
	let op_args = vec![op_register];
	let op = circuit::Operation{
	    op_type: OpType::H,
	    n_qb: None,
	    params: None,
	    op_box: None,
	    signature: None,
	    conditional: None
	};

	// Filling out the commands
	let expected_command = circuit::Command{ op: op, args: op_args, opgroup: None };

	assert_eq!(command, expected_command);
	
    }

    #[test]
    fn test_generate_circuit_from_instruction_list() {

	let file_path = Path::new("example_files/SimpleGroverBaseProfile_2.bc");
	let module = Module::from_bc_path(file_path).expect("File not found.");

	let func_name = "Microsoft__Quantum__Samples__SimpleGrover__SearchForMarkedInput__Interop";
	let func = module.get_func_by_name(func_name).expect("Function not found.");

	let instructions = &func.basic_blocks[0].instrs;

	let commands: Vec<Command> = instructions
	    .iter()
	    .map(|i| i.get_call().to_command().expect("Command not found."))
	    .collect();

	println!("{:?}", commands);

	// A register of qubits for the circuit
	let register0 = circuit::Register("q".to_string(), vec![0]);
	let register1 = circuit::Register("q".to_string(), vec![1]);
	let register2 = circuit::Register("q".to_string(), vec![2]);
	// Filling out the qubit register while creating an empty bit register
	let circuit_qubits = vec![register0.clone(), register1.clone(), register2.clone()];
	let circuit_bits: Vec<circuit::Register> = vec![];
	
	// Defining the global phase and implicit permutation
	let phase = "0.0".to_string();

	// Two clones for the implicit permutation
	let register3 = register0.clone();
	let register4 = register0.clone();
	let implicit_permutation = vec![circuit::Permutation(register3, register4)];

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
	println!("{:?}", circuit_json_str);
	serde_json::to_writer(&File::create("./data.json").unwrap(), &circuit_json_str);
    }
    
    #[test]
    fn test_generate_circuit_from_single_expression() {
	
	let file_path = Path::new("example_files/SimpleGroverBaseProfile.bc");
	let module = Module::from_bc_path(file_path).expect("File not found.");

	let func_name = "Microsoft__Quantum__Samples__SimpleGrover__SearchForMarkedInput__Interop";
	let func = module.get_func_by_name(func_name).expect("Function not found.");

	let first_instruction = &func.basic_blocks[0].instrs[0];

	// println!("{:?}", first_instruction);

	// let optype: OpType;
	// let qubit_index: Option<&u64>;

	let call_inst = first_instruction.get_call();

	let optype = call_inst.get_optype().expect("Optype not found.");
	let qubit_index = call_inst.get_qubit_index();
	
	// match first_instruction {
	//     llvm_ir::Instruction::Call(call) => call_instr = call,
	//     // Assert_eq!(call.get_in_name().expect("Call not found").as_string(), func_name);
	//     _ => unreachable!(),
	// }
	
	println!("{:?}", optype);
	println!("{:?}", qubit_index);
	
	// optype = call.get_optype().expect("No optype found.");
	// qubit_index = call.get_qubit_index();
	
	// A register of qubits for the circuit
	let register = circuit::Register("q".to_string(), vec![0]);

	// Two clones for the implicit permutation
	let register1 = register.clone();
	let register2 = register.clone();

	// Filling out the qubit register while creating an empty bit register
	let circuit_qubits = vec![register];
	let circuit_bits: Vec<circuit::Register> = vec![];

	// Filling out the op type for simple H gate 
	// let optype = circuit::OpType::H;
	let op_register = circuit::Register("q".to_string(), qubit_index);
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
    
    
    // fn parse_simple_instruction() {

    // 	let file_path = Path::new("example_files/SimpleGroverBaseProfile.bc");
	
    // 	let module = Module::from_bc_path(file_path).expect("File not found.");
    // 	let first_function = &module.functions[0];
    // 	let first_basicblock = &first_function.basic_blocks[0];

    // 	let instructions = &first_basicblock.instrs;

    // 	println!("Instruction {:?}", instructions[1]);

    // 	to_command(&instructions[0]);

    // 	let first_instruction = &first_basicblock.instrs[1];
    // 	let call = match_call(first_instruction);

    // 	println!("Call {:?}", call.unwrap());
    // 	let call_function = &call.unwrap().function;
    // 	println!("Call_function {:?}", call_function);

    // 	let operand = match_function(call_function).unwrap();
    // 	let const_ref = match_operand(operand).unwrap();
    // 	let global_ref = &*const_ref.as_ref().to_string();
    // 	// println!("{:?}", global_ref);
    // 	// println!("{:?}", global_ref.to_string());

    // 	let op: Vec<&str> = global_ref.split("__").collect();
    // 	print!("Op {:?}", op[3]);

    // 	let params = &call.unwrap().arguments;
    // 	println!("Params0 {:?}", params[0].0);

    // 	let param_const_ref = match_operand(&params[0].0).unwrap();

    // 	println!("Param_Const_Ref {}", param_const_ref);


    // 	let stuff: &str = &*param_const_ref.to_string();

    // 	println!("Stuff {}", stuff);

    // 	if stuff.contains("Qubit") {
    // 		println!("Qubit!");
    // 	}

    // 	if stuff.contains("null") {
    // 		println!("null !")

    // 	}
    // }



    // fn test_generate_simple_circuit() {
    // 	// A register of qubits for the circuit
    // 	let register = circuit::Register("q".to_string(), vec![0]);

    // 	// Two clones for the implicit permutation
    // 	let register1 = register.clone();
    // 	let register2 = register.clone();

    // 	// Filling out the qubit register while creating an empty bit register
    // 	let circuit_qubits = vec![register];
    // 	let circuit_bits: Vec<circuit::Register> = vec![];

    // 	// Filling out the op type for simple H gate 
    // 	let optype = circuit::OpType::H;
    // 	let op_register = circuit::Register("q".to_string(), vec![0]);
    // 	let op_args = vec![op_register];
    // 	let op = circuit::Operation{
    // 	    op_type: optype,
    // 	    n_qb: None,
    // 	    params: None,
    // 	    op_box: None,
    // 	    signature: None,
    // 	    conditional: None
    // 	};

    // 	// Filling out the commands
    // 	let command = circuit::Command{op: op, args: op_args, opgroup: None};
    // 	let commands = vec![command];

    // 	// Defining the global phase and implicit permutation
    // 	let phase = "0.0".to_string();
    // 	let implicit_permutation = vec![circuit::Permutation(register1, register2)];

    // 	// Creating the circuit with all previously defined parameters
    // 	let circuit = circuit::Circuit{
    // 	    name: None,
    // 	    phase: phase,
    // 	    commands: commands,
    // 	    qubits: circuit_qubits,
    // 	    bits: circuit_bits,
    // 	    implicit_permutation: implicit_permutation
    // 	};


    // 	let circuit_json_str = serde_json::to_string(&circuit).unwrap();
    // 	// println!("{:?}", c_json.unwrap());

    // 	let file_path = Path::new("example_files/simple_H_pytket_circuit.json");
    // 	let file = File::open(file_path).expect("File not found.");
    // 	let reader = BufReader::new(file);
    // 	let pytket_circuit: circuit::Circuit = serde_json::from_reader(reader).expect("Error while reading.");
    // 	// serde_json::to_writer(&File::create("./data.json").unwrap(), &c);
    // 	let pytket_circuit_str: String = serde_json::to_string(&pytket_circuit).unwrap();

    // 	assert_eq!(circuit_json_str, pytket_circuit_str);
    // }   

}
