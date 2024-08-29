use anyhow::Result;
use clap::StructOpt;

use toipe::config::ToipeConfig;
use toipe::Toipe;

fn main() -> Result<()> {
    let config = ToipeConfig::parse();
    let mut tty = toipe::tty::Tty::new(&config)?;
    let mut toipe = Toipe::new(config)?;
    toipe.run(&mut tty)?;
    Ok(())
}
