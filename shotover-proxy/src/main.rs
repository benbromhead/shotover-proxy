#![warn(rust_2018_idioms)]
#![recursion_limit = "256"]

use anyhow::Result;
use clap::Clap;

use shotover_proxy::runner::{ConfigOpts, Runner};

fn main() -> Result<()> {
    Runner::new(ConfigOpts::parse())?
        .with_logging()?
        .run_block()
}
