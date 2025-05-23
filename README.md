# git-commits

[![Latest Version](https://img.shields.io/crates/v/git-commits.svg)](https://crates.io/crates/git-commits)
[![Docs](https://docs.rs/git-commits/badge.svg)](https://docs.rs/git-commits)
[![License](https://img.shields.io/github/license/vallentin/git-commits.svg)](https://github.com/vallentin/git-commits)

Abstraction of [`git2`] providing a simple interface
for easily iterating over commits and changes in a
Git repository.

In short, both `git log --name-status` and
`git log --stat --format=fuller` can be
implemented with just a handful of lines.

[`git2`]: https://crates.io/crates/git2

## Example

```rust
let repo = git_commits::open("path-to-repo")?;

for commit in repo.commits()? {
    // The `commit` contains the message, author, committer, time, etc
    let commit = commit?;
    println!("\n{}", commit);

    for change in commit.changes()? {
        // The `change` contains change kind, old/new path, old/new sizes, etc
        let change = change?;
        println!("  {}", change);

        // match change {
        //     Change::Added(change) => {}
        //     Change::Modified(change) => {}
        //     Change::Deleted(change) => {}
        //     Change::Renamed(change) => {}
        // }
    }
}
```
