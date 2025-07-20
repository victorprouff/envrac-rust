use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize)]
pub struct GithubRequest {
    pub message: String,
    pub committer: Committer,
    pub author: Author,
    pub content: String,
    pub branch: String
}

#[derive(Debug, Serialize)]
pub struct Author {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct Committer {
    pub name: String,
    pub email: String,
}
impl fmt::Display for GithubRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GithubRequest {{ message: {}, committer: {}, content: {} }}",
               self.message, self.committer, self.content)
    }
}

impl fmt::Display for Committer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Committer {{ name: {}, email: {} }}",
               self.name, self.email)
    }
}

impl fmt::Display for Author {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Committer {{ name: {}, email: {} }}",
               self.name, self.email)
    }
}