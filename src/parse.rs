// use llvm_ir::Module;
// use llvm_ir::function::Function;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::FunctionValue;

use std::path::Path;

use crate::circuit::{OpType, Circuit};



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
    use crate::circuit;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::Path;

    #[test]
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
