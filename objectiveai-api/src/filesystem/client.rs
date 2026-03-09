use std::path::{Path, PathBuf};

/// Reads files from local git repositories on the filesystem.
///
/// Repositories are expected at `{base_dir}/{owner}/{repository}/`.
/// Files can be read from the working tree or from a specific git commit.
#[derive(Debug, Clone)]
pub struct Client {
    pub base_dir: PathBuf,
    pub commit_author_name: String,
    pub commit_author_email: String,
}

impl Client {
    pub fn new(base_dir: PathBuf, commit_author_name: String, commit_author_email: String) -> Self {
        Self { base_dir, commit_author_name, commit_author_email }
    }

    /// Removes the entire base directory and all its contents.
    pub fn clear(&self) -> std::io::Result<()> {
        if self.base_dir.exists() {
            std::fs::remove_dir_all(&self.base_dir)?;
        }
        Ok(())
    }

    /// Returns the repository path for the given owner and repository.
    pub fn repo_path(&self, owner: &str, repository: &str) -> PathBuf {
        self.base_dir.join(owner).join(repository)
    }

    /// Checks whether a repository exists on the filesystem as an initialized git repository.
    pub fn repository_exists(&self, owner: &str, repository: &str) -> bool {
        let repo_path = self.repo_path(owner, repository);
        git2::Repository::open(&repo_path).is_ok()
    }

    /// Resolves the HEAD commit SHA for a repository.
    pub fn resolve_head(&self, owner: &str, repository: &str) -> Result<String, super::Error> {
        let repo_path = self.repo_path(owner, repository);
        let repo = git2::Repository::open(&repo_path)?;
        let head = repo.head()?;
        let commit = head.peel_to_commit()?;
        Ok(commit.id().to_string())
    }

    /// Reads a file's raw content from a repository.
    ///
    /// If `commit` is `Some`, reads from that specific git commit.
    /// If `commit` is `None`, reads from the working tree and resolves HEAD.
    ///
    /// Returns `Ok(None)` if the repository or file does not exist.
    /// Returns `Ok(Some((content, resolved_commit)))` on success.
    pub async fn read_file(
        &self,
        owner: &str,
        repository: &str,
        commit: Option<&str>,
        file_name: &str,
    ) -> Result<Option<(String, String)>, super::Error> {
        let repo_path = self.repo_path(owner, repository);

        match commit {
            Some(sha) => {
                match read_file_at_commit(&repo_path, file_name, sha) {
                    Ok(content) => Ok(Some((content, sha.to_string()))),
                    Err(e) if is_not_found(&e) => Ok(None),
                    Err(e) => Err(e),
                }
            }
            None => {
                let file_path = repo_path.join(file_name);
                match tokio::fs::read_to_string(&file_path).await {
                    Ok(content) => {
                        let resolved = self
                            .resolve_head(owner, repository)
                            .unwrap_or_else(|_| "HEAD".to_string());
                        Ok(Some((content, resolved)))
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
                    Err(e) => Err(e.into()),
                }
            }
        }
    }

    /// Reads and deserializes a JSON file from a repository.
    ///
    /// If `commit` is `Some`, reads from that specific git commit.
    /// If `commit` is `None`, reads from the working tree and resolves HEAD.
    ///
    /// Returns `Ok(None)` if the repository or file does not exist.
    /// Returns `Ok(Some((value, resolved_commit)))` on success.
    pub async fn read_json<T: serde::de::DeserializeOwned>(
        &self,
        owner: &str,
        repository: &str,
        commit: Option<&str>,
        file_name: &str,
    ) -> Result<Option<(T, String)>, super::Error> {
        let Some((content, resolved_commit)) =
            self.read_file(owner, repository, commit, file_name).await?
        else {
            return Ok(None);
        };

        let mut de = serde_json::Deserializer::from_str(&content);
        let value = serde_path_to_error::deserialize(&mut de)?;
        Ok(Some((value, resolved_commit)))
    }

    /// Publishes invention files to a local git repository.
    ///
    /// Handles any initial state: creates the directory if needed, initializes
    /// or resets the git repository, writes files, and commits.
    ///
    /// Returns the commit SHA on success.
    pub fn publish(
        &self,
        owner: &str,
        repository: &str,
        files: &[(&str, &str)],
        commit_message: &str,
    ) -> Result<String, super::Error> {
        let repo_path = self.repo_path(owner, repository);

        // Create directory recursively if needed.
        std::fs::create_dir_all(&repo_path)?;

        // Open or initialize the git repository.
        let repo = match git2::Repository::open(&repo_path) {
            Ok(repo) => {
                // Reset working tree to clean state.
                let mut checkout = git2::build::CheckoutBuilder::new();
                checkout.force();
                checkout.remove_untracked(true);
                if let Ok(head) = repo.head() {
                    if let Ok(commit) = head.peel_to_commit() {
                        repo.reset(
                            commit.as_object(),
                            git2::ResetType::Hard,
                            Some(&mut checkout),
                        )?;
                    }
                }
                repo
            }
            Err(_) => git2::Repository::init(&repo_path)?,
        };

        // Write files to the working tree.
        for (name, content) in files {
            let file_path = repo_path.join(name);
            std::fs::write(&file_path, content)?;
        }

        // Stage all files.
        let mut index = repo.index()?;
        for (name, _) in files {
            index.add_path(Path::new(name))?;
        }
        index.write()?;
        let tree_oid = index.write_tree()?;
        let tree = repo.find_tree(tree_oid)?;

        // Create commit.
        let sig = git2::Signature::now(&self.commit_author_name, &self.commit_author_email)?;
        let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
        let parents: Vec<&git2::Commit> = parent.iter().collect();
        let commit_oid = repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            commit_message,
            &tree,
            &parents,
        )?;

        Ok(commit_oid.to_string())
    }
}

/// Returns true if the git error represents a "not found" condition.
fn is_not_found(e: &super::Error) -> bool {
    match e {
        super::Error::Git(e) => {
            e.code() == git2::ErrorCode::NotFound
                || e.class() == git2::ErrorClass::Object
                || e.class() == git2::ErrorClass::Reference
        }
        _ => false,
    }
}

/// Reads a file from a git repository at a specific commit.
fn read_file_at_commit(
    repo_path: &Path,
    file_name: &str,
    commit_sha: &str,
) -> Result<String, super::Error> {
    let repo = git2::Repository::open(repo_path)?;
    let oid = git2::Oid::from_str(commit_sha)?;
    let commit = repo.find_commit(oid)?;
    let tree = commit.tree()?;
    let entry = tree
        .get_name(file_name)
        .ok_or_else(|| git2::Error::from_str(&format!("{} not found at commit {}", file_name, commit_sha)))?;
    let blob = repo.find_blob(entry.id())?;
    let content = std::str::from_utf8(blob.content())?;
    Ok(content.to_string())
}
