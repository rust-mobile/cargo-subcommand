use cargo_subcommand::Subcommand;

fn main() {
    let cmd = Subcommand::new(std::env::args(), "subcommand", |_, _| Ok(false)).unwrap();
    println!("{:#?}", cmd);
}
