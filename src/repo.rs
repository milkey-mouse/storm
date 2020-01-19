use super::config::Config;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use phf::phf_map;
use quick_error::quick_error;
use serde::{
    de::{self, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use std::{collections::HashMap, error::Error, fmt, iter, marker::PhantomData};

quick_error! {
    #[derive(Debug)]
    pub enum RepoError {
        NoSuchRepo {
            description("No repo exists with the specified name")
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoConfig {
    #[serde(deserialize_with = "string_or_seq", rename = "default")]
    default_repos: Vec<String>,

    #[serde(flatten)]
    repos: HashMap<String, Repo>,
}

impl Default for RepoConfig {
    fn default() -> Self {
        let default_repos = vec![String::from("dummy")];
        let mut repos = HashMap::new();
        repos.insert(String::from("dummy"), Repo::Dummy(Default::default()));
        Self {
            default_repos,
            repos,
        }
    }
}

mod dummy;
mod gentoo;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
enum Repo {
    Dummy(dummy::Repo),
    Gentoo(gentoo::Repo),
}

/*static SUBCOMMANDS: phf::Map<&'static str, &'static crate::SubCommand> = phf_map! {
    "dummy" => &dummy::CMD,
    "gentoo" => &gentoo::CMD,
};*/

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
            let seq = iter::once(String::from(value));
            Deserialize::deserialize(de::value::SeqDeserializer::new(seq))
        }

        fn visit_seq<S: SeqAccess<'de>>(self, seq: S) -> Result<T, S::Error> {
            Deserialize::deserialize(de::value::SeqAccessDeserializer::new(seq))
        }
    }

    deserializer.deserialize_any(StringOrSeq(PhantomData))
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
            SubCommand::with_name("add").about("Add a new repository"), // TODO: add subcommands like main does
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
            SubCommand::with_name("sync")
                .about("Sync repositories")
                .arg(Arg::with_name("repo").multiple(true).index(1)),
        )
}

fn list(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let config = Config::load()?;

    if args.is_present("default") {
        for repo in config.repo.default_repos {
            println!("{}", repo);
        }
    } else {
        let mut repos = config.repo.repos.keys().collect::<Vec<_>>();
        repos.sort_unstable();

        for repo in repos {
            /*if config.default_repos.contains(repo) && isatty(STDOUT) {
                println!("{} (default)", repo);
            } else {
                println!("{}", repo);
            }*/
            println!("{}", repo);
        }
    }

    Ok(())
}

// TODO: make all potentially error-producing functions #[must_use]
fn add(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
    // TODO: delegate to per-repo-type subcommands
    Ok(())
}

fn remove(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let mut config = Config::load()?;

    let repo = args.value_of("repo").unwrap();

    if let None = config.repo.repos.remove(repo) {
        return Err(Box::new(RepoError::NoSuchRepo))
    }
    
    config.repo.default_repos.retain(|r| r != repo); 

    config.save()
}

fn rename(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let mut config = Config::load()?;

    let old_name = args.value_of("old").unwrap();
    let new_name = args.value_of("new").unwrap();

    for repo in config.repo.default_repos.iter_mut() {
        if repo == old_name {
            *repo = String::from(new_name);
        }
    }

    let repo = config.repo.repos.remove(old_name).ok_or(RepoError::NoSuchRepo)?;
    // TODO: use unwrap_none when stable
    if !config.repo.repos.insert(String::from(new_name), repo).is_none() {
        panic!();
    }

    config.save()
}

fn sync(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
    Ok(())
}

static SUBCOMMANDS: phf::Map<&'static str, crate::SubCommandFn> = phf_map! {
    "list" => list,
    "add" => add,
    "remove" => remove,
    "rename" => rename,
    "sync" => sync,
};

fn run(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
    crate::run_subcommand(&SUBCOMMANDS, args)
}

pub static CMD: crate::SubCommand = crate::SubCommand { args, run };
