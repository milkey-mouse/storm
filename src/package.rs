use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub struct Package<'a> {
    repo: Option<&'a str>,
    name: &'a str,
}

impl<'a> Package<'a> {
    pub fn new(name: &'a str) -> Self {
        Self { repo: None, name }
    }

    pub fn with_repo(repo: &'a str, name: &'a str) -> Self {
        Self {
            repo: Some(repo),
            name,
        }
    }

    pub fn parse(s: &'a str) -> Self {
        match s.find(":") {
            Some(idx) if s[..idx] == *"_" => Self {
                repo: None,
                name: &s[idx + 1..],
            },
            Some(idx) => Self {
                repo: Some(&s[..idx]),
                name: &s[idx + 1..],
            },
            None => Self {
                repo: None,
                name: s,
            },
        }
    }
}

impl<'a> Display for Package<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if let Some(repo) = self.repo {
            write!(f, "{}:{}", repo, self.name)
        } else if self.name.contains(":") {
            write!(f, "<no repo>:{}", self.name)
        } else {
            write!(f, "{}", self.name)
        }
    }
}
