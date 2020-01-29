use crate::config::Config;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use phf::phf_map;
use quick_error::quick_error;
use serde::{
    de::{self, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
#[cfg(feature = "interactive")]
use serde_diff::{simple_serde_diff, SerdeDiff};
use std::{borrow::Borrow, collections::HashMap, error::Error, fmt, iter, marker::PhantomData};

quick_error! {
    #[derive(Debug)]
    pub enum RepoError {
        NoSuchRepo {
            display("no repo exists with the specified name")
        }
    }
}

mod dummy;
mod gentoo;

// TODO: Data::Enum is not supported by serde_diff
#[cfg_attr(feature = "interactive", derive(Clone, PartialEq/*, SerdeDiff*/))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
enum Repo {
    Dummy(dummy::DummyRepo),
    Gentoo(gentoo::GentooRepo),
}

simple_serde_diff!(Repo);

static ADD_SUBCOMMANDS: phf::Map<&'static str, &'static crate::SubCommand<Repo>> = phf_map! {
    "dummy" => &dummy::CMD,
    //"gentoo" => &gentoo::CMD,
};

// This deserializer parses a single string as an array with a single string.
fn string_or_seq<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    struct StringOrSeq<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for StringOrSeq<T>
    where
        T: Deserialize<'de>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or array of strings")
        }

        fn visit_str<E: de::Error>(self, value: &str) -> Result<T, E> {
            let seq = iter::once(value.to_string());
            Deserialize::deserialize(de::value::SeqDeserializer::new(seq))
        }

        fn visit_seq<S: SeqAccess<'de>>(self, seq: S) -> Result<T, S::Error> {
            Deserialize::deserialize(de::value::SeqAccessDeserializer::new(seq))
        }
    }

    deserializer.deserialize_any(StringOrSeq(PhantomData))
}

#[cfg_attr(feature = "interactive", derive(Clone, PartialEq, SerdeDiff))]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RepoConfig {
    #[serde(deserialize_with = "string_or_seq", rename = "default")]
    default_repos: Vec<String>,

    #[serde(flatten)]
    repos: HashMap<String, Repo>,
}

impl RepoConfig {
    fn list(&self, sort: bool, only_default: bool) -> Vec<String> {
        let mut repos = if only_default {
            self.default_repos.clone()
        } else {
            self.repos.keys().cloned().collect()
        };

        if sort {
            repos.sort_unstable()
        }

        repos
    }

    fn add(&mut self, name: String, args: &ArgMatches) -> Result<(), Box<dyn Error>> {
        let repo = crate::run_subcommand(&ADD_SUBCOMMANDS, args)?;

        self.repos.insert(name, repo);

        Ok(())
    }

    fn remove<T: Borrow<str>>(&mut self, name: T) -> Result<(), Box<dyn Error>> {
        if let None = self.repos.remove(name.borrow()) {
            return Err(Box::new(RepoError::NoSuchRepo));
        }

        self.default_repos.retain(|r| r != name.borrow());

        Ok(())
    }

    fn rename<O: Borrow<str>, N: Borrow<str>>(
        &mut self,
        old_name: O,
        new_name: N,
    ) -> Result<(), Box<dyn Error>> {
        let repo = self
            .repos
            .remove(old_name.borrow())
            .ok_or(RepoError::NoSuchRepo)?;

        // TODO: use unwrap_none when stable
        if !self
            .repos
            .insert(new_name.borrow().to_string(), repo)
            .is_none()
        {
            panic!();
        }

        for repo in self.default_repos.iter_mut() {
            if repo == old_name.borrow() {
                *repo = new_name.borrow().to_string();
            }
        }

        Ok(())
    }

    fn set_default(
        &mut self,
        name: String,
        default: bool,
        first: bool,
    ) -> Result<(), Box<dyn Error>> {
        if !self.repos.contains_key(&name) {
            return Err(Box::new(RepoError::NoSuchRepo));
        }

        self.default_repos.retain(|r| r != &name);

        if default {
            if first {
                self.default_repos.insert(0, name);
            } else {
                self.default_repos.push(name);
            }
        }

        Ok(())
    }

    fn sync(&self) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }
}

fn args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.about("Manage package repositories")
        .setting(AppSettings::SubcommandRequired)
        .subcommand(
            SubCommand::with_name("list")
                .about("List repositories")
                .alias("show")
                .arg(
                    Arg::with_name("default")
                        .long("default")
                        .short("d")
                        .help("List default repositories in order of precedence"),
                ),
        )
        .subcommand(
            ADD_SUBCOMMANDS.entries().fold(
                SubCommand::with_name("add")
                    .about("Add a new repository")
                    .setting(AppSettings::SubcommandRequired)
                    .arg(
                        Arg::with_name("name")
                            .required(true)
                            .index(1)
                            .help("Name of the new repository"),
                    )
                    .arg(
                        Arg::with_name("default")
                            .long("default")
                            .short("d")
                            .help("Set the new repository as a default"),
                    )
                    .arg(
                        Arg::with_name("precedence")
                            .help("Whether new defaults should be checked first or last")
                            .long("precedence")
                            .short("p")
                            .takes_value(true)
                            .possible_values(&["first", "last"])
                            .default_value("last"),
                    ),
                |args, (name, subcommand)| {
                    args.subcommand((subcommand.args)(SubCommand::with_name(*name)))
                },
            ),
        )
        .subcommand(
            SubCommand::with_name("remove")
                .about("Remove a repository")
                .arg(
                    Arg::with_name("repo")
                        .required(true)
                        .index(1)
                        .help("Name of the repository to remove"),
                ),
        )
        .subcommand(
            SubCommand::with_name("rename")
                .about("Rename a repository")
                .arg(Arg::with_name("old").required(true).index(1))
                .arg(Arg::with_name("new").required(true).index(2)),
        )
        .subcommand(
            SubCommand::with_name("set-default")
                .about("Add or remove a repository from the default repositories")
                .arg(Arg::with_name("repo").required(true).index(1))
                .arg(
                    Arg::with_name("default")
                        .required(true)
                        .index(2)
                        .possible_values(&["true", "false"])
                        .default_value("true"),
                )
                .arg(
                    Arg::with_name("precedence")
                        .help("Whether new defaults should be checked first or last")
                        .long("precedence")
                        .short("p")
                        .takes_value(true)
                        .possible_values(&["first", "last"])
                        .default_value("last"),
                ),
        )
        .subcommand(
            SubCommand::with_name("sync")
                .about("Sync repositories")
                .arg(Arg::with_name("repo").multiple(true).index(1)),
        )
}

fn list(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let default_only = args.is_present("default");

    for repo in Config::load()?.repo.list(!default_only, default_only) {
        /*if config.default_repos.contains(repo) && isatty(STDOUT) {
            println!("{} (default)", repo);
        } else {
            println!("{}", repo);
        }*/
        println!("{}", repo);
    }

    Ok(())
}

// TODO: weird problem with clap parsing when added repo name is (or is close to) a valid subcommand
fn add(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let mut config = Config::load()?;

    config
        .repo
        .add(args.value_of("name").unwrap().to_string(), &args)?;

    config.save()
}

fn remove(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let mut config = Config::load()?;

    config.repo.remove(args.value_of("repo").unwrap())?;

    config.save()
}

fn rename(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let mut config = Config::load()?;

    let old_name = args.value_of("old").unwrap();
    let new_name = args.value_of("new").unwrap();

    config.repo.rename(old_name, new_name)?;

    config.save()
}

fn set_default(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let mut config = Config::load()?;

    let repo = args.value_of("repo").unwrap();
    let default = match args.value_of("default") {
        Some("true") => true,
        Some("false") => false,
        _ => unreachable!(),
    };

    let first = match args.value_of("precedence") {
        Some("first") => true,
        Some("last") => false,
        _ => unreachable!(),
    };

    config.repo.set_default(repo.to_string(), default, first)?;

    config.save()
}

fn sync(_args: &ArgMatches) -> Result<(), Box<dyn Error>> {
    Config::load()?.repo.sync()
}

static SUBCOMMANDS: phf::Map<&'static str, crate::SubCommandFn<()>> = phf_map! {
    "list" => list,
    "add" => add,
    "remove" => remove,
    "rename" => rename,
    "set-default" => set_default,
    "sync" => sync,
};

fn run(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
    crate::run_subcommand(&SUBCOMMANDS, args)
}

pub static CMD: crate::SubCommand<()> = crate::SubCommand { args, run };
