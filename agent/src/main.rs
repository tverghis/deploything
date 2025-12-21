use agent::docker_api::Container;

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let docker = bollard::Docker::connect_with_defaults().unwrap();
    let container = Container::spawn_from_image(&docker, "mccutchen/go-httpbin", "latest")
        .await
        .unwrap();

    container.stop().await.unwrap();
}
