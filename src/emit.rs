// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::array1d;
use super::basic_values;
use crate::circuit::Register;
use crate::circuit::{Circuit, Command, OpType};
use inkwell::values::{BasicMetadataValueEnum, BasicValue, BasicValueEnum};
use inkwell::{module::Module, values::FunctionValue};
use qirlib::passes::run_basic_passes_on;
use qirlib::{codegen::CodeGenerator, module};
use std::collections::HashMap;
/// # Errors
///
/// Will return `Err` if module fails verification that the current `Module` is valid.
pub fn write_circ_to_file(circ: &Circuit, file_name: &str) -> Result<(), String> {
    let ctx = inkwell::context::Context::create();
    let generator = populate_context(&ctx, circ)?;
    run_basic_passes_on(&generator.module);
    generator.emit_ir(file_name)?;

    Ok(())
}

/// # Errors
///
/// Will return `Err` if module fails verification that the current `Module` is valid.
pub fn get_ir_string(circ: &Circuit) -> Result<String, String> {
    let ctx = inkwell::context::Context::create();
    let generator = populate_context(&ctx, circ)?;
    run_basic_passes_on(&generator.module);
    let ir = generator.get_ir_string();

    Ok(ir)
}

/// # Errors
///
/// Will return `Err` if module fails verification that the current `Module` is valid.
pub fn get_bitcode_base64_string(circ: &Circuit) -> Result<String, String> {
    let ctx = inkwell::context::Context::create();
    let generator = populate_context(&ctx, circ)?;
    run_basic_passes_on(&generator.module);

    let b64 = generator.get_bitcode_base64_string();

    Ok(b64)
}

/// # Errors
///
/// Will return `Err` if
///  - module cannot be loaded.
///  - module fails verification that the current `Module` is valid.
pub fn populate_context<'a>(
    ctx: &'a inkwell::context::Context,
    circ: &'a Circuit,
) -> Result<CodeGenerator<'a>, String> {
    let module = module::load_template(
        &circ.name.clone().unwrap_or("tket_circuit".to_string()),
        ctx,
    )?;
    let generator = CodeGenerator::new(ctx, module)?;
    build_entry_function(&generator, circ)?;
    Ok(generator)
}

fn build_entry_function(generator: &CodeGenerator<'_>, circ: &Circuit) -> Result<(), String> {
    let entrypoint = get_entry_function(&generator.module);

    let entry = generator.context.append_basic_block(entrypoint, "entry");
    generator.builder.position_at_end(entry);

    let qubits = write_qubits(circ, generator);

    let registers = write_registers(circ, generator);

    write_instructions(circ, generator, &qubits, &registers);

    free_qubits(generator, &qubits);

    let output = registers.get("results").unwrap();
    generator.builder.build_return(Some(&output.0));

    generator.module.verify().map_err(|e| e.to_string())
}

fn free_qubits<'ctx>(
    generator: &CodeGenerator<'ctx>,
    qubits: &HashMap<String, BasicValueEnum<'ctx>>,
) {
    for (_, value) in qubits.iter() {
        emit_release(generator, value);
    }
}

fn write_qubits<'ctx>(
    circ: &Circuit,
    generator: &CodeGenerator<'ctx>,
) -> HashMap<String, BasicValueEnum<'ctx>> {
    let qubits = circ
        .qubits
        .iter()
        .map(|reg| {
            let indexed_name = format!("{}{}", &reg.0[..], reg.1[0]);
            let value = emit_allocate(generator, indexed_name.as_str());
            (indexed_name, value)
        })
        .collect();

    qubits
}

struct TmpReg {
    pub name: String,
    pub size: u64,
}
fn write_registers<'ctx>(
    circ: &Circuit,
    generator: &CodeGenerator<'ctx>,
) -> HashMap<String, (BasicValueEnum<'ctx>, Option<u64>)> {
    let mut registers = HashMap::new();
    // let number_of_registers = circ.registers.len() as u64;
    let number_of_registers = 1;
    if number_of_registers > 0 {
        let results = array1d::emit_array_allocate1d(generator, 8, number_of_registers, "results");
        registers.insert(String::from("results"), (results, None));
        let mut sub_results = vec![];
        for reg in [TmpReg {
            name: "c".to_string(),
            size: circ.bits.len() as u64,
        }] {
            let (sub_result, entries) =
                array1d::emit_array_1d(generator, reg.name.as_str(), reg.size);
            sub_results.push(sub_result);
            registers.insert(reg.name.clone(), (sub_result, None));
            for (index, _) in entries {
                registers.insert(format!("{}{}", reg.name, index), (sub_result, Some(index)));
            }
        }
        array1d::set_elements(generator, &results, &sub_results, "results");
    } else {
        let results = array1d::emit_empty_result_array_allocate1d(generator, "results");
        registers.insert(String::from("results"), (results, None));
    }
    // let results = array1d::emit_empty_result_array_allocate1d(generator, "results");
    // registers.insert(String::from("results"), (results, None));

    registers
}

fn write_instructions<'ctx>(
    circ: &Circuit,
    generator: &CodeGenerator<'ctx>,
    qubits: &HashMap<String, BasicValueEnum<'ctx>>,
    registers: &HashMap<String, (BasicValueEnum<'ctx>, Option<u64>)>,
) {
    for com in &circ.commands {
        emit(generator, com, qubits, registers);
    }
}

pub(crate) fn get_entry_function<'ctx>(module: &Module<'ctx>) -> FunctionValue<'ctx> {
    let ns = "QuantumApplication";
    let method = "Run";
    let entrypoint_name = format!("{}__{}__body", ns, method);
    let entrypoint = module.get_function(&entrypoint_name).unwrap();

    while let Some(basic_block) = entrypoint.get_last_basic_block() {
        unsafe {
            basic_block.delete().unwrap();
        }
    }
    entrypoint
}

pub(crate) fn emit_allocate<'ctx>(
    generator: &CodeGenerator<'ctx>,
    result_name: &str,
) -> BasicValueEnum<'ctx> {
    let args = [];
    emit_call_with_return(
        generator,
        generator.runtime_library.qubit_allocate.unwrap(),
        &args,
        result_name,
    )
}

pub(crate) fn emit_release<'ctx>(generator: &CodeGenerator<'ctx>, qubit: &BasicValueEnum<'ctx>) {
    let args = [qubit.as_basic_value_enum().into()];
    emit_void_call(generator, generator.runtime_library.qubit_release.unwrap(), &args);
}

pub(crate) fn emit_void_call<'ctx>(
    generator: &CodeGenerator<'ctx>,
    function: FunctionValue<'ctx>,
    args: &[BasicMetadataValueEnum<'ctx>],
) {
    let _ = generator
        .builder
        .build_call(function, args, "")
        .try_as_basic_value()
        .right()
        .unwrap();
}

pub(crate) fn emit_call_with_return<'ctx>(
    generator: &CodeGenerator<'ctx>,
    function: FunctionValue<'ctx>,
    args: &[BasicMetadataValueEnum<'ctx>],
    name: &str,
) -> BasicValueEnum<'ctx> {
    generator
        .builder
        .build_call(function, args, name)
        .try_as_basic_value()
        .left()
        .unwrap()
}

/// # Panics
///
/// Panics if the qubit name doesn't exist
fn get_qubit<'ctx>(
    name: &str,
    qubits: &HashMap<String, BasicValueEnum<'ctx>>,
) -> BasicValueEnum<'ctx> {
    *qubits
        .get(name)
        .unwrap_or_else(|| panic!("Qubit {} not found.", name))
}

/// # Panics
///
/// Panics if the register name doesn't exist
fn get_register<'ctx>(
    name: &str,
    registers: &HashMap<String, (BasicValueEnum<'ctx>, Option<u64>)>,
) -> (BasicValueEnum<'ctx>, Option<u64>) {
    registers
        .get(name)
        .unwrap_or_else(|| panic!("Register {} not found.", name))
        .to_owned()
}

fn measure<'ctx>(
    generator: &CodeGenerator<'ctx>,
    qubit: &str,
    target: &str,
    qubits: &HashMap<String, BasicValueEnum<'ctx>>,
    registers: &HashMap<String, (BasicValueEnum<'ctx>, Option<u64>)>,
) {
    let find_qubit = |name| get_qubit(name, qubits);
    let find_register = |name| get_register(name, registers);

    // measure the qubit and save the result to a temporary value
    let result = emit_call_with_return(
        generator,
        generator
            .intrinsics
            .m
            .expect("m must be defined in the template"),
        &[find_qubit(qubit).into()],
        "measurement",
    );

    // find the parent register and offset for the given target
    let (register, index) = find_register(target);
    // get the bitcast pointer to the target location
    let bitcast_indexed_target_register = array1d::get_bitcast_result_pointer_array_element(
        generator,
        index.unwrap(),
        &register,
        target,
    );

    // get the existing value from that location and decrement its ref count as its
    // being replaced with the measurement.
    let existing_value = generator.builder.build_load(
        bitcast_indexed_target_register.into_pointer_value(),
        "existing_value",
    );
    let minus_one = basic_values::i64_to_i32(generator, -1);
    generator.builder.build_call(
        generator.runtime_library.result_update_reference_count.unwrap(),
        &[existing_value.into(), minus_one],
        "",
    );

    // increase the ref count of the new value and store it in the target register
    let one = basic_values::i64_to_i32(generator, 1);
    generator.builder.build_call(
        generator.runtime_library.result_update_reference_count.unwrap(),
        &[result.into(), one],
        "",
    );
    let _ = generator
        .builder
        .build_store(bitcast_indexed_target_register.into_pointer_value(), result);
}

fn controlled<'ctx>(
    generator: &CodeGenerator<'ctx>,
    intrinsic: FunctionValue<'ctx>,
    control: BasicValueEnum<'ctx>,
    qubit: BasicValueEnum<'ctx>,
) {
    emit_void_call(generator, intrinsic, &[control.into(), qubit.into()]);
    let minus_one = basic_values::i64_to_i32(generator, -1);
    generator.builder.build_call(
        generator.runtime_library.array_update_reference_count.unwrap(),
        &[control.into(), minus_one],
        "",
    );
}

pub(crate) fn emit<'ctx>(
    generator: &CodeGenerator<'ctx>,
    com: &Command,
    qubits: &HashMap<String, BasicValueEnum<'ctx>>,
    registers: &HashMap<String, (BasicValueEnum<'ctx>, Option<u64>)>,
) {
    let qb_name = |reg: &Register| format!("{}{}", &reg.0[..], reg.1[0]);
    let intrinsics = &generator.intrinsics;
    let find_qubit = |reg: &Register| get_qubit(&qb_name(reg), qubits);
    let ctl = |value| array1d::create_ctl_wrapper(generator, value);
    let optype = &com.op.op_type;
    match optype {
        OpType::CX => {
            let control = ctl(&find_qubit(&com.args[0]));
            let qubit = find_qubit(&com.args[1]);
            controlled(
                generator,
                intrinsics
                    .x_ctl
                    .expect("x_ctl must be defined in the template"),
                control,
                qubit,
            );
        }
        // Instruction::Cz(inst) => {
        //     let control = ctl(&find_qubit(&inst.control));
        //     let qubit = find_qubit(&inst.target);
        //     controlled(
        //         generator,
        //         intrinsics
        //             .z_ctl
        //             .expect("z_ctl must be defined in the template"),
        //         control,
        //         qubit,
        //     );
        // }
        OpType::H => emit_void_call(
            generator,
            intrinsics.h.expect("h must be defined in the template"),
            &[find_qubit(&com.args[0]).into()],
        ),
        OpType::Measure => {
            measure(
                generator,
                &qb_name(&com.args[0]),
                &qb_name(&com.args[1]),
                // &format!("results"),
                // &format!("results{}", &com.args[1].1[0]),
                qubits,
                registers,
            );
        }
        //     Instruction::Reset(inst) => calls::emit_void_call(
        //         generator,
        //         intrinsics
        //             .reset
        //             .expect("reset must be defined in the template"),
        //         &[find_qubit(&inst.qubit).into()],
        //     ),
        //     Instruction::Rx(inst) => calls::emit_void_call(
        //         generator,
        //         intrinsics.r_x.expect("r_x must be defined in the template"),
        //         &[
        //             basic_values::f64_to_f64(generator, inst.theta),
        //             find_qubit(&inst.qubit).into(),
        //         ],
        //     ),
        //     Instruction::Ry(inst) => calls::emit_void_call(
        //         generator,
        //         intrinsics.r_y.expect("r_y must be defined in the template"),
        //         &[
        //             basic_values::f64_to_f64(generator, inst.theta),
        //             find_qubit(&inst.qubit).into(),
        //         ],
        //     ),
        //     Instruction::Rz(inst) => calls::emit_void_call(
        //         generator,
        //         intrinsics.r_z.expect("r_z must be defined in the template"),
        //         &[
        //             basic_values::f64_to_f64(generator, inst.theta),
        //             find_qubit(&inst.qubit).into(),
        //         ],
        //     ),
        //     Instruction::S(inst) => calls::emit_void_call(
        //         generator,
        //         intrinsics.s.expect("s must be defined in the template"),
        //         &[find_qubit(&inst.qubit).into()],
        //     ),
        //     Instruction::SAdj(inst) => calls::emit_void_call(
        //         generator,
        //         intrinsics
        //             .s_adj
        //             .expect("s_adj must be defined in the template"),
        //         &[find_qubit(&inst.qubit).into()],
        //     ),
        //     Instruction::T(inst) => calls::emit_void_call(
        //         generator,
        //         intrinsics.t.expect("t must be defined in the template"),
        //         &[find_qubit(&inst.qubit).into()],
        //     ),
        //     Instruction::TAdj(inst) => calls::emit_void_call(
        //         generator,
        //         intrinsics
        //             .t_adj
        //             .expect("t_adj must be defined in the template"),
        //         &[find_qubit(&inst.qubit).into()],
        //     ),
        //     Instruction::X(inst) => calls::emit_void_call(
        //         generator,
        //         intrinsics.x.expect("x must be defined in the template"),
        //         &[find_qubit(&inst.qubit).into()],
        //     ),
        //     Instruction::Y(inst) => calls::emit_void_call(
        //         generator,
        //         intrinsics.y.expect("y must be defined in the template"),
        //         &[find_qubit(&inst.qubit).into()],
        //     ),
        //     Instruction::Z(inst) => calls::emit_void_call(
        //         generator,
        //         intrinsics.z.expect("z must be defined in the template"),
        //         &[find_qubit(&inst.qubit).into()],
        //     ),
        //     Instruction::DumpMachine => calls::emit_void_call(
        //         generator,
        //         intrinsics
        //             .dumpmachine
        //             .expect("dumpmachine must be defined before use"),
        //         &[basic_values::i8_null_ptr(generator)],
        //     ),
        _ => panic!("unsupported optype"),
    }
}
