//! Abstraction of [`git2`] providing a simple interface
//! for easily iterating over commits and changes in a
//! Git repository.
//!
//! In short, both `git log --name-status` and
//! `git log --stat --format=fuller` can be
//! implemented with just a handful of lines.
//!
//! [`git2`]: https://crates.io/crates/git2
//!
//! # Example
//!
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let repo = git_commits::open("path-to-repo")?;
//!
//! for commit in repo.commits()? {
//!     // The `commit` contains the message, author, committer, time, etc
//!     let commit = commit?;
//!     println!("\n{}", commit);
//!
//!     for change in commit.changes()? {
//!         // The `change` contains change kind, old/new path, old/new sizes, etc
//!         let change = change?;
//!         println!("  {}", change);
//!
//!         // match change {
//!         //     Change::Added(change) => {}
//!         //     Change::Modified(change) => {}
//!         //     Change::Deleted(change) => {}
//!         //     Change::Renamed(change) => {}
//!         // }
//!     }
//! }
//! #     Ok(())
//! # }
//! ```

#![forbid(unsafe_code, elided_lifetimes_in_paths)]

mod change;
mod changes;
mod commit;

pub use git2::Error as GitError;
pub use git2::Sort;

pub use crate::change::{Added, Change, ChangeKind, Deleted, Modified, Renamed};
pub use crate::changes::Changes;
pub use crate::commit::{Commit, Signature};

use std::iter::FusedIterator;
use std::path::Path;

use git2::{Repository, Revwalk};

#[inline]
pub fn open(path: impl AsRef<Path>) -> Result<Repo, GitError> {
    Repo::open(path)
}

pub struct Repo(Repository);

impl Repo {
    /// Attempt to open an already-existing repository at `path`.
    ///
    /// The path can point to either a normal or bare repository.
    #[inline]
    pub fn open(path: impl AsRef<Path>) -> Result<Self, GitError> {
        let repo = Repository::open(path)?;
        Ok(Self(repo))
    }

    /// Attempt to open an already-existing repository at or above `path`.
    ///
    /// This starts at `path` and looks up the filesystem hierarchy
    /// until it finds a repository.
    #[inline]
    pub fn discover(path: impl AsRef<Path>) -> Result<Self, GitError> {
        let repo = Repository::discover(path)?;
        Ok(Self(repo))
    }

    /// Returns an iterator that produces all commits
    /// in the repo.
    ///
    /// _See [`.commits_ext()`](Repo::commits_ext) to be
    /// able to specify the order._
    #[inline]
    pub fn commits(&self) -> Result<Commits<'_>, GitError> {
        self.commits_ext(Sort::NONE)
    }

    /// Returns an iterator that produces all commits
    /// in the repo.
    #[inline]
    pub fn commits_ext(&self, sort: Sort) -> Result<Commits<'_>, GitError> {
        Commits::new(&self.0, sort)
    }
}

pub struct Commits<'repo> {
    repo: &'repo Repository,
    revwalk: Revwalk<'repo>,
}

impl<'repo> Commits<'repo> {
    fn new(repo: &'repo Repository, sort: Sort) -> Result<Self, GitError> {
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(sort)?;

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
