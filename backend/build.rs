extern crate prost_build;

fn main() {
    prost_build::compile_protos(&["src/message.proto"], &["backend/src", "src/"]).unwrap();
}
