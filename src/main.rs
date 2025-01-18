use futures::{stream::StreamExt, Sink, SinkExt, Stream};
use std::{env, io, str};
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
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Invalid String")),
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
    let tty_path = args.nth(1).unwrap_or_else(|| DEFAULT_TTY.into());

    let mut port = tokio_serial::new(tty_path, DEFAULT_BAUD).open_native_async()?;

    port.set_exclusive(false)
        .expect("Unable to set serial port exclusive to false");

    let (mut writer, mut reader) = LineCodec.framed(port).split();

    match test_once(&mut writer, &mut reader, TEST_CMD, EXPECTED_RESP).await {
        Ok(_) => println!("Test passed"),
        Err(e) => println!("Test failed: {}", e),
    }

    Ok(())
}
