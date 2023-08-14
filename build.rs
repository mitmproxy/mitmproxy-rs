extern crate prost_build;

fn main() {
    let protoc_path = match std::env::var("PROTOC") {
        Ok(path) if !path.is_empty() => std::path::PathBuf::from(path),
        _ => protoc_bin_vendored::protoc_bin_path().expect("protoc is not available"),
    };
    std::env::set_var("PROTOC", protoc_path);

    prost_build::compile_protos(
        &["./src/packet_sources/ipc.proto"],
        &["./src/packet_sources/"],
    )
    .unwrap();
}
