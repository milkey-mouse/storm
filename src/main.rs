use std::{error::Error, process};

use clap::{
    app_from_crate, crate_authors, crate_description, crate_name, crate_version, App, AppSettings,
    ArgMatches,
};
use phf::phf_map;

mod build;
mod install;
mod list;
mod package;
mod uninstall;

pub struct SubCommand {
    pub args: for<'a, 'b> fn(App<'a, 'b>) -> App<'a, 'b>,
    pub run: fn(&ArgMatches) -> Result<(), Box<dyn Error>>,
}

static SUBCOMMANDS: phf::Map<&'static str, &'static SubCommand> = phf_map! {
    "build" => &build::CMD,
    "install" => &install::CMD,
    "list" => &list::CMD,
    "uninstall" => &uninstall::CMD,
};

fn main() {
    let matches = SUBCOMMANDS
        .entries()
        .fold(
            app_from_crate!()
                .setting(AppSettings::SubcommandRequired)
                .global_settings(&[
                    AppSettings::ArgRequiredElseHelp,
                    AppSettings::InferSubcommands,
                    AppSettings::VersionlessSubcommands,
                ]),
            |args, (name, subcommand)| {
                args.subcommand((subcommand.args)(clap::SubCommand::with_name(*name)))
            },
        )
        .get_matches();

    if let (subcommand, Some(subcommand_args)) = matches.subcommand() {
        if let Some(subcommand) = SUBCOMMANDS.get(subcommand) {
            process::exit(match (subcommand.run)(subcommand_args) {
                Ok(()) => 0,
                Err(err) => {
                    eprintln!("error: {:?}", err);
                    1
                }
            });
        }
    }
}
