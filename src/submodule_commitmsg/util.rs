pub fn short_id_for_commit_in_repo(
    repo: &git2::Repository,
    oid: git2::Oid,
) -> Result<String, git2::Error> {
    match repo
        .find_object(oid, Some(git2::ObjectType::Commit))?
        .short_id()?
        .as_str()
    {
        Some(str) => Ok(str.to_owned()),
        None => Err(git2::Error::from_str("")),
    }
}
