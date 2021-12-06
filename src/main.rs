
use structopt::StructOpt;

mod cli;
use cli::Cli;

mod config;
use config::Config;

mod util;

mod url;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::from_args();
    let config = Config::read_config()?;
    config.do_it_all(&args)?;


    Ok(())
}
