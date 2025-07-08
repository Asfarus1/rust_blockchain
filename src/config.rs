use clap::Parser;

#[derive(Debug, Parser)]
#[command(about, long_about = None)]
pub struct Config {
    #[arg(short, long, env, default_value_t = 3000)]
    pub port: u16,
}
