use std::process::exit;

use anyhow::Result;
use bollard::container;
use bollard::exec::CreateExecOptions;

const IMAGE: &str = "alpine:latest";

mod docker;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    docker::pull(IMAGE).await?;

    let container_config = container::Config {
        image: Some(IMAGE),
        tty: Some(true),
        ..Default::default()
    };

    let container = docker::Container::new::<_, &str>(container_config, None, None).await?;
    container.start().await?;

    let exec = container
        .exec(CreateExecOptions {
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            attach_stdin: Some(true),
            tty: Some(true),
            cmd: Some(vec!["ash"]),
            working_dir: Some("/root"),
            ..Default::default()
        })
        .await?;

    exec.attach().await?;

    container.remove().await?;
    exit(0);
}
