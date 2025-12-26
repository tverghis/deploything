fn main() {
    prost_build::compile_protos(
        &[
            "../../protos/deploything/v1/remote_command.proto",
            "../../protos/deploything/v1/agent_snapshot.proto",
        ],
        &["../../protos"],
    )
    .unwrap();
}
