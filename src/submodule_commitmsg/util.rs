pub fn short_id_for_commit_in_repo(
    repo: &git2::Repository,
    oid: git2::Oid,
) -> Result<String, git2::Error> {
    match repo
        .find_object(oid, Some(git2::ObjectType::Commit))?
        .short_id()?
        .as_str()
    {
        Ok(str) => Ok(str.to_owned()),
        Err(_) => Err(git2::Error::from_str("")),
    }
}
