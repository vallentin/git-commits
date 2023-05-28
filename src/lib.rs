#![forbid(unsafe_code)]
#![deny(elided_lifetimes_in_paths)]

pub mod prelude {
    pub use crate::ext::prelude::*;
}

mod ext;

pub use git2::Error as GitError;

pub use crate::prelude::*;

use git2::{Commit, DiffDelta, DiffFormat, DiffHunk, DiffLine, Repository};

use crate::ext::WalkOutput;

#[inline]
pub fn commits(repo: &Repository) -> Result<Commits<'_>, GitError> {
    repo.commits()
}

#[inline]
pub fn count_commits(repo: &Repository) -> Result<usize, GitError> {
    repo.count_commits()
}

#[inline]
pub fn walk_commits<T, F>(repo: &Repository, f: F) -> Result<(), GitError>
where
    F: FnMut(Commit<'_>) -> T,
    T: WalkOutput,
{
    repo.walk_commits(f)
}

#[inline]
pub fn walk_changes<T, F>(
    repo: &Repository,
    commit: &Commit<'_>,
    format: DiffFormat,
    f: F,
) -> Result<(), GitError>
where
    F: FnMut(DiffDelta<'_>, Option<DiffHunk<'_>>, DiffLine<'_>) -> T,
    T: WalkOutput,
{
    commit.walk_changes(&repo, format, f)
}
