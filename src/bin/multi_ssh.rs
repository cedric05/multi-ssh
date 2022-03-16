use clap::Parser;
use futures::future::join_all;

use rustyline::{error::ReadlineError, Editor};
use std::{error::Error, path::Path};

/// Simple multi-ssh command runner for windows dude
#[derive(Parser)]
#[clap(about, version, author)]
pub struct Args {
    /// config file to connect
    /// For example:
    ///   {
    ///      "public_address": "171.31.0.3",
    ///      "keyfile": "~/.ssh/id_rsa",
    ///      "tag": "cluster-alpha",
    ///   }
    ///`
    #[clap(short, long, default_value = "node_config.json")]
    pub config: String,

    /// filter nodes in above config files
    /// example: `cluster-alpha`
    #[clap(short, long)]
    pub tag: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let node_config_file = std::fs::File::options().read(true).open(args.config)?;
    let node_list: Vec<multi_ssh::Node> = serde_json::from_reader(node_config_file)?;

    let has_filter = args.tag.is_some();

    let filter = if has_filter {
        args.tag.unwrap()
    } else {
        String::new()
    };

    let mut all_sessions: Vec<multi_ssh::NodeSession> = join_all(
        node_list
            .into_iter()
            .filter(|x| if has_filter { x.tag == filter } else { true })
            .map(|x| x.get_node_session()),
    )
    .await
    .into_iter()
    .inspect(|x| {
        if x.is_err() {
            println!("Error: {:?}", x.as_ref().unwrap_err())
        }
    })
    .flatten()
    .collect();

    if all_sessions.is_empty() {
        println!("Not able to connect to even a single server, exiting");
        return Ok(());
    }
    println!("sessions created for all ips");

    let mut rl = Editor::<()>::new();
    let history_location = format!("{}", shellexpand::tilde(&format!("~/.ochestra.txt")));
    if rl.load_history(&history_location).is_err() {
        // println!("No previous history.");
    }
    loop {
        let readline = rl.readline("multi-ssh>> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                if !line.is_empty() {
                    let expand = shell_words::split(&line);
                    match expand {
                        Ok(expand)
                            if expand.len() > 0 && (expand[0] == ".copy" && expand[0] == ".cp") =>
                        {
                            // copy files
                            let mut only_files = vec![];
                            for file in line.split(",") {
                                let file = Path::new(file);
                                if file.exists() && file.is_file() {
                                    only_files.push(file);
                                } else {
                                    println!("Error: {:?} doesn't exist or not a file, will continue on to next file", file);
                                }
                            }
                            if only_files.is_empty() {
                                println!(
                                    "Error: No files to copy, ReEnter files with `,` seperated"
                                );
                            }
                            join_all(only_files.iter().map(|file| {
                                join_all(all_sessions.iter().map(|x| x.copy_file(&file)))
                            }))
                            .await;
                        }
                        _ => {
                            // run command
                            join_all(all_sessions.iter_mut().map(|x| x.run_command(&line))).await;
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                continue;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    join_all(all_sessions.into_iter().map(|x| x.close())).await;
    rl.save_history(&history_location)?;
    Ok(())
}
