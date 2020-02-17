use cargo_subcommand::Subcommand;

fn main() {
    let cmd = Subcommand::new("subcommand", |_, _| Ok(false)).unwrap();
    match cmd.cmd() {
        "list" => {
            for artifact in cmd.artifacts() {
                println!("{:?}", artifact);
            }
        }
        _ => {}
    }
}
