use clap::Parser;

#[derive(Parser)]
#[command(name = "retina")]
#[command(about = "A Discord bot modelled after Dyno")]
#[command(version)]
pub struct Cli {
    #[arg(short, long, default_value = "config.toml")]
    pub config: String,
}
