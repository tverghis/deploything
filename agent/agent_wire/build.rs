fn main() {
    prost_build::compile_protos(
        &["../../protos/deploything/v1/run_application.proto"],
        &["../../protos"],
    )
    .unwrap();
}
