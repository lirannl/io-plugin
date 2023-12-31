use serde::{Deserialize, Serialize};
use serde_json::{from_slice, to_vec};
use std::{
    error::Error,
    io::{Read, Write},
    pin::Pin,
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::IOPluginError;

const BUF_SIZE: usize = 100;

pub fn io_read<T: for<'a> Deserialize<'a>>(source: &mut dyn Read) -> Result<T, Box<dyn Error>> {
    let mut vec = Vec::<u8>::new();
    let mut buf = [0; BUF_SIZE];
    let mut size = BUF_SIZE;
    while size == BUF_SIZE {
        size = match source.read(&mut buf) {
            Ok(0) => Err(IOPluginError::PipeClosed.into()),
            Err(err) => Err(err.into()),
            Ok(read) => Ok::<_, Box<dyn Error>>(read),
        }?;
        vec.extend(&buf[..size]);
    }
    Ok(from_slice(&vec)?)
}

pub fn io_write<T: Serialize>(sink: &mut dyn Write, message: T) -> Result<(), Box<dyn Error>> {
    let message = to_vec(&message)?;
    sink.write_all(&message)?;
    sink.flush()?;
    Ok(())
}

pub async fn io_read_async<T: for<'a> Deserialize<'a>>(
    mut source: Pin<&mut dyn AsyncRead>,
) -> Result<T, Box<dyn Error>> {
    let mut vec = Vec::<u8>::new();
    let mut buf = [0; BUF_SIZE];
    let mut size = BUF_SIZE;
    while size == BUF_SIZE {
        size = match source.read(&mut buf).await {
            Ok(0) => Err(IOPluginError::PipeClosed.into()),
            Err(err) => Err(err.into()),
            Ok(read) => Ok::<_, Box<dyn Error>>(read),
        }?;
        vec.extend(&buf[..size]);
    }
    Ok(from_slice(&vec)?)
}

pub async fn io_write_async<T: Serialize>(
    mut sink: Pin<&mut dyn AsyncWrite>,
    message: T,
) -> Result<(), Box<dyn Error>> {
    let message = to_vec(&message)?;
    sink.write_all(&message).await?;
    sink.flush().await?;
    Ok(())
}
