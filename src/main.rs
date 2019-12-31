extern crate git2;
mod submodule_commitmsg;

use std::io::Write;
use std::path::Path;
use submodule_commitmsg::SubmoduleUpdate;

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
        let title_component = &submodule.title;
        if !title.is_empty() {
            title.push_str(", ");
        }
        title.push_str(&title_component);
    }

    println!("Update {}", title);

    let multiple_updates = submodule_updates.len() > 1;

    for submodule in submodule_updates.iter() {
        match &submodule.message {
            Some(message) => {
                println!("");
                if multiple_updates {
                    println!("{}:", submodule.name);
                }
                println!("{}", message)
            }
            None => (),
        }
    }
}
