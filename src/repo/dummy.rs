use super::Repo;
use clap::{App, ArgMatches};
use serde::{Deserialize, Serialize};
#[cfg(feature = "interactive")]
use serde_diff::{simple_serde_diff, SerdeDiff};
use std::error::Error;

#[cfg_attr(feature = "interactive", derive(Clone, PartialEq/*, SerdeDiff*/))]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DummyRepo;

simple_serde_diff!(DummyRepo);

fn args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.about("Add a dummy repository")
}

fn run(_args: &ArgMatches) -> Result<Repo, Box<dyn Error>> {
    Ok(Repo::Dummy(DummyRepo::default()))
}

pub(super) static CMD: crate::SubCommand<Repo> = crate::SubCommand { args, run };
