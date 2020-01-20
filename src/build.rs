use crate::package::Package;
use clap::{App, Arg, ArgMatches};
use std::error::Error;

fn args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.about("Build packages").arg(
        Arg::with_name("package")
            .required(true)
            .multiple(true)
            .index(1),
    )
}

fn run(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let _packages = args
        .values_of("package")
        .unwrap()
        .map(Package::parse)
        .collect::<Vec<_>>();
    Ok(())
}

pub static CMD: crate::SubCommand<()> = crate::SubCommand { args, run };
