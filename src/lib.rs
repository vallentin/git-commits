#![forbid(unsafe_code)]
#![deny(elided_lifetimes_in_paths)]

mod ext;

pub use git2::Error as GitError;

pub use crate::ext::prelude::*;

use git2::Repository;

#[inline]
pub fn commits(repo: &Repository) -> Result<Commits<'_>, GitError> {
    repo.commits()
}

#[inline]
pub fn count_commits(repo: &Repository) -> Result<usize, GitError> {
    repo.count_commits()
}
