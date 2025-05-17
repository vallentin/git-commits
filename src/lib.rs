#![forbid(unsafe_code, elided_lifetimes_in_paths)]

mod change;
mod changes;
mod commit;

pub use git2::Error as GitError;

pub use crate::change::{Added, Change, ChangeKind, Deleted, Modified, Renamed};
pub use crate::changes::Changes;
pub use crate::commit::{Commit, Signature};

use std::iter::FusedIterator;
use std::path::Path;

use git2::{Repository, Revwalk, Sort};

#[inline]
pub fn open(path: impl AsRef<Path>) -> Result<Repo, GitError> {
    Repo::open(path)
}

pub struct Repo(Repository);

impl Repo {
    #[inline]
    pub fn open(path: impl AsRef<Path>) -> Result<Self, GitError> {
        let repo = Repository::open(path)?;
        Ok(Self(repo))
    }

    #[inline]
    pub fn commits(&self) -> Result<Commits<'_>, GitError> {
        Commits::new(&self.0)
    }
}

pub struct Commits<'repo> {
    repo: &'repo Repository,
    revwalk: Revwalk<'repo>,
}

impl<'repo> Commits<'repo> {
    fn new(repo: &'repo Repository) -> Result<Self, GitError> {
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(Sort::TIME | Sort::REVERSE)?;

        Ok(Self { repo, revwalk })
    }
}

impl<'repo> Iterator for Commits<'repo> {
    type Item = Result<Commit<'repo>, GitError>;

    fn next(&mut self) -> Option<Self::Item> {
        let oid = self.revwalk.next()?;
        let oid = match oid {
            Ok(oid) => oid,
            Err(err) => return Some(Err(err)),
        };

        let commit = match self.repo.find_commit(oid) {
            Ok(commit) => commit,
            Err(err) => return Some(Err(err)),
        };

        let commit = Commit::new(self.repo, commit);

        Some(Ok(commit))
    }
}

impl FusedIterator for Commits<'_> {}
