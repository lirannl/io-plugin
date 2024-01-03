#![feature(async_closure)]
use io_plugin_example::{Error, ExamplePluginHandle};
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    error::Error as StdError, path::PathBuf, process::Stdio as StdioBehaviour, str::FromStr,
};
use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    main,
    process::Command,
};

#[main]
async fn main() {
    let mut plugin = (async || -> Result<_, Box<dyn StdError>> {
        let path = PathBuf::from_str("target/debug/plugin-example")?;
        let process = Command::new(path)
            .stdin(StdioBehaviour::piped())
            .stdout(StdioBehaviour::piped())
            .spawn()?;
        Ok(ExamplePluginHandle::new(process).await?)
    })()
    .await
    .unwrap();
    println!("Welcome! Input desired action here:");
    while let Ok(Some(line)) = BufReader::new(stdin()).lines().next_line().await {
        if line == "exit" {
            break;
        }
        react_to_line(line, &mut plugin)
            .await
            .unwrap_or_else(|e| eprintln!("{e:#?}"));
        println!("\nInput desired action here:");
    }
}

lazy_static! {
    static ref NUMS_PARSER: Regex = Regex::new(r"-?[\d]+(:?\.\d+)?").unwrap();
}

async fn react_to_line(
    line: String,
    plugin: &mut ExamplePluginHandle,
) -> Result<(), Box<dyn StdError>> {
    let nums = NUMS_PARSER
        .find_iter(&line)
        .into_iter()
        .filter_map(|c| Some(c.as_str().parse::<f64>().ok()?))
        .collect::<Vec<_>>();
    if line == "get" {
        println!("State is: {}", plugin.get_state().await?);
        return Ok(());
    } else if line.starts_with("request_bytes ") {
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
    } else if line.starts_with("set ") {
        plugin
            .set_state(
                nums.get(0)
                    .ok_or(Error::Generic("No new state provided.".to_string()))?
                    .to_string()
                    .parse()?,
            )
            .await?;
        return Ok(());
    };
    if let [n1, n2] = nums[..] {
        let result = plugin.op::<f64>(n1, n2).await?;
        println!("Result: {result}");
    } else {
    }
    Ok(())
}
