use anyhow::Result;
use bollard::container::{Config, RemoveContainerOptions, LogOutput};
use bollard::Docker;

use bollard::exec::{CreateExecOptions, ResizeExecOptions, StartExecResults};
use bollard::image::CreateImageOptions;
use futures_util::{StreamExt, TryStreamExt};
use std::io::{stdout, Write, stderr};
use termion::raw::IntoRawMode;
use termion::{terminal_size, get_tty};
use tokio::fs::File;
use tokio::task::spawn;
use tokio::io::AsyncWriteExt;
use std::process::exit;

const IMAGE: &'static str = "ubuntu:focal";

#[tokio::main]
async fn main() -> Result<()> {
    let docker = Docker::connect_with_unix_defaults()?;

    docker
        .create_image(
            Some(CreateImageOptions {
                from_image: IMAGE,
                ..Default::default()
            }),
            None,
            None,
        )
        .try_collect::<Vec<_>>()
        .await?;

    let container_config = Config {
        image: Some(IMAGE),
        tty: Some(true),
        ..Default::default()
    };

    let id = docker
        .create_container::<&str, &str>(None, container_config)
        .await?
        .id;
    docker.start_container::<String>(&id, None).await?;

    let exec = docker
        .create_exec(
            &id,
            CreateExecOptions {
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                attach_stdin: Some(true),
                tty: Some(true),
                cmd: Some(vec!["bash"]),
                working_dir: Some("/root"),
                ..Default::default()
            },
        )
        .await?
        .id;
    if let StartExecResults::Attached {
        mut output,
        mut input,
    } = docker.start_exec(&exec, None).await?
    {
        spawn(async move {
            let mut tty = File::from_std(get_tty().unwrap());
            tokio::io::copy(&mut tty, &mut input).await.unwrap();
        });

        let tty_size = terminal_size()?;
        docker
            .resize_exec(
                &exec,
                ResizeExecOptions {
                    height: tty_size.1,
                    width: tty_size.0,
                },
            )
            .await?;

        // set stdout in raw mode so we can do tty stuff
        let stdout = stdout();
        let mut stdout = stdout.lock().into_raw_mode()?;
        let stderr = stderr();
        let mut stderr = stderr.lock().into_raw_mode()?;

        // pipe docker exec output into stdout
        while let Some(Ok(output)) = output.next().await {
            match output {
                LogOutput::StdErr { message } => {
                    stdout.write_all(message.as_ref())?;
                    stdout.flush()?;
                }
                LogOutput::StdOut { message } => {
                    stderr.write_all(message.as_ref())?;
                    stderr.flush()?;
                }
                _ => ()
            }
        }
    }

    docker
        .remove_container(
            &id,
            Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            }),
        )
        .await?;

    exit(0);
}
