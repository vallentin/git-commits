use git_commits::Change;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).unwrap_or_else(|| String::from("."));
    let repo = git_commits::open(&path)?;

    for commit in repo.commits()? {
        let commit = commit?;

        let msg = commit.message_lossy();
        let first_line = msg.trim().lines().next().unwrap_or_default();

        let committer = commit.committer();
        let author = commit.author();

        println!();
        println!("SHA:     {}", commit.sha());
        println!("Time:    {}", committer.time_local().unwrap()); // same as `commit.time_local()`
        println!("Author:  {}", author.name_lossy());
        println!("Message: {first_line}");
        println!();

        for change in commit.changes()? {
            let change = change?;

            // The following `match` can be simplified, by simply doing:
            // println!("  {}", change);

            match change {
                Change::Added(change) => {
                    println!(
                        "  {} {} ({} bytes)",
                        change.kind().letter(),
                        change.path().display(),
                        change.size(),
                    );
                }
                Change::Modified(change) => {
                    println!(
                        "  {} {} ({} -> {} bytes)",
                        change.kind().letter(),
                        change.path().display(),
                        change.old_size(),
                        change.new_size(),
                    );
                }
                Change::Deleted(change) => {
                    println!(
                        "  {} {} ({} bytes)",
                        change.kind().letter(),
                        change.path().display(),
                        change.size(),
                    );
                }
                Change::Renamed(change) => {
                    println!(
                        "  {} {} -> {} ({} bytes)",
                        change.kind().letter(),
                        change.old_path().display(),
                        change.new_path().display(),
                        change.size(),
                    );
                }
            }
        }
    }

    Ok(())
}
