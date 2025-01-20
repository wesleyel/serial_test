use bytes::BytesMut;
use std::io;
use tokio_util::codec::{Decoder, Encoder};

pub struct LineCodec;

impl Decoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let newline = src.iter().position(|b| *b == b'\n');
        if let Some(n) = newline {
            let line = src.split_to(n + 1);
            log::trace!("Received: {}", String::from_utf8_lossy(line.as_ref()));
            return match std::str::from_utf8(line.as_ref()) {
                Ok(s) => Ok(Some(s.to_string())),
                Err(_) => Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Invalid String: {:?}", line.as_ref()),
                )),
            };
        }
        if src.len() > 256 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Buffer too long: {:?}", src.len()),
            ));
        }
        Ok(None)
    }
}

impl Encoder<String> for LineCodec {
    type Error = io::Error;

    fn encode(&mut self, item: String, dst: &mut BytesMut) -> Result<(), Self::Error> {
        log::trace!("Sending: {:?}", item);
        if item.ends_with("\r\n") {
            dst.extend_from_slice(item.as_bytes());
        } else {
            dst.extend_from_slice(item.as_bytes());
            dst.extend_from_slice(b"\r\n");
        }
        Ok(())
    }
}
