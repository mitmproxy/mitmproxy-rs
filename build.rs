extern crate prost_build;
use protoc_bin_vendored;

fn main() {
    if let Ok(protoc_path) = protoc_bin_vendored::protoc_bin_path() {
        std::env::set_var("PROTOC", protoc_path);
        prost_build::compile_protos(
            &["./src/packet_sources/ipc.proto"],
            &["./src/packet_sources/"],
        )
        .unwrap();
    } else {
        panic!("Protoc path not found");
    }
}
