use futures::stream::StreamExt;
use std::{env, io, str};
use tokio_util::codec::{Decoder, Encoder};

use bytes::BytesMut;
use tokio_serial::SerialPortBuilderExt;

const DEFAULT_TTY: &str = "/dev/ttyS3";
const DEFAULT_BAUD: u32 = 921600;

struct LineCodec;

impl Decoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let newline = src.as_ref().iter().position(|b| *b == b'\n');
        if let Some(n) = newline {
            let line = src.split_to(n + 1);
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

    fn encode(&mut self, _item: String, _dst: &mut BytesMut) -> Result<(), Self::Error> {
        Ok(())
    }
}

struct HexCodec;

impl Decoder for HexCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let hex = src
            .as_ref()
            .iter()
            .map(|b| {
                let char_representation = if b.is_ascii_graphic() {
                    *b as char
                } else {
                    '.'
                };
                format!("{}({:02x})", char_representation, b)
            })
            .collect::<String>();
        Ok(Some(hex))
    }
}

impl Encoder<String> for HexCodec {
    type Error = io::Error;

    fn encode(&mut self, _item: String, _dst: &mut BytesMut) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> tokio_serial::Result<()> {
    let mut args = env::args();
    let tty_path = args.nth(1).unwrap_or_else(|| DEFAULT_TTY.into());

    let mut port = tokio_serial::new(tty_path, DEFAULT_BAUD).open_native_async()?;

    port.set_exclusive(false)
        .expect("Unable to set serial port exclusive to false");

    // let mut reader = LineCodec.framed(port);

    // while let Some(line_result) = reader.next().await {
    //     let line = line_result.expect("Failed to read line");
    //     println!("{}", line);
    // }
    let mut reader = HexCodec.framed(port);
    while let Some(hex_result) = reader.next().await {
        let hex = hex_result.expect("Failed to read hex");
        println!("{}", hex);
    }
    Ok(())
}
