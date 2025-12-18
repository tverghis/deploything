use bollard::{Docker, query_parameters::CreateImageOptionsBuilder};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    let docker = bollard::Docker::connect_with_defaults().unwrap();
    pull_image(&docker, "alpine", "3").await;
}

async fn pull_image(docker: &Docker, name: &str, tag: &str) {
    let options = CreateImageOptionsBuilder::new()
        .from_image(name)
        .tag(tag)
        .build();

    let mut create_image_stream = docker.create_image(Some(options), None, None);

    while let Some(info) = create_image_stream.next().await {
        match info {
            Ok(info) => {
                println!("{}: {}", info.id.unwrap(), info.status.unwrap());
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}
