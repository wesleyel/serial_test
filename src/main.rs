use futures::{stream::StreamExt, Sink, SinkExt, Stream};
use std::{env, io, str, sync::Arc};
use tokio::sync::RwLock;
use tokio_util::codec::{Decoder, Encoder, Framed};

use bytes::BytesMut;
use tokio_serial::SerialPortBuilderExt;

const DEFAULT_TTY: &str = "/dev/ttyS3";
const DEFAULT_BAUD: u32 = 921600;
const TEST_CMD: &str = "$QXMONCSTM\r\n";
const EXPECTED_RESP: &str = "QXMONCSTM,BG1101";

struct LineCodec;

impl Decoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let newline = src.as_ref().windows(2).position(|w| w == b"\r\n");
        if let Some(n) = newline {
            let line = src.split_to(n + 2);
            return match str::from_utf8(line.as_ref()) {
                Ok(s) => Ok(Some(s.to_string())),
                Err(_) => Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Invalid String: {:#?}", line.as_ref()),
                )),
            };
        }
        Ok(None)
    }
}

impl Encoder<String> for LineCodec {
    type Error = io::Error;

    fn encode(&mut self, item: String, dst: &mut BytesMut) -> Result<(), Self::Error> {
        if item.ends_with("\r\n") {
            dst.extend_from_slice(item.as_bytes());
        } else {
            dst.extend_from_slice(item.as_bytes());
            dst.extend_from_slice(b"\r\n");
        }
        Ok(())
    }
}
#[allow(dead_code)]
async fn test_once<W, R>(
    writer: &mut W,
    reader: &mut R,
    cmd: &str,
    expected: &str,
) -> anyhow::Result<()>
where
    W: Sink<String, Error = io::Error> + Unpin,
    R: Stream<Item = Result<String, io::Error>> + Unpin,
{
    writer.send(cmd.to_string()).await?;
    let line = reader.next().await.expect("Failed to read line");
    match line {
        Ok(line) => {
            if line.contains(expected) {
                return Ok(());
            } else {
                return Err(anyhow::anyhow!("Unexpected response: {}", line));
            }
        }
        Err(e) => return Err(anyhow::anyhow!("Failed to read line: {}", e)),
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let mut args = env::args();
    let callback_lock = Arc::new(RwLock::new(false));
    let tty_path = args.nth(1).unwrap_or_else(|| DEFAULT_TTY.into());
    let mut port = tokio_serial::new(tty_path, DEFAULT_BAUD).open_native_async()?;
    port.set_exclusive(false)
        .expect("Unable to set serial port exclusive to false");

    let (mut writer, mut reader) = LineCodec.framed(port).split();

    let callback_lock_child = callback_lock.clone();
    // 子线程负责读取串口消息
    let reader_handle = tokio::spawn(async move {
        while let Some(Ok(line)) = reader.next().await {
            println!("Received line: {}", line);
            if line.contains(EXPECTED_RESP) {
                println!("Callback received");
                let mut callback = callback_lock_child.write().await;
                *callback = true;
            }
        }
    });

    for _ in 0..10 {
        writer.send(TEST_CMD.to_string()).await?;
        let current_time = tokio::time::Instant::now();
        loop {
            let callback = callback_lock.read().await;
            if *callback {
                println!("Callback received");
                let mut callback = callback_lock.write().await;
                *callback = false;
                break;
            }
            if current_time.elapsed() > tokio::time::Duration::from_secs(10) {
                return Err(anyhow::anyhow!("Timeout"));
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    reader_handle.await?;
    println!("Test passed");
    Ok(())
}
