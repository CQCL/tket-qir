// use llvm_ir::Module;
// use llvm_ir::function::Function;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::FunctionValue;

use std::path::Path;

use crate::circuit::{OpType};



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


// pub struct QIRRuntime<'ctx> {
//     pub quantum_application_run: FunctionValue<'ctx>,
// }

// impl<'ctx> QIRRuntime<'ctx> {
//     pub fn new(module: &Module<'ctx>) -> Self {
// 	QIRRuntime {
// 	    quantum_application_run: mo
	    
// 	}
//     }

//     pub get_function(module: &Module<'ctx>, name: &str) -> Option<Function<'ctx>> {
// 	let function_name =
//     }
// }
