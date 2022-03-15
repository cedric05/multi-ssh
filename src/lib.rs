use openssh::*;
use serde::{Deserialize, Serialize};
use std::io::ErrorKind::InvalidInput;
use std::{error::Error, path::Path, process::Output};
use tokio::io::{stderr, stdout, AsyncWriteExt};

#[derive(Serialize, Deserialize, Debug)]

pub struct Node {
    pub public_address: String,
    pub tag: String,
    pub keyfile: Option<String>,
}

#[derive(Debug)]
pub struct NodeSession {
    pub node: Node,
    pub session: Session,
}

impl NodeSession {
    pub async fn run_command(&mut self, command: &str) -> Result<(), Box<dyn Error>> {
        let status = self.session.raw_command(command).output().await;
        match status {
            Ok(output) => {
                self.print_output(output).await?;
                Ok(())
            }
            Err(openssh::Error::Disconnected) => {
                println!(
                    "node: {} disconnected! trying to connect again.",
                    self.node.public_address
                );
                self.session = self.node.connect().await?;
                self.print_output(self.session.raw_command(command).output().await?)
                    .await?;
                Ok(())
            }
            Err(err) => Err(err)?,
        }
    }

    pub async fn copy_file(&self, file: &Path) -> Result<(), Box<dyn Error>> {
        // need to confirm everytime.
        // because of immutable copy
        let remote_file_path = &file.file_name().ok_or(std::io::Error::from(InvalidInput))?;
        // if self.session.check().await.is_err() {
        //     self.session = self.node.connect().await?
        // }
        let mut remote_file = self.session.sftp().write_to(remote_file_path).await?;
        let mut local_file = tokio::fs::File::open(file).await?;
        tokio::io::copy(&mut local_file, &mut remote_file).await?;
        println!(
            "{:?} copied to {}",
            remote_file_path, self.node.public_address
        );
        Ok(())
    }

    pub async fn close(self) -> Result<(), openssh::Error> {
        self.session.close().await
    }

    async fn print_output(&self, output: Output) -> Result<(), Box<dyn Error>> {
        println!(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━     {}    ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            self.node.public_address
        );
        if !output.stdout.is_empty() {
            stdout().write_all(&output.stdout).await?;
        }
        if !output.stderr.is_empty() {
            stderr().write_all(&output.stdout).await?;
        }
        Ok(())
    }
}

impl Node {
    pub async fn get_node_session(self) -> Result<NodeSession, openssh::Error> {
        self.connect().await.map(|session| NodeSession {
            session: session,
            node: self,
        })
    }

    pub async fn connect(&self) -> Result<Session, openssh::Error> {
        let mut session_builder = SessionBuilder::default();
        if self.keyfile.is_some() {
            session_builder.keyfile(&self.keyfile.as_ref().unwrap());
        }
        session_builder.known_hosts_check(KnownHosts::Accept);
        let session = session_builder
            .connect(format!("{}", &self.public_address))
            .await?;
        println!("session connected: {}", self.public_address);
        Ok(session)
    }
}
