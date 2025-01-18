mod cli;
mod codec;

use cli::TestCase;
use tokio_util::codec::Decoder;

use futures::{sink::SinkExt, stream::StreamExt};
use std::{cell::Cell, process::ExitCode, sync::Arc};
use tokio::{
    signal,
    sync::{Mutex, RwLock},
};

use crate::codec::LineCodec;
use tokio_serial::SerialPortBuilderExt;

fn init_port(opts: &cli::Options) -> anyhow::Result<tokio_serial::SerialStream> {
    let mut port = tokio_serial::new(&opts.port, opts.baud).open_native_async()?;
    port.set_exclusive(false)
        .expect("Unable to set serial port exclusive to false");
    Ok(port)
}

async fn handle_reader<T>(
    mut reader: T,
    testcase: TestCase,
    test_ok: Arc<RwLock<bool>>,
    child_alive: Arc<RwLock<bool>>,
) where
    T: StreamExt<Item = Result<String, std::io::Error>> + std::marker::Unpin,
{
    loop {
        if *child_alive.read().await {
            log::info!("Child thread exited");
            break;
        }
        if let Some(Ok(line)) = reader.next().await {
            if line.contains(&testcase.expected) {
                log::debug!("[child] Test passed");
                let mut state = test_ok.write().await;
                *state = true;
            } else {
                log::trace!("[child] Line not expected: {}", line);
            }
        } else {
            log::debug!("[child] Read error");
        }
    }
}

async fn round_test<T>(
    opts: &cli::Options,
    test_ok: &Arc<RwLock<bool>>,
    writer: &mut T,
    testcase: TestCase,
) -> bool
where
    T: SinkExt<String, Error = std::io::Error> + std::marker::Unpin,
{
    if let Err(e) = writer.send(testcase.command.to_string()).await {
        log::error!("[main] send command error: {}", e);
        return false;
    }
    {
        let mut state = test_ok.write().await;
        *state = false;
    }
    let round_current = tokio::time::Instant::now();
    loop {
        if *test_ok.read().await {
            log::info!("[main] Test passed");
            return true;
        }
        if round_current.elapsed() > tokio::time::Duration::from_millis(opts.round_timeout) {
            log::debug!("[main] Round timeout");
            return false;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(opts.round_interval)).await;
    }
}

async fn main_loop<T>(
    opts: &cli::Options,
    test_ok: &Arc<RwLock<bool>>,
    testcase: TestCase,
    writer: &mut T,
    child_alive: Arc<RwLock<bool>>,
) -> anyhow::Result<()>
where
    T: SinkExt<String, Error = std::io::Error> + std::marker::Unpin,
{
    let total = Arc::new(Mutex::new(0));
    let success = Arc::new(Mutex::new(0));
    let continuous_fail = Arc::new(Mutex::new(0));
    let current_time = tokio::time::Instant::now();

    let child_alive_clone = child_alive.clone();
    let total_clone = total.clone();
    let success_clone = success.clone();
    let continuous_fail_clone = continuous_fail.clone();
    tokio::spawn(async move {
        signal::ctrl_c().await.unwrap();
        {
            let mut state = child_alive_clone.write().await;
            *state = true;
        }
        log::info!("[main] Ctrl-C received");
        log::info!("[main] Total: {}", *total_clone.lock().await);
        log::info!("[main] Success: {}", *success_clone.lock().await);
        log::info!(
            "[main] Continuous fail: {}",
            *continuous_fail_clone.lock().await
        );
        std::process::exit(0);
    });

    // loop for total test seconds
    loop {
        if current_time.elapsed() > tokio::time::Duration::from_secs(opts.test_seconds) {
            log::info!("[main] Test finished in {} seconds", opts.test_seconds);
            log::info!(
                "[main] Test passed: {} / {}",
                *success.lock().await,
                *total.lock().await
            );
            return Ok(());
        }

        *total.lock().await += 1;
        if round_test(&opts, &test_ok, writer, testcase.clone()).await {
            *success.lock().await += 1;
            *continuous_fail.lock().await = 0;
        } else {
            *continuous_fail.lock().await += 1;
        }
        if *continuous_fail.lock().await > opts.max_fail_count {
            log::error!(
                "[main] continuous fail {} over max fail {}",
                *continuous_fail.lock().await,
                opts.max_fail_count
            );
            return Err(anyhow::anyhow!("continuous fail over max fail"));
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(opts.interval)).await;
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let opts = cli::parse_options();
    log::debug!("Options: {:#?}", opts);
    let testcase: TestCase = opts.test_suite.clone().into();

    let child_alive = Arc::new(RwLock::new(false));
    let test_ok = Arc::new(RwLock::new(false));

    let port = init_port(&opts).expect("Failed to initialize port");
    let (mut writer, reader) = LineCodec.framed(port).split();

    let reader_handle = tokio::spawn(handle_reader(
        reader,
        testcase.clone(),
        test_ok.clone(),
        child_alive.clone(),
    ));

    let retval = match main_loop(&opts, &test_ok, testcase, &mut writer, child_alive.clone()).await
    {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    };

    {
        let mut state = child_alive.write().await;
        *state = true;
    }

    reader_handle
        .await
        .expect("Failed to wait for reader thread");
    retval
}
