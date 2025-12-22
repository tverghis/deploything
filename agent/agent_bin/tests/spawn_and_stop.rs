use std::time::Duration;

use agent_bin::docker_api::{self, ContainerHostConfig};
use bollard::Docker;

#[tokio::test]
async fn spawn_and_stop() {
    let docker = Docker::connect_with_defaults().unwrap();

    let host_config = ContainerHostConfig::With {
        port_map: ("8080/tcp", "8080").into(),
    };

    let container = docker_api::Container::spawn_from_image(
        &docker,
        "mccutchen/go-httpbin",
        "latest",
        host_config,
    )
    .await
    .unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;

    let resp = ureq::get("http://localhost:8080/status/200")
        .call()
        .unwrap();

    assert_eq!(200, resp.status());

    container.stop().await.unwrap();
}
