use super::repo::RepoConfig;
use super::sandbox::SandboxConfig;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use lazy_static::lazy_static;
use phf::phf_map;
use quick_error::quick_error;
use serde::{Deserialize, Serialize};
use std::{
    env,
    error::Error,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};
use toml::value::{Table, Value};

quick_error! {
    #[derive(Debug)]
    pub enum ConfigError {
        NoConfigFile {
            description("No config file was defined; specify --file or at least set $HOME")
        }
        NoSuchKey {
            description("No such option exists in the configuration file")
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub cli: CliConfig,
    pub sandbox: SandboxConfig,
    pub repo: RepoConfig,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CliConfig {
    pub prompt: bool,
}

impl Config {
    fn get_path<'a, P: AsRef<Path>>(path: &'a Option<P>) -> Result<&'a Path, Box<dyn Error>> {
        path.as_ref()
            .map(|p| p.as_ref())
            .or_else(|| DEFAULT_CONFIG_FILE.as_ref().map(|p| p.as_path()))
            .ok_or_else(|| -> Box<dyn Error> { Box::new(ConfigError::NoConfigFile) })
    }

    pub(self) fn load_raw<P: AsRef<Path>>(path: Option<P>) -> Result<toml::Value, Box<dyn Error>> {
        match fs::read_to_string(Self::get_path(&path)?) {
            Ok(s) => Ok(toml::from_str(&s)?),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                Ok(Value::try_from(Self::default()).unwrap())
            }
            Err(e) => Err(Box::new(e)),
        }
    }

    pub fn load() -> Result<Config, Box<dyn Error>> {
        Ok(Self::load_raw::<PathBuf>(None)?.try_into()?)
    }

    pub(self) fn save_raw<P: AsRef<Path>, T: Serialize + ?Sized>(
        path: Option<P>,
        config: &T,
    ) -> Result<(), Box<dyn Error>> {
        let mut config_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(Self::get_path(&path)?)?;
        config_file.write(&toml::to_string_pretty(&config)?.into_bytes())?;
        Ok(())
    }

    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        Self::save_raw::<PathBuf, _>(None, self.into())
    }
}

lazy_static! {
    pub static ref DEFAULT_STORE_DIR: Option<PathBuf> = env::var_os("HOME")
        .map(|h| { Path::new(&h).join([".local", "share", "storm"].iter().collect::<PathBuf>()) });
    static ref DEFAULT_CONFIG_FILE: Option<PathBuf> = DEFAULT_STORE_DIR
        .as_ref()
        .map(|s| s.as_path().join("config"));
}

// NOTE: root is only potentially mutated if create is set to true
fn find_key<'a>(
    root: &'a mut Value,
    path: &str,
    create: bool,
) -> Result<&'a mut Value, Box<dyn Error>> {
    let mut ptr = root;
    for leaf in path.split(".") {
        ptr = match ptr {
            Value::Table(tbl) => {
                if create && !tbl.contains_key(leaf) {
                    tbl.insert(leaf.to_string(), Value::Table(Table::new()));
                }
                tbl.get_mut(leaf).ok_or(ConfigError::NoSuchKey)?
            }
            Value::Array(arr) => arr
                .get_mut(leaf.parse::<usize>().or(Err(ConfigError::NoSuchKey))?)
                .ok_or(ConfigError::NoSuchKey)?,
            _ => return Err(Box::new(ConfigError::NoSuchKey)),
        };
    }
    Ok(ptr)
}

fn get(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
    match find_key(
        &mut Config::load_raw(args.value_of_os("file"))?,
        args.value_of("key").unwrap(),
        false,
    )? {
        Value::String(s) => println!("{}", s),
        Value::Integer(i) => println!("{}", i),
        Value::Float(f) => println!("{}", f),
        x => println!("{}", x.to_string().trim_end_matches("\n")),
    }

    Ok(())
}

fn set(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let mut config = Config::load_raw(args.value_of_os("file"))?;

    // the TOML parser seems to want complete key-value pairings
    let raw_value = args.value_of("value").unwrap();
    let value = format!("x={}", raw_value)
        .parse()
        .map(|p| match p {
            Value::Table(mut tbl) => tbl.get_mut("x").unwrap().clone(),
            _ => panic!(),
        })
        .or_else(|_| -> Result<_, Box<dyn Error>> {
            // fall back to treating the value as a literal string
            Ok(Value::String(raw_value.to_string()))
        })?;

    let key = find_key(&mut config, args.value_of("key").unwrap(), true)?;

    *key = value;

    Config::save_raw(args.value_of_os("file"), &config)
}

fn unset(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let mut config = Config::load_raw(args.value_of_os("file"))?;

    let key_path = args.value_of("key").unwrap();
    let (key_parent, key_name) = if let Some(idx) = key_path.rfind(".") {
        let (key_path, key_name) = key_path.split_at(idx + 1);
        // chop off the last period
        let key_path = &key_path[..key_path.len() - 1];
        (find_key(&mut config, &key_path, false)?, key_name)
    } else {
        (&mut config, key_path)
    };

    if let Value::Table(tbl) = key_parent {
        if let None = tbl.remove(key_name) {
            return Err(Box::new(ConfigError::NoSuchKey));
        }
    } else {
        return Err(Box::new(ConfigError::NoSuchKey));
    }

    Config::save_raw(args.value_of_os("file"), &config)
}

fn show(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let config = Config::load_raw(args.value_of_os("file"))?;

    if args.is_present("raw") {
        print!("{}", toml::to_string_pretty(&config)?);
    } else {
        let config = config.try_into::<Config>()?;
        print!("{}", toml::to_string_pretty(&config)?);
    }

    Ok(())
}

fn reset(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
    Config::save_raw(args.value_of_os("file"), &Config::default())
}

fn args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.about("Get and set configuration options")
        .setting(AppSettings::SubcommandRequired)
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .global(true)
                .takes_value(true)
                .value_name("CONFIG_FILE")
                .help("Config file to edit (defaults to config for active package store)"),
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("Get the current value for a configuration option")
                .arg(Arg::with_name("key").required(true).index(1)),
        )
        .subcommand(
            SubCommand::with_name("set")
                .about("Set the value for a configuration option")
                .arg(Arg::with_name("key").required(true).index(1))
                .arg(Arg::with_name("value").required(true).index(2)),
        )
        .subcommand(
            SubCommand::with_name("unset")
                .about("Remove/reset an option from the configuration")
                .arg(Arg::with_name("key").required(true).index(1)),
        )
        .subcommand(
            SubCommand::with_name("show")
                .about("Validate and show the entire configuration file")
                .arg(Arg::with_name("raw").short("r").long("raw").help(
                    "Show the entire config, even parts irrelevant to this version of storm",
                )),
        )
        .subcommand(
            SubCommand::with_name("reset")
                .about("Replace all configuration options with their default values"),
        )
}

static SUBCOMMANDS: phf::Map<&'static str, crate::SubCommandFn> = phf_map! {
    "get" => get,
    "set" => set,
    "unset" => unset,
    "show" => show,
    "reset" => reset,
};

fn run(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
    crate::run_subcommand(&SUBCOMMANDS, args)
}

pub static CMD: crate::SubCommand = crate::SubCommand { args, run };
