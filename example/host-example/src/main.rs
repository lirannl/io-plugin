#![feature(async_closure)]
use io_plugin_example::{Error, ExamplePluginHandle, ExamplePluginMessage, ExamplePluginResponse};
use lazy_static::lazy_static;
use regex::Regex;
use rmp_serde::{from_slice, to_vec};
use std::{error::Error as StdError, io::ErrorKind, process::Stdio as StdioBehaviour};
use tokio::{
    io::{stdin, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    main,
    process::{Child, ChildStdin, ChildStdout, Command},
    runtime::Handle as Runtime,
    sync::Mutex,
};

struct Stdio {
    stdin: ChildStdin,
    stdout: ChildStdout,
}

struct Handle {
    kill_fn: Box<dyn FnMut() -> ()>,
    stdio: Mutex<Stdio>,
}
impl Handle {
    fn new(mut process: Child) -> Result<Self, Error> {
        let stdio = process
            .stdin
            .take()
            .and_then(|stdin| {
                Some(Stdio {
                    stdin,
                    stdout: process.stdout.take()?,
                })
            })
            .ok_or(Error::InitialisationError(
                "Stdin/stdout have not been piped".to_string(),
            ));
        Ok(Handle {
            kill_fn: Box::new(move || {
                Runtime::try_current()
                    .unwrap()
                    .block_on(process.kill())
                    .unwrap()
            }),
            stdio: Mutex::new(stdio?),
        })
    }
}
impl Drop for Handle {
    fn drop(&mut self) {
        (self.kill_fn)()
    }
}
const BUF_SIZE: usize = 8000;
impl ExamplePluginHandle for Handle {
    async fn message(
        &mut self,
        message: ExamplePluginMessage,
    ) -> Result<ExamplePluginResponse, Box<dyn StdError>> {
        let mut stdio = self.stdio.lock().await;
        stdio.stdin.write(&to_vec(&message)?).await?;
        let buf = {
            let mut vec = Vec::new();

            // Continue trying to read until the buffer doesn't fill completely
            let mut buf = [0; BUF_SIZE];
            let mut read = stdio.stdout.read(&mut buf).await?;
            while read == BUF_SIZE {
                vec.copy_from_slice(&buf[..read]);
                read = stdio.stdout.read(&mut buf).await.or_else(|err| {
                    if err.kind() == ErrorKind::UnexpectedEof {
                        Ok(0)
                    } else {
                        Err(err)
                    }
                })?;
            }

            vec
        };
        Ok(from_slice(&buf)?)
    }
}

#[main]
async fn main() {
    let mut plugin = (async || -> Result<_, Box<dyn StdError>> {
        let plugin = Command::new("./plugin-example")
            .stdin(StdioBehaviour::piped())
            .stdout(StdioBehaviour::piped())
            .spawn()?;
        Ok(Handle::new(plugin)?)
    })()
    .await
    .unwrap();

    while let Ok(Some(line)) = BufReader::new(stdin()).lines().next_line().await {
        if line == "exit" {
            break;
        }
        react_to_line(line, &mut plugin)
            .await
            .unwrap_or_else(|e| eprintln!("{e:#?}"));
    }
}

lazy_static! {
    static ref NUMS_PARSER: Regex = Regex::new(r"-?[\d]+(:?\.\d+)?").unwrap();
}

async fn react_to_line(line: String, plugin: &mut Handle) -> Result<(), Box<dyn StdError>> {
    let nums = NUMS_PARSER
        .find_iter(&line)
        .into_iter()
        .filter_map(|c| Some(c.as_str().parse::<f64>().ok()?))
        .collect::<Vec<_>>();
    if line.starts_with("request_bytes ") {
        let bytes = plugin
            .random_bytes(
                nums.get(0)
                    .ok_or(Error::Generic(
                        "You must specify the amount of bytes desired.".to_string(),
                    ))?
                    .to_string()
                    .parse()?,
            )
            .await?;
        println!("Got {} bytes!", bytes.len());
        return Ok(());
    };
    if let [n1, n2] = nums[..] {
        let result = plugin.float_op(n1, n2).await?;
        println!("Result: {result}");
    } else {
    }
    Ok(())
}
