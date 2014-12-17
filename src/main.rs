#![feature(slicing_syntax)]

extern crate collections;
extern crate serialize;
extern crate git2;

use serialize::hex::{ToHex};


fn main() {
    let mut args = std::os::args();
    let argv0 = args.remove(0);
    let filenames = args;

    let p = Path::new(".");
    let r: git2::Repository = match git2::Repository::discover(&p) {
        Ok(r) => r,
        Err(e) => {
            std::os::set_exit_status(1);
            let _ = writeln!(&mut std::io::stderr(), "{}: no repository found: {}", argv0, e);
            return;
        },
    };

    let submodules = match r.submodules() {
        Ok(submodules) => submodules,
        Err(e) => {
            std::os::set_exit_status(1);
            let _ = writeln!(&mut std::io::stderr(), "{}: failed to enumerate submodules: {}", argv0, e);
            return;
        }
    };

    let submodule_updates: Vec<SubmoduleUpdate> = submodules.iter().filter_map(|submodule| {
        let path = submodule.path();
        let name = String::from_str(path.as_str().or(submodule.name()).unwrap_or(""));
        if filenames.len() > 0 && !filenames.contains(&name) {
            return None;
        }
        SubmoduleUpdate::from_submodule(submodule)
    }).collect();

    let mut title = String::new();
    for submodule in submodule_updates.iter() {
        let title_component = submodule.get_title();
        if title.len() > 0 {
            title.push_str(", ");
        }
        title.push_str(&**title_component);
    }

    if submodule_updates.len() == 0 {
        return;
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
            },
            None => (),
        }
    }
}


#[deriving(Show)]
struct SubmoduleUpdate<'a> {
    name: collections::string::String,
    title: collections::string::String,
    message: Option<collections::string::String>,
}

impl<'a> SubmoduleUpdate<'a> {
    pub fn from_submodule(submodule: &'a git2::Submodule) -> Option<SubmoduleUpdate<'a>> {
        let path = submodule.path();
        let name = String::from_str(path.as_str().or(submodule.name()).unwrap_or("???"));

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

        let id_from_str = match submodule.head_id() {
            Some(id) => id.as_bytes()[0..4].to_hex(),
            None => String::from_str("????????"),
        };
        let id_to_str = match submodule.workdir_id() {
            Some(id) => id.as_bytes()[0..4].to_hex(),
            None => String::from_str("????????"),
        };

        let mut title_change_separator = "..";
        let mut message: Option<String> = None;
        let mut have_dropped_revs: bool = false;

        if let Ok(r) = git2::Repository::open(&path) {
            let mut message_lines = std::vec::Vec::<String>::new();

            let mut walk = match r.revwalk() {
                Ok(rw) => rw,
                Err(_) => return None,
            };
            walk.set_sorting(git2::SORT_TOPOLOGICAL);

            let _ = walk.hide(&current_id);
            let _ = walk.push(&new_id);
            for oid in walk {
                let mut m = String::new();
                m.push('+');
                m.push_str(&*oid.as_bytes()[0..4].to_hex());
                match r.find_commit(oid) {
                    Ok(c) => match c.message() {
                        Some(cm) => match cm.split('\n').nth(0) {
                            Some(ct) => {
                                m.push(' ');
                                m.push_str(ct);
                            },
                            None => (),
                        },
                        None => (),
                    },
                    Err(_) => (),
                }
                message_lines.push(m);
            }

            // libgit2 bug requires setting sort again
            walk.set_sorting(git2::SORT_TOPOLOGICAL);
            let _ = walk.hide(&new_id);
            let _ = walk.push(&current_id);
            for oid in walk {
                let mut m = String::new();
                have_dropped_revs = true;
                m.push('-');
                m.push_str(&*oid.as_bytes()[0..4].to_hex());
                match r.find_commit(oid) {
                    Ok(c) => match c.message() {
                        Some(cm) => match cm.split('\n').nth(0) {
                            Some(ct) => {
                                m.push(' ');
                                m.push_str(ct);
                            },
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

            message = Some(message_lines.connect("\n"));
        }

        let title = format!("{} ({}{}{})", name.clone(), id_from_str, title_change_separator, id_to_str);

        Some(SubmoduleUpdate{ name: name, title: title, message: message })
    }

    pub fn get_name(&self) -> &collections::string::String {
        &self.name
    }

    pub fn get_title(&self) -> &collections::string::String {
        &self.title
    }

    pub fn get_message(&self) -> Option<collections::string::String> {
        self.message.clone()
    }
}
