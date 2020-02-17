use cargo_subcommand::Subcommand;

fn main() {
    let cmd = Subcommand::new("subcommand", |_, _| Ok(false)).unwrap();
    println!("{:#?}", cmd);
}
