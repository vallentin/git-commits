pub mod prelude {
    pub use super::{Commits, RepositoryExt};
}

use git2::{Repository, Revwalk, Sort};

use crate::GitError;

pub trait RepositoryExt {
    fn commits(&self) -> Result<Commits<'_>, GitError>;
    fn count_commits(&self) -> Result<usize, GitError>;
}

impl RepositoryExt for Repository {
    fn commits(&self) -> Result<Commits<'_>, GitError> {
        Commits::new(self)
    }

    fn count_commits(&self) -> Result<usize, GitError> {
        Ok(revwalk(self)?.count())
    }
}

pub struct Commits<'a> {
    repo: &'a Repository,
    revwalk: Revwalk<'a>,
}

impl<'a> Commits<'a> {
    fn new(repo: &'a Repository) -> Result<Self, GitError> {
        let revwalk = revwalk(repo)?;
        Ok(Self { repo, revwalk })
    }
}

impl<'a> Iterator for Commits<'a> {
    type Item = Result<git2::Commit<'a>, GitError>;

    fn next(&mut self) -> Option<Self::Item> {
        let oid = match self.revwalk.next()? {
            Ok(oid) => oid,
            Err(err) => return Some(Err(err)),
        };
        let commit = match self.repo.find_commit(oid) {
            Ok(commit) => commit,
            Err(err) => return Some(Err(err)),
        };
        Some(Ok(commit))
    }
}

fn revwalk(repo: &Repository) -> Result<Revwalk<'_>, GitError> {
    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(Sort::REVERSE | Sort::TIME)?;
    revwalk.push_head()?;
    Ok(revwalk)
}
