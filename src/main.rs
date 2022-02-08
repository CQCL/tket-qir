// mod array1d;
// mod basic_values;
mod circuit;
// mod emit;
use pyqir_generator::emit::write_model_to_file;
// use pyqir_generator::emitemit::{get_ir_string, write_circ_to_file};
fn main() {
    let circ_s = r#"{"bits": [["c", [0]], ["c", [1]]], "commands": [{"args": [["q", [0]]], "op": {"type": "H"}}, {"args": [["q", [0]], ["q", [1]]], "op": {"type": "CX"}}, {"args": [["q", [0]], ["c", [0]]], "op": {"type": "Measure"}}, {"args": [["q", [1]]], "op": {"params": ["0.2"], "type": "Rz"}}, {"args": [["q", [1]], ["c", [1]]], "op": {"type": "Measure"}}], "implicit_permutation": [[["q", [0]], ["q", [0]]], [["q", [1]], ["q", [1]]]], "phase": "0.0", "qubits": [["q", [0]], ["q", [1]]]}"#;
    let p: circuit::Circuit = serde_json::from_str(circ_s).unwrap();
    write_model_to_file(&p, "dump.ll");
    // dbg!(get_ir_string(&p));
}
