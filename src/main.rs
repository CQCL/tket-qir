// mod array1d;
// mod basic_values;
mod circuit;
// mod emit;
use qirlib::generation::emit::ir;
// use pyqir_generator::emitemit::{get_ir_string, write_circ_to_file};

fn read_json_str(circ_s: &str) -> String {
    let p: circuit::Circuit = serde_json::from_str(circ_s).unwrap();
    ir(&p).unwrap()
}
fn main() {
    println!(
        "{}",
        read_json_str(
            r#"{"bits": [["c", [0]], ["c", [1]]], "commands": [{"args": [["q", [0]]], "op": {"type": "H"}}, {"args": [["q", [0]], ["q", [1]]], "op": {"type": "CX"}}, {"args": [["q", [0]], ["c", [0]]], "op": {"type": "Measure"}}, {"args": [["q", [1]]], "op": {"params": ["0.2"], "type": "Rz"}}, {"args": [["q", [1]], ["c", [1]]], "op": {"type": "Measure"}}], "implicit_permutation": [[["q", [0]], ["q", [0]]], [["q", [1]], ["q", [1]]]], "phase": "0.0", "qubits": [["q", [0]], ["q", [1]]]}"#
        )
    );
    // dbg!(get_ir_string(&p));
    println!(
        "{}",
        read_json_str(
            r#"{"bits": [["c", [0]], ["c", [1]]], "commands": [{"args": [["q", [0]]], "op": {"type": "X"}}, {"args": [["q", [0]], ["c", [0]]], "op": {"type": "Measure"}}, {"args": [["c", [0]], ["q", [1]]], "op": {"conditional": {"op": {"type": "Z"}, "value": 0, "width": 1}, "type": "Conditional"}}, {"args": [["c", [1]], ["q", [0]], ["q", [1]]], "op": {"conditional": {"op": {"box": {"circuit": {"bits": [], "commands": [{"args": [["q", [0]], ["q", [1]]], "op": {"type": "CX"}}], "implicit_permutation": [[["q", [0]], ["q", [0]]], [["q", [1]], ["q", [1]]]], "phase": "0.0", "qubits": [["q", [0]], ["q", [1]]]}, "id": "91810268-1b06-47b4-8609-992d066b56f2", "type": "CircBox"}, "type": "CircBox"}, "value": 1, "width": 1}, "type": "Conditional"}}], "implicit_permutation": [[["q", [0]], ["q", [0]]], [["q", [1]], ["q", [1]]]], "phase": "0.0", "qubits": [["q", [0]], ["q", [1]]]}"#
        )
    );
}
