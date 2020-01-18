use super::SubCommand;
use clap::{App, Arg, ArgGroup, ArgMatches};
use std::error::Error;

fn args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.about("List installed/built packages")
        .arg(
            Arg::with_name("all")
                .long("all")
                .short("a")
                .help("List all packages in the repositories"),
        )
        .arg(
            Arg::with_name("built")
                .long("built")
                .short("b")
                .help("List packages with saved builds"),
        )
        .arg(
            Arg::with_name("installed")
                .long("installed")
                .short("i")
                .help("List currently installed packages"),
        )
        .group(
            ArgGroup::with_name("type")
                .args(&["all", "built", "installed"])
                .required(false),
        )
    /*.arg(
        Arg::with_name("glob")
            .index(1),
    )*/
}

fn run(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
    println!("list");
    dbg!(args);
    Ok(())
}

pub static CMD: SubCommand = SubCommand { args, run };
