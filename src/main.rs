use anyhow::Result;
use clap::Parser;
use repo_doctor::{Cli, run};
use tracing_subscriber::{EnvFilter, fmt};

fn main() -> Result<()> {
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    let output = run(cli)?;
    println!("{output}");

    Ok(())
}
