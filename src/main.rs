use inc_20210825::{audit, cleanup};

fn main() {
    let matches = clap::App::new("inc-20210805")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(clap::SubCommand::with_name("audit"))
        .subcommand(clap::SubCommand::with_name("cleanup"))
        .get_matches();

    match matches.subcommand() {
        ("audit", Some(matches)) => {
            audit::run(matches);
        }
        ("cleanup", Some(matches)) => {
            cleanup::run(matches);
        }
        _ => unreachable!(),
    }
}
