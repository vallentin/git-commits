use git_commits::Change;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).unwrap_or_else(|| String::from("."));
    let repo = git_commits::open(&path)?;

    for commit in repo.commits()? {
        let commit = commit?;
        let msg = commit.message().unwrap().trim();

        let author = commit.author();
        let committer = commit.committer();

        println!();
        print!("[{}] ", &commit.sha()[..7]);
        println!("{}", msg);
        println!("Author:    {}", author.name_lossy());
        println!("Committer: {}", committer.name_lossy());

        for change in commit.changes()? {
            let change = change?;

            // The following `match` can be simplified, by simply doing:
            // println!("  {}", change);

            let kind = change.kind();
            match change {
                Change::Added(change) => {
                    println!(
                        "  {} {} ({} bytes)",
                        kind.letter(),
                        change.path.display(),
                        change.size,
                    );
                }
                Change::Modified(change) => {
                    println!(
                        "  {} {} ({} -> {} bytes)",
                        kind.letter(),
                        change.path.display(),
                        change.old_size,
                        change.new_size,
                    );
                }
                Change::Deleted(change) => {
                    println!(
                        "  {} {} ({} bytes)",
                        kind.letter(),
                        change.path.display(),
                        change.size,
                    );
                }
                Change::Renamed(change) => {
                    println!(
                        "  {} {} -> {} ({} bytes)",
                        kind.letter(),
                        change.old_path.display(),
                        change.new_path.display(),
                        change.size,
                    );
                }
            }
        }
    }

    Ok(())
}
