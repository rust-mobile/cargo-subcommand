use cargo_subcommand::{Args, Subcommand};
use clap::Parser;

fn main() {
    let args = Args::parse();
    let cmd = Subcommand::new(args).unwrap();
    println!("{cmd:#?}");
}
