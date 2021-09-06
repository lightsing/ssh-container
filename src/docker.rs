use std::hash::Hash;
use std::io::{stderr, stdout, Write};

use bollard::container::{CreateContainerOptions, LogOutput, RemoveContainerOptions};
use bollard::exec::{CreateExecOptions, ResizeExecOptions, StartExecResults};
use bollard::image::CreateImageOptions;
use bollard::{container, Docker};
use futures_util::{StreamExt, TryStreamExt};
use once_cell::sync::Lazy;
use serde::Serialize;
use termion::raw::IntoRawMode;
use termion::{get_tty, terminal_size};
use tokio::fs::File;
use tokio::task::spawn;

pub static DOCKER: Lazy<Docker> = Lazy::new(|| Docker::connect_with_unix_defaults().unwrap());

#[derive(Debug)]
pub struct Container<'d> {
    id: String,
    docker: &'d Docker,
}

#[derive(Debug)]
pub struct Exec<'d> {
    id: String,
    docker: &'d Docker,
}

pub async fn pull<T>(image: T) -> Result<(), bollard::errors::Error>
where
    T: Into<String> + Serialize + Default,
{
    DOCKER
        .create_image(
            Some(CreateImageOptions {
                from_image: image,
                ..Default::default()
            }),
            None,
            None,
        )
        .try_collect::<Vec<_>>()
        .await?;
    Ok(())
}

impl<'d> Container<'d> {
    pub async fn new<T, N>(
        config: container::Config<T>,
        options: Option<CreateContainerOptions<N>>,
        docker: Option<&'d Docker>,
    ) -> Result<Container<'d>, bollard::errors::Error>
    where
        T: Into<String> + Eq + Hash + Serialize,
        N: Into<String> + Serialize,
    {
        let docker = match docker {
            None => default(),
            Some(docker) => docker,
        };
        let id = docker.create_container::<N, T>(options, config).await?.id;
        Ok(Self { id, docker })
    }

    pub async fn start(&self) -> Result<(), bollard::errors::Error> {
        self.docker.start_container::<String>(&self.id, None).await
    }

    pub async fn exec<T>(
        &self,
        options: CreateExecOptions<T>,
    ) -> Result<Exec<'d>, bollard::errors::Error>
    where
        T: Into<String> + Serialize,
    {
        let response = self.docker.create_exec(&self.id, options).await?;
        Ok(Exec {
            id: response.id,
            docker: self.docker,
        })
    }

    pub async fn remove(self) -> Result<(), bollard::errors::Error> {
        self.docker
            .remove_container(
                &self.id,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await?;
        Ok(())
    }
}

impl<'d> Exec<'d> {
    pub async fn attach(&self) -> Result<(), bollard::errors::Error> {
        if let StartExecResults::Attached {
            mut output,
            mut input,
        } = self.docker.start_exec(&self.id, None).await?
        {
            spawn(async move {
                let mut tty = File::from_std(get_tty().unwrap());
                tokio::io::copy(&mut tty, &mut input).await.unwrap();
            });

            let tty_size = terminal_size()?;
            self.docker
                .resize_exec(
                    &self.id,
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
                    _ => (),
                }
            }
        }
        Ok(())
    }
}
