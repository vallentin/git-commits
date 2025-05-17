use std::iter::FusedIterator;
use std::path::Path;

use git2::{Delta, Diff, DiffDelta, DiffFile, Repository};

use super::GitError;
use super::{Added, Change, Commit, Deleted, Modified, Renamed};

pub struct Changes<'repo, 'commit> {
    commit: &'commit Commit<'repo>,
    diff: Diff<'repo>,
    idx_delta: usize,
    next_change: Option<Change>,
}

impl<'repo, 'commit> Changes<'repo, 'commit> {
    pub(crate) fn from_commit(commit: &'commit Commit<'repo>) -> Result<Self, GitError> {
        let current_tree = commit.commit.tree()?;

        let parent_tree = commit
            .commit
            .parent(0)
            .ok()
            .map(|parent| parent.tree())
            .transpose()?;

        let mut diff =
            commit
                .repo
                .diff_tree_to_tree(parent_tree.as_ref(), Some(&current_tree), None)?;

        diff.find_similar(None)?;

        Ok(Self {
            commit,
            diff,
            idx_delta: 0,
            next_change: None,
        })
    }
}

impl<'repo, 'commit> Iterator for Changes<'repo, 'commit> {
    type Item = Result<Change, GitError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(change) = self.next_change.take() {
                return Some(Ok(change));
            }

            let delta = match self.diff.get_delta(self.idx_delta) {
                Some(delta) => delta,
                None => return None,
            };
            self.idx_delta += 1;

            match extract_changes(&self.commit.repo, delta) {
                Ok(Some((change, next_change))) => {
                    self.next_change = next_change;

                    return Some(Ok(change));
                }
                Ok(None) => {}
                Err(err) => return Some(Err(err)),
            }
        }
    }
}

impl FusedIterator for Changes<'_, '_> {}

struct ChangeFileRef<'diff> {
    path: &'diff Path,
    /// Total size in bytes.
    size: usize,
}

impl<'diff> ChangeFileRef<'diff> {
    fn new(repo: &Repository, file: DiffFile<'diff>) -> Option<Self> {
        if !file.exists() {
            return None;
        }

        let path = file.path()?;

        let oid = file.id();
        let Ok(blob) = repo.find_blob(oid) else {
            // Technically safe to unwrap as if `file` exists,
            // then `find_blob()` returns `Ok`
            return None;
        };

        Some(Self {
            path,
            size: blob.size(),
        })
    }
}

fn extract_changes<'repo>(
    repo: &Repository,
    delta: DiffDelta<'_>,
) -> Result<Option<(Change, Option<Change>)>, GitError> {
    let old_file = ChangeFileRef::new(repo, delta.old_file());
    let new_file = ChangeFileRef::new(repo, delta.new_file());

    match delta.status() {
        Delta::Added | Delta::Copied => {
            let Some(new_file) = new_file else {
                // Technically, this is an error but it would never occur
                return Ok(None);
            };

            let change = Change::Added(Added {
                path: new_file.path.to_path_buf(),
                size: new_file.size,
            });

            Ok(Some((change, None)))
        }
        Delta::Modified => {
            let Some(old_file) = old_file else {
                // Technically, this is an error but it would never occur
                return Ok(None);
            };
            let Some(new_file) = new_file else {
                // Technically, this is an error but it would never occur
                return Ok(None);
            };

            let change = Change::Modified(Modified {
                path: new_file.path.to_path_buf(),
                old_size: old_file.size,
                new_size: new_file.size,
            });

            Ok(Some((change, None)))
        }
        Delta::Deleted => {
            let Some(old_file) = old_file else {
                // Technically, this is an error but it would never occur
                return Ok(None);
            };

            let change = Change::Deleted(Deleted {
                path: old_file.path.to_path_buf(),
                size: old_file.size,
            });

            Ok(Some((change, None)))
        }
        Delta::Renamed => {
            let Some(old_file) = old_file else {
                // Technically, this is an error but it would never occur
                return Ok(None);
            };
            let Some(new_file) = new_file else {
                // Technically, this is an error but it would never occur
                return Ok(None);
            };

            let change_modified = if old_file.size != new_file.size {
                Some(Change::Modified(Modified {
                    path: new_file.path.to_path_buf(),
                    old_size: old_file.size,
                    new_size: new_file.size,
                }))
            } else {
                None
            };

            let change_renamed = Change::Renamed(Renamed {
                old_path: old_file.path.to_path_buf(),
                new_path: new_file.path.to_path_buf(),
                size: new_file.size,
            });

            let change = match change_modified {
                Some(change_modified) => (change_modified, Some(change_renamed)),
                None => (change_renamed, None),
            };

            Ok(Some(change))
        }
        Delta::Unmodified
        | Delta::Ignored
        | Delta::Untracked
        | Delta::Typechange
        | Delta::Unreadable
        | Delta::Conflicted => {
            return Ok(None);
        }
    }
}
