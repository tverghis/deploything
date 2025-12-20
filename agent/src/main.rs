use agent::docker_api;

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let docker = bollard::Docker::connect_with_defaults().unwrap();
    docker_api::image::pull(&docker, "alpine", "3")
        .await
        .unwrap();
}
