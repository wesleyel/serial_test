use clap::{Parser, ValueEnum};

#[derive(Debug, Clone)]
pub struct TestCase {
    pub command: String,
    pub expected: String,
}

#[derive(ValueEnum, Debug, Clone)]
pub enum TestSuite {
    Regular,
    SingleBD,
}

impl From<TestSuite> for TestCase {
    fn from(test_suite: TestSuite) -> Self {
        match test_suite {
            TestSuite::Regular => TestCase {
                command: "$QXMONCSTM".to_string(),
                expected: "QXMONCSTM,BG1101".to_string(),
            },
            TestSuite::SingleBD => TestCase {
                command: "$QXMON".to_string(),
                expected: "QXMON,BG1101".to_string(),
            },
        }
    }
}

/// A comprehensive tool designed for continuous read/write testing of serial devices.
#[derive(Parser, Debug, Clone)]
#[command(version = env!("GIT_VERSION"))]
pub struct Options {
    /// Serial port to connect
    pub port: String,

    /// Baud rate
    #[arg(short, long, default_value = "921600")]
    pub baud: u32,

    /// Test total times in seconds
    #[arg(short, long, default_value = "10")]
    pub test_seconds: u64,

    /// Test interval in milliseconds
    #[arg(short, long, default_value = "1000")]
    pub interval: u64,

    /// Round max timeout in milliseconds
    #[arg(long, default_value = "30")]
    pub round_timeout: u64,

    /// Round interval in milliseconds
    #[arg(long, default_value = "5")]
    pub round_interval: u64,

    /// Max continuous fail count
    #[arg(short, long, default_value = "5")]
    pub max_fail_count: u32,

    /// Increase verbosity
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Test suite
    #[arg(short = 's', long, default_value = "regular")]
    pub test_suite: TestSuite,
}

pub fn parse_options() -> Options {
    let opts = Options::parse();
    let debug_level = match opts.verbose {
        0 => log::Level::Info,
        1 => log::Level::Debug,
        _ => log::Level::Trace,
    };
    simple_logger::init_with_level(debug_level).unwrap();
    opts
}
