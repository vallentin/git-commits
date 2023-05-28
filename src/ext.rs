pub mod prelude {
    pub use super::{CommitExt, Commits, DiffExt, RepositoryExt};
}

use std::ops::ControlFlow;

use git2::{
    Commit, Diff, DiffDelta, DiffFormat, DiffHunk, DiffLine, DiffOptions, ErrorCode, Repository,
    Revwalk, Sort, Tree,
};

use crate::GitError;

pub trait WalkOutput {
    /// Returns `Ok(true)` to signal that the iteration should stop
    /// without any error occurring.
    fn finished(self) -> Result<bool, GitError>;
}

impl WalkOutput for () {
    fn finished(self) -> Result<bool, GitError> {
        Ok(false)
    }
}

impl WalkOutput for bool {
    fn finished(self) -> Result<bool, GitError> {
        Ok(self)
    }
}

impl WalkOutput for ControlFlow<()> {
    fn finished(self) -> Result<bool, GitError> {
        match self {
            ControlFlow::Continue(()) => Ok(false),
            ControlFlow::Break(()) => Ok(true),
        }
    }
}

impl<T> WalkOutput for Result<T, GitError>
where
    T: WalkOutput,
{
    fn finished(self) -> Result<bool, GitError> {
        self?.finished()
    }
}

pub trait RepositoryExt {
    fn commits(&self) -> Result<Commits<'_>, GitError>;
    fn count_commits(&self) -> Result<usize, GitError>;

    fn walk_commits<T, F>(&self, mut f: F) -> Result<(), GitError>
    where
        F: FnMut(Commit<'_>) -> T,
        T: WalkOutput,
    {
        for commit in self.commits()? {
            if f(commit?).finished()? {
                break;
            }
        }
        Ok(())
    }
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

pub trait CommitExt {
    fn walk_diffs<T, F>(&self, repo: &Repository, f: F) -> Result<(), GitError>
    where
        F: FnMut(Diff<'_>) -> T,
        T: WalkOutput;

    fn walk_changes<T, F>(
        &self,
        repo: &Repository,
        format: DiffFormat,
        mut f: F,
    ) -> Result<(), GitError>
    where
        F: FnMut(DiffDelta<'_>, Option<DiffHunk<'_>>, DiffLine<'_>) -> T,
        T: WalkOutput,
    {
        self.walk_diffs(repo, |diff| diff.walk_changes(format, &mut f))
    }
}

impl CommitExt for Commit<'_> {
    fn walk_diffs<T, F>(&self, repo: &Repository, mut f: F) -> Result<(), GitError>
    where
        F: FnMut(Diff<'_>) -> T,
        T: WalkOutput,
    {
        let new_tree = self.tree()?;
        if self.parent_count() == 0 {
            walk_diff(repo, None, Some(&new_tree), f)?;
        } else {
            for parent in self.parents() {
                let old_tree = parent.tree()?;
                walk_diff(repo, Some(&old_tree), Some(&new_tree), &mut f)?;
            }
        }
        Ok(())
    }
}

fn walk_diff<T, F>(
    repo: &Repository,
    old_tree: Option<&Tree<'_>>,
    new_tree: Option<&Tree<'_>>,
    f: F,
) -> Result<(), GitError>
where
    F: FnOnce(Diff<'_>) -> T,
    T: WalkOutput,
{
    let mut opts = DiffOptions::new();
    opts.show_binary(true);

    let mut diff = repo.diff_tree_to_tree(old_tree, new_tree, Some(&mut opts))?;
    diff.find_similar(None)?;

    f(diff).finished()?;

    Ok(())
}

pub trait DiffExt {
    fn walk_changes<T, F>(&self, format: DiffFormat, f: F) -> Result<(), GitError>
    where
        F: FnMut(DiffDelta<'_>, Option<DiffHunk<'_>>, DiffLine<'_>) -> T,
        T: WalkOutput;
}

impl DiffExt for Diff<'_> {
    fn walk_changes<T, F>(&self, format: DiffFormat, mut f: F) -> Result<(), GitError>
    where
        F: FnMut(DiffDelta<'_>, Option<DiffHunk<'_>>, DiffLine<'_>) -> T,
        T: WalkOutput,
    {
        let mut error = None;
        let res = self.print(format, |delta, hunk, line| {
            match f(delta, hunk, line).finished() {
                Ok(stop) => !stop,
                Err(err) => {
                    debug_assert!(error.is_none());
                    error = Some(err);
                    false
                }
            }
        });
        match res {
            Ok(()) => Ok(()),
            Err(err) if err.code() == ErrorCode::User => match error {
                Some(err) => Err(err),
                None => Ok(()),
            },
            Err(err) => {
                debug_assert!(error.is_none());
                Err(err)
            }
        }
    }
}
