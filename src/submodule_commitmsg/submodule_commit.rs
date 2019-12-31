use super::util::short_id_for_commit_in_repo;

#[derive(Debug)]
pub struct SubmoduleCommit {
    pub id: std::string::String,
    pub title: Option<std::string::String>,
}

impl SubmoduleCommit {
    pub fn new<TId: Into<String>, TTitle: Into<String>>(
        id: TId,
        title: Option<TTitle>,
    ) -> SubmoduleCommit {
        SubmoduleCommit {
            id: id.into(),
            title: title.map(|title| title.into()),
        }
    }

    pub fn from_repository_oid(
        r: &git2::Repository,
        oid: git2::Oid,
    ) -> Result<SubmoduleCommit, git2::Error> {
        let id = short_id_for_commit_in_repo(r, oid)?;
        let title = r
            .find_commit(oid)
            .ok()
            .and_then({ |c| c.message().map(|cm| cm.to_owned()) })
            .and_then({ |cm| cm.split('\n').nth(0).map(|ct| ct.to_owned()) });

        Ok(SubmoduleCommit::new(&id, title))
    }
}
