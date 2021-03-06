use crate::package::Package;
use clap::{App, Arg, ArgMatches};
use std::error::Error;

fn args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.about("Install packages").arg(
        Arg::with_name("package")
            .required(true)
            .multiple(true)
            .index(1),
    )
}

fn run(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let packages = args
        .values_of("package")
        .unwrap()
        .map(Package::parse)
        .collect::<Vec<_>>();

    dbg!(packages);
    Ok(())
}

pub static CMD: crate::SubCommand<()> = crate::SubCommand { args, run };
