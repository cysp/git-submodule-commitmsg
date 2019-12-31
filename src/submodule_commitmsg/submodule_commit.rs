use super::util::short_id_for_commit_in_repo;

#[derive(Debug)]
pub struct SubmoduleCommit {
    pub id: std::string::String,
    pub title: std::string::String,
}

impl SubmoduleCommit {
    pub fn new(id: &str, title: &str) -> SubmoduleCommit {
        SubmoduleCommit {
            id: id.to_owned(),
            title: title.to_owned(),
        }
    }
    pub fn from_repository_oid(
        r: &git2::Repository,
        oid: git2::Oid,
    ) -> Result<SubmoduleCommit, git2::Error> {
        let id = short_id_for_commit_in_repo(r, oid)?;

        let title = match r.find_commit(oid)?.message() {
            Some(cm) => match cm.split('\n').nth(0) {
                Some(ct) => ct,
                None => "",
            }
            .to_owned(),
            None => return Err(git2::Error::from_str("")),
        };

        Ok(SubmoduleCommit::new(&id, &title))
    }
}
