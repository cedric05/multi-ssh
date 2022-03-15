use openssh::*;

use serde::{Deserialize, Serialize};
use std::{error::Error, path::Path};
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
                println!(
                    "**********************************       {}           *****************************",
                    self.node.public_address
                );
                if !output.stdout.is_empty() {
                    stdout().write_all(&output.stdout).await?;
                }
                if !output.stderr.is_empty() {
                    stderr().write_all(&output.stdout).await?;
                }
                println!("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",);
            }
            Err(err) => match err {
                openssh::Error::Disconnected => {
                    println!(
                        "node: {} disconnected! trying to connect again.",
                        self.node.public_address
                    );
                    let node_session = self.node.connect().await?;
                    println!("node: {} connected", self.node.public_address);
                    self.session = node_session;
                }
                _ => println!("unhanlded error: {}", err),
            },
        }
        Ok(())
    }

    pub async fn copy_file(&self, file: &Path) -> Result<(), Box<dyn Error>> {
        let mut remote_file = self.session.sftp().write_to(file.file_name().unwrap()).await?;
        let mut local_file = tokio::fs::File::open(file).await?;
        tokio::io::copy(&mut local_file, &mut remote_file).await?;
        println!("{:?} copied to {}", file.file_name().as_ref().unwrap(), self.node.public_address);
        Ok(())
    }

    pub async fn close(self) -> Result<(), openssh::Error> {
        self.session.close().await
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
