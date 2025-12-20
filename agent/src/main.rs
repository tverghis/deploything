use agent::docker_api;

#[tokio::main]
async fn main() {
    let docker = bollard::Docker::connect_with_defaults().unwrap();
    docker_api::image::pull(&docker, "alpine", "3")
        .await
        .unwrap();
}
