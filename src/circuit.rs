
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
pub struct Register(pub String, pub Vec<i64>);


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
    X,
    Y,
    Z,
    T,
    Tdg,
    CX,
    Measure,
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
