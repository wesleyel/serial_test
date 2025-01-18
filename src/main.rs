mod cli;
mod codec;

use cli::TestCase;
use tokio_util::codec::Decoder;

use futures::{stream::StreamExt, SinkExt};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::codec::LineCodec;
use tokio_serial::SerialPortBuilderExt;

fn init_port(opts: &cli::Options) -> anyhow::Result<tokio_serial::SerialStream> {
    let mut port = tokio_serial::new(&opts.port, opts.baud).open_native_async()?;
    port.set_exclusive(false)
        .expect("Unable to set serial port exclusive to false");
    Ok(port)
}

// async fn round_test(opts: &cli::Options, test_ok: &Arc<RwLock<bool>>, writer: &mut impl Sink<String>) -> anyhow::Result<()> {
//     let testcase: TestCase = opts.test_suite.into();
//     writer.send(testcase.command.to_string()).await?;
//     {
//         let mut state = test_ok.write().await;
//         *state = false;
//     }
//     Ok(())
// }

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let opts = cli::parse_options();
    log::info!("Options: {:#?}", opts);

    let port = init_port(&opts)?;
    let child_alive = Arc::new(RwLock::new(false));
    let test_ok = Arc::new(RwLock::new(false));

    let (mut writer, mut reader) = LineCodec.framed(port).split();

    let test_ok_clone = test_ok.clone();
    let child_alive_clone = child_alive.clone();
    let testcase: TestCase = opts.test_suite.into();
    let reader_handle = tokio::spawn(async move {
        loop {
            if *child_alive_clone.read().await {
                log::info!("Child thread exited");
                break;
            }
            if let Some(Ok(line)) = reader.next().await {
                if line.contains(&testcase.expected) {
                    log::debug!("[child] Test passed");
                    let mut state = test_ok_clone.write().await;
                    *state = true;
                } else {
                    log::trace!("[child] Line not expected: {}", line);
                }
            } else {
                log::debug!("[child] Read error");
            }
        }
    });

    let mut total = 0;
    let mut success = 0;
    let current_time = tokio::time::Instant::now();
    // loop for total test seconds
    loop {
        if current_time.elapsed() > tokio::time::Duration::from_secs(opts.test_seconds as u64) {
            log::info!("Test finished in {} seconds", opts.test_seconds);
            break;
        }

        total += 1;
        writer.send(testcase.command.to_string()).await?;
        {
            let mut state = test_ok.write().await;
            *state = false;
        }
        let round_current = tokio::time::Instant::now();
        // loop for each round
        loop {
            if *test_ok.read().await {
                log::info!("Test passed");
                success += 1;
                break;
            }
            if round_current.elapsed()
                > tokio::time::Duration::from_millis(opts.round_timeout as u64)
            {
                log::debug!("Round timeout");
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(
                opts.round_interval as u64,
            ))
            .await;
        }
    }
    {
        let mut state = child_alive.write().await;
        *state = true;
    }

    reader_handle.await?;
    log::info!("Test passed: {} / {}", success, total);
    Ok(())
}
