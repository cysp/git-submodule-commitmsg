use super::util::short_id_for_commit_in_repo;
use super::SubmoduleCommit;

#[derive(Debug)]
pub struct SubmoduleUpdate {
    pub name: std::string::String,
    pub title: std::string::String,
    pub message: Option<std::string::String>,
}

impl<'a> SubmoduleUpdate {
    pub fn new(
        name: &str,
        from_id: &str,
        to_id: &str,
        added_commits: Vec<SubmoduleCommit>,
        dropped_commits: Vec<SubmoduleCommit>,
    ) -> SubmoduleUpdate {
        let title_operator = if dropped_commits.is_empty() {
            ".."
        } else {
            "..."
        };

        let title = format!("{} ({}{}{})", name, from_id, title_operator, to_id);

        let mut message_lines = std::vec::Vec::<String>::new();
        for commit in added_commits {
            let mut message_line = format!("+{}", commit.id);
            if let Some(commit_title) = commit.title {
                message_line.push_str(&format!(" {}", commit_title));
            }
            message_lines.push(message_line);
        }
        for commit in dropped_commits {
            let mut message_line = format!("-{}", commit.id);
            if let Some(commit_title) = commit.title {
                message_line.push_str(&format!(" {}", commit_title));
            }
            message_lines.push(message_line);
        }

        SubmoduleUpdate {
            name: name.to_owned(),
            title: title,
            message: if message_lines.is_empty() {
                None
            } else {
                Some(message_lines.join("\n"))
            },
        }
    }

    pub fn from_submodule(submodule: &'a git2::Submodule) -> Option<SubmoduleUpdate> {
        let path = submodule.path();
        let name = path
            .to_str()
            .or(submodule.name())
            .unwrap_or("???")
            .to_owned();

        let submodule_repo = match submodule.open() {
            Ok(repo) => repo,
            Err(_) => return None,
        };

        let current_id = match submodule.head_id() {
            Some(id) => id,
            None => return None,
        };
        let new_id = match submodule.workdir_id() {
            Some(id) => id,
            None => return None,
        };

        if current_id == new_id {
            return None;
        }

        let id_from_str = short_id_for_commit_in_repo(&submodule_repo, current_id)
            .unwrap_or("???????".to_owned());
        let id_to_str =
            short_id_for_commit_in_repo(&submodule_repo, new_id).unwrap_or("???????".to_owned());

        let mut added_commits: Vec<SubmoduleCommit> = vec![];
        let mut dropped_commits: Vec<SubmoduleCommit> = vec![];

        if let Ok(r) = git2::Repository::open(&path) {
            let mut walk = match r.revwalk() {
                Ok(rw) => rw,
                Err(_) => return None,
            };
            walk.set_sorting(git2::Sort::TOPOLOGICAL);

            let _ = walk.hide(current_id);
            let _ = walk.push(new_id);
            for oid in walk {
                let oid = match oid {
                    Ok(oid) => oid,
                    Err(_) => continue,
                };

                if let Ok(commit) = SubmoduleCommit::from_repository_oid(&r, oid) {
                    added_commits.push(commit);
                }
            }

            let mut walk = match r.revwalk() {
                Ok(rw) => rw,
                Err(_) => return None,
            };
            walk.set_sorting(git2::Sort::TOPOLOGICAL);

            let _ = walk.hide(new_id);
            let _ = walk.push(current_id);
            for oid in walk {
                let oid = match oid {
                    Ok(oid) => oid,
                    Err(_) => continue,
                };

                if let Ok(commit) = SubmoduleCommit::from_repository_oid(&r, oid) {
                    dropped_commits.push(commit);
                }
            }
        }

        Some(Self::new(
            &name,
            &id_from_str,
            &id_to_str,
            added_commits,
            dropped_commits,
        ))
    }
}

#[test]
fn test_degenerate() {
    let update = SubmoduleUpdate::new("name", "from", "to", vec![], vec![]);

    assert_eq!(update.name, "name");
    assert_eq!(update.title, "name (from..to)");
    assert_eq!(update.message, None);
}

#[test]
fn test_adding_one_commit() {
    let update = SubmoduleUpdate::new(
        "name",
        "from",
        "to",
        vec![SubmoduleCommit::new("0000000", Some("commit".to_owned()))],
        vec![],
    );

    assert_eq!(update.name, "name");
    assert_eq!(update.title, "name (from..to)");
    assert_eq!(update.message, Some("+0000000 commit".to_owned()));
}

#[test]
fn test_adding_one_commit_without_a_title() {
    let update = SubmoduleUpdate::new(
        "name",
        "from",
        "to",
        vec![SubmoduleCommit::new("0000000", None as Option<&str>)],
        vec![],
    );

    assert_eq!(update.name, "name");
    assert_eq!(update.title, "name (from..to)");
    assert_eq!(update.message, Some("+0000000".to_owned()));
}

#[test]
fn test_dropping_one_commit() {
    let update = SubmoduleUpdate::new(
        "name",
        "from",
        "to",
        vec![],
        vec![SubmoduleCommit::new("0000000", Some("commit".to_owned()))],
    );

    assert_eq!(update.name, "name");
    assert_eq!(update.title, "name (from...to)");
    assert_eq!(update.message, Some("-0000000 commit".to_owned()));
}

#[test]
fn test_adding_and_dropping_one_commit() {
    let update = SubmoduleUpdate::new(
        "name",
        "from",
        "to",
        vec![SubmoduleCommit::new("0000000", Some("commit".to_owned()))],
        vec![SubmoduleCommit::new("0000000", Some("commit".to_owned()))],
    );

    assert_eq!(update.name, "name");
    assert_eq!(update.title, "name (from...to)");
    assert_eq!(
        update.message,
        Some("+0000000 commit\n-0000000 commit".to_owned())
    );
}
