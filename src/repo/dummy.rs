use super::Repo;
use clap::{App, ArgMatches};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DummyRepo;

fn args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.about("Add a dummy repository")
}

fn run(_args: &ArgMatches) -> Result<Repo, Box<dyn Error>> {
    Ok(Repo::Dummy(DummyRepo::default()))
}

pub(super) static CMD: crate::SubCommand<Repo> = crate::SubCommand { args, run };
