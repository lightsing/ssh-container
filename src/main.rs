use std::process::exit;

use anyhow::Result;
use bollard::container::Config;
use bollard::exec::CreateExecOptions;
use bollard::Docker;

const IMAGE: &str = "ubuntu:focal";

mod docker;

#[tokio::main]
async fn main() -> Result<()> {
    docker::init(Docker::connect_with_unix_defaults().unwrap());

    docker::pull(IMAGE).await?;

    let container_config = Config {
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
            cmd: Some(vec!["bash"]),
            working_dir: Some("/root"),
            ..Default::default()
        })
        .await?;

    exec.attach().await?;

    container.remove().await?;
    exit(0);
}
