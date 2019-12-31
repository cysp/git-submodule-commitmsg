extern crate git2;

use std::io::Write;
use std::path::Path;

fn main() {
    let mut args = std::env::args();
    let argv0 = args.next().unwrap_or(String::new());
    let filenames = args.collect::<Vec<String>>();

    let p = Path::new(".");
    let r: git2::Repository = match git2::Repository::discover(&p) {
        Ok(r) => r,
        Err(e) => {
            let _ = writeln!(
                &mut std::io::stderr(),
                "{}: no repository found: {}",
                argv0,
                e
            );
            std::process::exit(1);
        }
    };

    let submodules = match r.submodules() {
        Ok(submodules) => submodules,
        Err(e) => {
            let _ = writeln!(
                &mut std::io::stderr(),
                "{}: failed to enumerate submodules: {}",
                argv0,
                e
            );
            std::process::exit(1);
        }
    };

    let submodule_updates: Vec<SubmoduleUpdate> = submodules
        .iter()
        .filter_map(|submodule| {
            let path = submodule.path();
            let name = path.to_str().or(submodule.name()).unwrap_or("").to_owned();
            if filenames.len() > 0 && !filenames.contains(&name) {
                return None;
            }
            SubmoduleUpdate::from_submodule(submodule)
        })
        .collect();

    if submodule_updates.is_empty() {
        return;
    }

    let mut title = String::new();
    for submodule in submodule_updates.iter() {
        let title_component = submodule.get_title();
        if !title.is_empty() {
            title.push_str(", ");
        }
        title.push_str(&**title_component);
    }

    println!("Update {}", title);

    let multiple_updates = submodule_updates.len() > 1;

    for submodule in submodule_updates.iter() {
        match submodule.get_message() {
            Some(message) => {
                println!("");
                if multiple_updates {
                    println!("{}:", submodule.get_name());
                }
                println!("{}", message)
            }
            None => (),
        }
    }
}

#[derive(Debug)]
struct SubmoduleUpdate {
    name: std::string::String,
    title: std::string::String,
    message: Option<std::string::String>,
}

impl<'a> SubmoduleUpdate {
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

        fn short_id_for_commit_in_repo(repo: &git2::Repository, oid: git2::Oid) -> Option<String> {
            repo.find_object(oid, Some(git2::ObjectType::Commit))
                .and_then(|commit| commit.short_id())
                .ok()
                .and_then(|commit_id| commit_id.as_str().map(|id| id.to_owned()))
        }

        let id_from_str = short_id_for_commit_in_repo(&submodule_repo, current_id)
            .unwrap_or("???????".to_owned());
        let id_to_str =
            short_id_for_commit_in_repo(&submodule_repo, new_id).unwrap_or("???????".to_owned());

        let mut title_change_separator = "..";
        let mut message: Option<String> = None;
        let mut have_dropped_revs: bool = false;

        if let Ok(r) = git2::Repository::open(&path) {
            let mut message_lines = std::vec::Vec::<String>::new();

            let mut walk = match r.revwalk() {
                Ok(rw) => rw,
                Err(_) => return None,
            };
            walk.set_sorting(git2::Sort::TOPOLOGICAL);

            let _ = walk.hide(current_id);
            let _ = walk.push(new_id);
            for oid in walk {
                let oid = match oid {
                    Ok(o) => o,
                    Err(_) => continue,
                };
                let mut m = String::new();
                m.push('+');
                m.push_str(&format!("{}", oid)[0..7]);
                match r.find_commit(oid) {
                    Ok(c) => match c.message() {
                        Some(cm) => match cm.split('\n').nth(0) {
                            Some(ct) => {
                                m.push(' ');
                                m.push_str(ct);
                            }
                            None => (),
                        },
                        None => (),
                    },
                    Err(_) => (),
                }
                message_lines.push(m);
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
                    Ok(o) => o,
                    Err(_) => continue,
                };
                let mut m = String::new();
                have_dropped_revs = true;
                m.push('-');
                m.push_str(&format!("{}", oid)[0..7]);
                match r.find_commit(oid) {
                    Ok(c) => match c.message() {
                        Some(cm) => match cm.split('\n').nth(0) {
                            Some(ct) => {
                                m.push(' ');
                                m.push_str(ct);
                            }
                            None => (),
                        },
                        None => (),
                    },
                    Err(_) => (),
                }
                message_lines.push(m);
            }

            if have_dropped_revs {
                title_change_separator = "...";
            }

            message = Some(message_lines.join("\n"));
        }

        let title = format!(
            "{} ({}{}{})",
            name.clone(),
            id_from_str,
            title_change_separator,
            id_to_str
        );

        Some(SubmoduleUpdate {
            name: name,
            title: title,
            message: message,
        })
    }

    pub fn get_name(&self) -> &std::string::String {
        &self.name
    }

    pub fn get_title(&self) -> &std::string::String {
        &self.title
    }

    pub fn get_message(&self) -> Option<std::string::String> {
        self.message.clone()
    }
}
