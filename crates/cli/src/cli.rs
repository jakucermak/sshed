use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// config file path
    #[arg(short, long)]
    config: String,
}

pub fn parse_args() {
    println!("Parsing args");
    Args::parse();
}
