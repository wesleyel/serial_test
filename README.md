# SerialPort Test

[![Crates.io Version](https://img.shields.io/crates/v/serialport_test)](https://crates.io/crates/serialport_test)

A comprehensive tool designed for continuous read/write testing of serial devices.

## Features

- **Main Thread Operations**: The main thread is responsible for writing test commands to the serial port.
- **Asynchronous Monitoring**: Utilizes a `tokio` reader thread to continuously monitor the serial port output. Upon receiving the expected output, it promptly notifies the main thread.
- **Logging and Metrics**: Integrated logging and metrics for enhanced monitoring and debugging.
- **Graceful Termination**: Supports stopping the test gracefully using `ctrl+c`.


## Usage

Change `TestSuite` in [src/cli.rs](src/cli.rs) to your desired test suite.

```bash
Usage: serial_test [OPTIONS] <PORT>

Arguments:
  <PORT>  Serial port to connect

Options:
  -b, --baud <BAUD>
          Baud rate [default: 921600]
  -t, --test-seconds <TEST_SECONDS>
          Test total times in seconds [default: 10]
  -i, --interval <INTERVAL>
          Test interval in milliseconds [default: 1000]
      --round-timeout <ROUND_TIMEOUT>
          Round max timeout in milliseconds [default: 600]
      --round-interval <ROUND_INTERVAL>
          Round interval in milliseconds [default: 100]
  -m, --max-fail-count <MAX_FAIL_COUNT>
          Max continuous fail count [default: 3]
  -v, --verbose...
          Increase verbosity
  -s, --test-suite <TEST_SUITE>
          Test suite [default: regular] [possible values: regular, single-bd]
  -h, --help
          Print help
```

## License

This project is licensed under the MIT License.
