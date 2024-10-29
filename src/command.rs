use std::io::Write;
use std::process;
use std::process::Stdio;

use crate::git_commit::GitCommit;
use crate::provider::AIProvider;
use crate::provider::LumenProvider;

use spinoff::{spinners, Color, Spinner};

pub struct LumenCommand {
    provider: LumenProvider,
}

impl LumenCommand {
    pub fn new(provider: LumenProvider) -> Self {
        LumenCommand { provider }
    }

    pub async fn explain(&self, sha: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut spinner = Spinner::new(spinners::Dots, "Loading", Color::Blue);
        let commit = GitCommit::new(sha.clone());
        let result = self.provider.explain(commit.clone()).await?;

        let result = format!(
            "commit {}\nAuthor: {} <{}>\nDate: {}\n\n{}\n-----\n{}",
            commit.sha,
            commit.author_name,
            commit.author_email,
            commit.date,
            commit.message,
            result
        );

        spinner.success("Done");

        // attempt to format using mdcat
        let mut mdcat = std::process::Command::new("mdcat")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let _source = std::process::Command::new("echo")
            .arg(result)
            .stdout(mdcat.stdin.take().unwrap())
            .spawn()
            .unwrap();

        let output = mdcat.wait_with_output().unwrap();

        println!("{}", String::from_utf8(output.stdout).unwrap());

        Ok(())
    }

    pub async fn list(&self) -> Result<(), Box<dyn std::error::Error>> {
        let command = "git log --color=always --format='%C(auto)%h%d %s %C(black)%C(bold)%cr' | fzf --ansi --no-sort --reverse --bind='enter:become(echo {1})' --wrap";

        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .output()
            .expect("Failed to execute command");

        if !output.status.success() {
            eprintln!("Command failed with status: {:?}", output.status);
            process::exit(1);
        }

        let mut sha = String::from_utf8(output.stdout).unwrap();
        sha.pop(); // remove trailing newline from echo

        self.explain(sha).await
    }
}
