use clap::{
    app_from_crate, crate_authors, crate_description, crate_name, crate_version, App, AppSettings,
    Arg, ArgMatches,
};
use nix::unistd::geteuid;
use phf::phf_map;
use std::{
    borrow::Borrow,
    env,
    error::Error,
    ffi::{OsStr, OsString},
    fs,
    path::Path,
    process,
};

mod build;
mod config;
mod install;
mod list;
mod package;
mod repo;
mod sandbox;
mod uninstall;

pub type SubCommandArgs = for<'a, 'b> fn(App<'a, 'b>) -> App<'a, 'b>;
pub type SubCommandFn<T> = fn(&ArgMatches) -> Result<T, Box<dyn Error>>;

pub struct SubCommand<T> {
    pub args: SubCommandArgs,
    pub run: SubCommandFn<T>,
}

impl<T> Borrow<SubCommandFn<T>> for &SubCommand<T> {
    fn borrow(&self) -> &SubCommandFn<T> {
        &self.run
    }
}

static SUBCOMMANDS: phf::Map<&'static str, &'static SubCommand<()>> = phf_map! {
    "build" => &build::CMD,
    "config" => &config::CMD,
    "install" => &install::CMD,
    "list" => &list::CMD,
    "repo" => &repo::CMD,
    "uninstall" => &uninstall::CMD,
};

fn main() {
    let matches = SUBCOMMANDS
        .entries()
        .fold(
            app_from_crate!()
                .arg({
                    let arg = Arg::with_name("pkgstore")
                        .help("Path to package store")
                        .env_os(OsStr::new("STORMPATH"))
                        .long("pkgstore")
                        .short("s")
                        .required(true)
                        .takes_value(true)
                        .validator_os(|x| {
                            if x.is_empty() {
                                Err(OsString::from("Store path is undefined/empty"))
                            } else {
                                let p = Path::new(x);
                                if p.is_dir() {
                                    Ok(())
                                } else if p.exists() {
                                    let mut err_string = OsString::from("Store path '");
                                    err_string.push(p);
                                    err_string.push("' exists and is not a directory");
                                    Err(err_string)
                                } else if p.parent().map(Path::is_dir).unwrap_or(false) {
                                    match fs::create_dir(p) {
                                        Ok(()) => Ok(()),
                                        Err(_) => {
                                            let mut err_string =
                                                OsString::from("Couldn't create store directory '");
                                            err_string.push(p);
                                            err_string.push("'");
                                            Err(err_string)
                                        }
                                    }
                                } else {
                                    let mut err_string =
                                        OsString::from("Parent directory of store path '");
                                    err_string.push(p);
                                    err_string.push("' doesn't exist");
                                    Err(err_string)
                                }
                            }
                        });

                    if geteuid().is_root() {
                        // TODO: is /var/lib/storm a good default?
                        arg.default_value_os(OsStr::new("/var/lib/storm"))
                    } else if let Some(store_dir) = &*config::DEFAULT_STORE_DIR {
                        arg.default_value_os(store_dir.as_os_str())
                    } else {
                        arg
                    }
                })
                .settings(&[
                    AppSettings::ArgRequiredElseHelp,
                    AppSettings::SubcommandRequired,
                ])
                .global_settings(&[
                    AppSettings::InferSubcommands,
                    AppSettings::VersionlessSubcommands,
                ]),
            |args, (name, subcommand)| {
                args.subcommand((subcommand.args)(clap::SubCommand::with_name(*name)))
            },
        )
        .get_matches();

    process::exit(match run_subcommand(&SUBCOMMANDS, &matches) {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("error: {}", err);
            1
        }
    });
}

pub fn run_subcommand<R, T: Borrow<fn(&ArgMatches) -> Result<R, Box<dyn Error>>>>(
    subcommands: &phf::Map<&'static str, T>,
    args: &ArgMatches,
) -> Result<R, Box<dyn Error>> {
    if let (subcommand, Some(subcommand_args)) = args.subcommand() {
        subcommands
            .get(subcommand)
            .expect("undefined subcommand")
            .borrow()(subcommand_args)
    } else {
        panic!("no subcommand");
    }
}
