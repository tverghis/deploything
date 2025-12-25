use std::time::Duration;

use agent_bin::docker_api;
use agent_wire::deploything::v1::{ContainerHostConfig, PortMap};
use bollard::Docker;

#[tokio::test]
async fn spawn_and_stop() {
    let docker = Docker::connect_with_defaults().unwrap();

    let host_config = Some(ContainerHostConfig {
        port_map: Some(PortMap {
            from: Some("8080/tcp".into()),
            to: Some("8080".into()),
        }),
    });

    let container = docker_api::Container::spawn_from_image(
        &docker,
        "mccutchen/go-httpbin",
        "latest",
        host_config.as_ref(),
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
