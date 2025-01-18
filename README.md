# Serial Test

This serial test tool is aimed for continuous read/write testing of serial devices.

## Features

- Main thread to write test command to the serial port.
- A `tokio` reader thread continuely spy on the serial port output, when expected output is received, it will notify the main thread.
- Add properly logging and metrics.
- Add ctrl+c to stop the test.


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
