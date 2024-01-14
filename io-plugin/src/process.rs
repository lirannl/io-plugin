use std::process::Stdio;
use std::{io, path::Path};
use tokio::process::{Child, Command};

pub fn spawn_process(path: &Path) -> Result<Child, io::Error> {
    Command::new(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
}