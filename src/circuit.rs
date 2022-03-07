use std::collections::HashMap;

use inkwell::values::{
    BasicMetadataValueEnum, BasicValueEnum, FunctionValue, IntValue, PointerValue,
};
use qirlib::codegen::CodeGenerator;
use qirlib::generation::interop::{ClassicalRegister, CodeGenModel, QuantumRegister};
use qirlib::generation::qir::instructions::{controlled, get_qubit, measure};
use serde::ser::Error as SerdeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

// Use a Patch enum to distinguish between a missing value and a null value.
// https://stackoverflow.com/questions/44331037/how-can-i-distinguish-between-a-deserialized-field-that-is-missing-and-one-that/44332837#44332837
#[derive(Debug)]
pub enum Patch<T> {
    Missing,
    Null,
    Value(T),
}

impl<T> Default for Patch<T> {
    fn default() -> Self {
        Patch::Missing
    }
}

impl<T> From<Option<T>> for Patch<T> {
    fn from(opt: Option<T>) -> Patch<T> {
        match opt {
            Some(v) => Patch::Value(v),
            None => Patch::Null,
        }
    }
}

impl<T> From<Patch<T>> for Option<T> {
    fn from(patch: Patch<T>) -> Option<T> {
        match patch {
            Patch::Value(v) => Some(v),
            Patch::Null => None,
            Patch::Missing => {
                panic!("Trying to convert a missing field to Option!")
            }
        }
    }
}

impl<'de, T> Deserialize<'de> for Patch<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::deserialize(deserializer).map(Into::into)
    }
}

impl<T> Patch<T> {
    pub fn is_missing(&self) -> bool {
        matches!(self, Patch::Missing)
    }
}

impl<T: Serialize> Serialize for Patch<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            // this will be serialized as null
            Patch::Null => serializer.serialize_none(),
            Patch::Value(v) => v.serialize(serializer),
            // should have been skipped
            Patch::Missing => Err(SerdeError::custom(
                r#"Patch fields need to be annotated with: 
  #[serde(default, skip_serializing_if = "Patch::is_missing")]"#,
            )),
        }
    }
}

/// Pytket specific models

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Register(pub String, pub Vec<u64>);

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct CompositeGate {
    // List of Symbols
    args: Vec<String>,
    definition: Box<Circuit>,
    name: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct BoxID(uuid::Uuid);

/// Box for an operation, the enum variant names come from the names
/// of the C++ operations and are renamed if the string corresponding
/// to the operation is differently named when serializing.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum OpBox {
    CircBox {
        id: BoxID,
        circuit: Circuit,
    },
    Unitary1qBox {
        id: BoxID,
        // 2x2 matrix of complex numbers
        matrix: [[(f32, f32); 2]; 2],
    },
    Unitary2qBox {
        id: BoxID,
        // 4x4 matrix of complex numbers
        matrix: [[(f32, f32); 4]; 4],
    },
    ExpBox {
        id: BoxID,
        // 4x4 matrix of complex numbers
        matrix: [[(f32, f32); 4]; 4],
        phase: f64,
    },
    PauliExpBox {
        id: BoxID,
        paulis: Vec<String>,
        // Symengine Expr
        phase: String,
    },
    PhasePolyBox {
        id: BoxID,
        n_qubits: u32,
        qubit_indices: Vec<(u32, u32)>,
    },
    Composite {
        id: BoxID,
        gate: CompositeGate,
        // Vec of Symengine Expr
        params: Vec<String>,
    },
    QControlBox {
        id: BoxID,
        n_controls: u32,
        op: Box<Operation>,
    },
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum OpType {
    H,
    CX,
    Measure,
    X,
    Y,
    Z,
    Rx,
    Ry,
    Rz,
    Conditional,
    CircBox,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Conditional {
    op: Box<Operation>,
    width: u32,
    value: u32,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Operation {
    #[serde(rename = "type")]
    pub op_type: OpType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n_qb: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Vec<String>>,
    #[serde(rename = "box")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub op_box: Option<OpBox>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditional: Option<Conditional>,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Command {
    pub op: Operation,
    pub args: Vec<Register>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opgroup: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Permutation(pub Register, pub Register);

/// Pytket canonical circuit
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Circuit {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    // Symengine Expr
    pub phase: String,
    pub commands: Vec<Command>,
    pub qubits: Vec<Register>,
    pub bits: Vec<Register>,
    pub implicit_permutation: Vec<Permutation>,
}

impl CodeGenModel for Circuit {
    fn name(&self) -> String {
        self.name.clone().unwrap_or("tket_circuit".to_string())
    }

    fn number_of_registers(&self) -> usize {
        self.registers().len()
    }

    fn registers(&self) -> Vec<ClassicalRegister> {
        let mut regmap = HashMap::new();
        for bit in self.bits.iter() {
            if let Some(size) = regmap.get_mut(&bit.0) {
                *size = std::cmp::max(*size, bit.1[0] + 1);
            } else {
                regmap.insert(bit.0.clone(), bit.1[0] + 1);
            }
        }
        regmap
            .into_iter()
            .map(|(name, size)| ClassicalRegister::new(name, size))
            .collect()
    }

    fn qubits(&self) -> Vec<QuantumRegister> {
        self.qubits
            .iter()
            .map(|qb| QuantumRegister::new(qb.0.clone(), qb.1[0] as u64))
            .collect()
    }

    fn static_alloc(&self) -> bool {
        true
    }

    // fn write_instructions<'ctx>(
    //     &self,
    //     generator: &qirlib::codegen::CodeGenerator<'ctx>,
    //     qubits: &HashMap<String, BasicValueEnum<'ctx>>,
    //     registers: &mut HashMap<String, Option<PointerValue<'ctx>>>,
    // ) {
    // }

    fn number_of_qubits(&self) -> usize {
        self.qubits.len()
    }

    fn write_instructions<'ctx>(
        &self,
        generator: &CodeGenerator<'ctx>,
        qubits: &HashMap<String, BasicValueEnum<'ctx>>,
        registers: &mut HashMap<String, Option<PointerValue<'ctx>>>,
        entry_point: FunctionValue,
    ) {
        for com in &self.commands {
            emit(generator, com, qubits, registers, entry_point);
        }
    }
}

pub(crate) fn emit<'ctx>(
    generator: &CodeGenerator<'ctx>,
    com: &Command,
    qubits: &HashMap<String, BasicValueEnum<'ctx>>,
    registers: &mut HashMap<String, Option<PointerValue<'ctx>>>,
    entry_point: FunctionValue,
) {
    let qb_name = |reg: &Register| format!("{}{}", &reg.0[..], reg.1[0]);

    let find_qubit = |reg: &Register| get_qubit(&qb_name(reg), qubits);

    let optype = &com.op.op_type;
    let params: Option<Vec<f64>> = com.op.params.as_ref().map(|params| {
        params
            .iter()
            .map(|p| p.parse().expect("Could not parse parameter to float."))
            .collect()
    });
    match optype {
        OpType::CX => {
            let control = find_qubit(&com.args[0]);
            let qubit = find_qubit(&com.args[1]);
            controlled(generator, generator.qis_cnot_body(), control, qubit);
        }
        OpType::H => {
            generator.emit_void_call(generator.qis_h_body(), &[find_qubit(&com.args[0]).into()]);
        }
        OpType::X => {
            generator.emit_void_call(generator.qis_x_body(), &[find_qubit(&com.args[0]).into()]);
        }
        OpType::Y => {
            generator.emit_void_call(generator.qis_y_body(), &[find_qubit(&com.args[0]).into()]);
        }
        OpType::Z => {
            generator.emit_void_call(generator.qis_z_body(), &[find_qubit(&com.args[0]).into()]);
        }
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
        // Instruction::Cz(inst) => {
        //     let control = find_qubit(&inst.control);
        //     let qubit = find_qubit(&inst.target);
        //     controlled(
        //         generator,
        //         generator
        //             .qis_cz_body()
        //             .expect("qis_cz_body must be defined in the template"),
        //         control,
        //         qubit,
        //     );
        // }
        // Instruction::Rx(inst) => {
        //     generator.emit_void_call(
        //         generator
        //             .qis_rx_body()
        //             .expect("qis_rx_body must be defined in the template"),
        //         &[
        //             generator.f64_to_f64(inst.theta),
        //             find_qubit(&inst.qubit).into(),
        //         ],
        //     );
        // }
        // Instruction::Ry(inst) => {
        //     generator.emit_void_call(
        //         generator
        //             .qis_ry_body()
        //             .expect("qis_ry_body must be defined in the template"),
        //         &[
        //             generator.f64_to_f64(inst.theta),
        //             find_qubit(&inst.qubit).into(),
        //         ],
        //     );
        // }
        OpType::Rz => {
            generator.emit_void_call(
                generator.qis_rz_body(),
                &[
                    generator.f64_to_f64(params.expect("Rz requires a parameter.")[0]),
                    find_qubit(&com.args[0]).into(),
                ],
            );
        }
        // Instruction::S(inst) => {
        //     generator.emit_void_call(
        //         generator
        //             .qis_s_body()
        //             .expect("qis_s_body must be defined in the template"),
        //         &[find_qubit(&inst.qubit).into()],
        //     );
        // }
        // Instruction::SAdj(inst) => {
        //     generator.emit_void_call(
        //         generator
        //             .qis_s_adj()
        //             .expect("qis_s_adj must be defined in the template"),
        //         &[find_qubit(&inst.qubit).into()],
        //     );
        // }
        // Instruction::T(inst) => {
        //     generator.emit_void_call(
        //         generator
        //             .qis_t_body()
        //             .expect("qis_t_body must be defined in the template"),
        //         &[find_qubit(&inst.qubit).into()],
        //     );
        // }
        // Instruction::TAdj(inst) => {
        //     generator.emit_void_call(
        //         generator
        //             .qis_t_adj()
        //             .expect("qis_t_adj must be defined in the template"),
        //         &[find_qubit(&inst.qubit).into()],
        //     );
        // }
        OpType::Conditional => {
            let (condition_bit, args) = match &com.args[..] {
                [a, b @ ..] => (a, b),
                _ => panic!("Not enough args to conditional."),
            };
            let mut conditional = com.op.conditional.as_ref().unwrap().clone();
            // for now only support conditional circbox
            if let Some(OpBox::CircBox { .. }) = conditional.op.op_box.as_ref() {
                ()
            } else {
                let newcirc = Circuit {
                    name: None,
                    phase: "".into(),
                    commands: vec![Command {
                        op: *conditional.op,
                        args: args.to_vec(),
                        opgroup: com.opgroup.clone(),
                    }],
                    qubits: args.to_vec(),
                    // if inner op uses bits this will break
                    bits: vec![],
                    implicit_permutation: vec![],
                };
                conditional.op = Box::new(Operation {
                    op_box: Some(OpBox::CircBox {
                        circuit: newcirc,
                        id: BoxID(uuid::Uuid::default()),
                    }),
                    op_type: OpType::CircBox,
                    n_qb: None,
                    params: None,
                    signature: None,
                    conditional: None,
                });
            }
            emit_if(
                generator,
                registers,
                qubits,
                entry_point,
                &conditional,
                &qb_name(condition_bit),
            )
        }
        _ => panic!("unsupported optype"),
    }
}

fn emit_if<'ctx>(
    generator: &CodeGenerator<'ctx>,
    registers: &mut HashMap<String, Option<PointerValue<'ctx>>>,
    qubits: &HashMap<String, BasicValueEnum<'ctx>>,
    entry_point: FunctionValue,
    conditional: &Conditional,
    condition_bit: &String,
) {
    let inner_circ = if let Some(OpBox::CircBox { circuit, .. }) = conditional.op.op_box.as_ref() {
        circuit
    } else {
        panic!("only works with CircBoxes.")
    };

    let comparison = match (conditional.width, conditional.value) {
        (1, 1) => get_one(generator),
        (1, 0) => get_zero(generator),
        _ => panic!("only supports condtioning on one bit"),
    };
    let result = registers
        .get(condition_bit)
        .unwrap_or_else(|| panic!("Result {} not found.", condition_bit))
        .unwrap_or_else(|| get_zero(generator));

    let condition = equal(generator, result, comparison);
    let then_block = generator.context.append_basic_block(entry_point, "then");
    let else_block = generator.context.append_basic_block(entry_point, "else");
    generator
        .builder
        .build_conditional_branch(condition, then_block, else_block);

    let continue_block = generator
        .context
        .append_basic_block(entry_point, "continue");

    let mut emit_block = |block, insts| {
        generator.builder.position_at_end(block);
        for inst in insts {
            emit(generator, inst, qubits, registers, entry_point);
        }

        generator.builder.build_unconditional_branch(continue_block);
    };

    emit_block(then_block, &inner_circ.commands);
    emit_block(else_block, &vec![]);
    generator.builder.position_at_end(continue_block);
}

fn get_zero<'a>(generator: &CodeGenerator<'a>) -> PointerValue<'a> {
    generator
        .emit_call_with_return(generator.rt_result_get_zero(), &[], "zero")
        .into_pointer_value()
}

pub(crate) fn get_one<'a>(generator: &CodeGenerator<'a>) -> PointerValue<'a> {
    generator
        .emit_call_with_return(generator.rt_result_get_one(), &[], "one")
        .into_pointer_value()
}

pub(crate) fn equal<'a>(
    generator: &CodeGenerator<'a>,
    result1: PointerValue<'a>,
    result2: PointerValue<'a>,
) -> IntValue<'a> {
    let result1 = BasicMetadataValueEnum::PointerValue(result1);
    let result2 = BasicMetadataValueEnum::PointerValue(result2);
    generator
        .emit_call_with_return(generator.rt_result_equal(), &[result1, result2], "equal")
        .into_int_value()
}
