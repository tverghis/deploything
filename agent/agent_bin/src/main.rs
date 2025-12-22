use agent_bin::docker_api::{Container, ContainerHostConfig};

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let docker = bollard::Docker::connect_with_defaults().unwrap();
    let container = Container::spawn_from_image(
        &docker,
        "mccutchen/go-httpbin",
        "latest",
        ContainerHostConfig::With {
            port_map: ("8080/tcp", "8080").into(),
        },
    )
    .await
    .unwrap();

    container.stop().await.unwrap();
}
