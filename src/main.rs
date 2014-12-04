extern crate serialize;
extern crate git2;

use std::fmt::{Show, Formatter};
use serialize::hex::{ToHex};


fn main() {
  let mut args = std::os::args();
  let argv0 = args.remove(0);
  // let filenames = args;

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

  let submodules: Vec<SubmoduleUpdate> = submodules.iter().filter_map(|submodule| {
    SubmoduleUpdate::from_submodule(submodule)
  }).collect();

  for submodule in submodules.iter() {
    println!("a: {}", submodule);
  }
}


struct SubmoduleUpdate<'a> {
  submodule: &'a git2::Submodule<'a>,
  title: String,
}

impl<'a> SubmoduleUpdate<'a> {
  // pub fn new(submodule: &'a git2::Submodule) -> SubmoduleUpdate<'a> {
  //   SubmoduleUpdate{ submodule: submodule }
  // }

  pub fn from_submodule(submodule: &'a git2::Submodule) -> Option<SubmoduleUpdate<'a>> {
    let current_id = match submodule.head_id() {
      Some(id) => id,
      None => return None,
    };
    let new_id = match submodule.workdir_id() {
      Some(id) => id,
      None => return None,
    };

    // if current_id == new_id {
    //   return None;
    // }

    let path = submodule.path();
    let name = path.as_str().or(submodule.name());

    let change = format!("{}..{}", current_id.as_bytes()[0..4].to_hex(), new_id.as_bytes()[0..4].to_hex());

    let title = format!("{} ({})", name, change);

    let r = match git2::Repository::open(&path) {
      Ok(r) => r,
      Err(_) => return None,
    };

    let added_walk = match r.new_revwalk() {
      Ok(rw) => rw,
      Err(_) => return None,
    };
    added_walk.set_sorting(git2::SORT_TOPOLOGICAL);
    added_walk.hide(&current_id);
    added_walk.push(&new_id);

    for oid in added_walk.oid_iter() {
      println!("+{}", oid);
    }

    let dropped_walk = match r.new_revwalk() {
      Ok(rw) => rw,
      Err(_) => return None,
    };
    dropped_walk.set_sorting(git2::SORT_TOPOLOGICAL);
    dropped_walk.hide(&new_id);
    dropped_walk.push(&current_id);

    for oid in dropped_walk.oid_iter() {
      println!("-{}", oid);
    }

    Some(SubmoduleUpdate{ submodule: submodule, title: title })
  }
}

impl<'a> std::fmt::Show for SubmoduleUpdate<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
    write!(f, "SubmoduleUpdate {{ path: {}, title: {} }}",
      self.submodule.path().as_str().unwrap(),
      self.title,
      )
  }
}
