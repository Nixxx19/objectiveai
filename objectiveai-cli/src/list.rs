use clap::Subcommand;
use serde::{Serialize, Deserialize};

#[derive(Subcommand)]
pub enum Source {
    /// List from the local filesystem
    Filesystem,
    /// List from favorites
    Favorites,
    /// List from ObjectiveAI
    Objectiveai,
    /// List from all sources
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ListItem {
    Favorite(objectiveai::config::Favorite),
    Item(objectiveai::RemotePath),
}

fn format_items(items: Vec<ListItem>) -> String {
    serde_json::to_string_pretty(&items).unwrap()
}

/// Returns true if a favorite matches a remote path (same remote+owner+repo,
/// and if the favorite specifies a commit, it must match too).
fn favorite_matches(fav: &objectiveai::config::Favorite, path: &objectiveai::RemotePath) -> bool {
    match (fav.path(), path) {
        (
            objectiveai::RemotePathCommitOptional::Github { owner: fo, repository: fr, commit: fc },
            objectiveai::RemotePath::Github { owner: po, repository: pr, commit: pc },
        ) => fo == po && fr == pr && fc.as_ref().is_none_or(|c| c == pc),
        (
            objectiveai::RemotePathCommitOptional::Filesystem { owner: fo, repository: fr, commit: fc },
            objectiveai::RemotePath::Filesystem { owner: po, repository: pr, commit: pc },
        ) => fo == po && fr == pr && fc.as_ref().is_none_or(|c| c == pc),
        (
            objectiveai::RemotePathCommitOptional::Mock { name: fn_ },
            objectiveai::RemotePath::Mock { name: pn },
        ) => fn_ == pn,
        _ => false,
    }
}

/// Returns favorites only. No API call.
pub fn favorites(
    get_favorites: impl FnOnce() -> Vec<objectiveai::config::Favorite>,
) -> Result<crate::Output, crate::error::Error> {
    let items: Vec<ListItem> = get_favorites()
        .into_iter()
        .map(ListItem::Favorite)
        .collect();
    Ok(crate::Output::Api(format_items(items)))
}

/// Fetches from a single remote source via api::run.
pub async fn single<F>(
    list_remote: F,
) -> Result<crate::Output, crate::error::Error>
where
    F: FnOnce(objectiveai::HttpClient) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<objectiveai::RemotePath>, crate::error::Error>> + Send>> + Send + 'static,
{
    crate::api::run(|http_client| async move {
        let items: Vec<ListItem> = list_remote(http_client).await?
            .into_iter()
            .map(ListItem::Item)
            .collect();
        Ok(format_items(items))
    }).await
}

/// Fetches from all sources with de-duplication via api::run.
///
/// 1. Favorites first
/// 2. Filesystem items that don't match any favorite
/// 3. Objectiveai items that don't match any favorite or filesystem item
pub async fn all<FsF, OaiF>(
    get_favorites: impl FnOnce() -> Vec<objectiveai::config::Favorite> + Send + 'static,
    list_filesystem: FsF,
    list_objectiveai: OaiF,
) -> Result<crate::Output, crate::error::Error>
where
    FsF: FnOnce(objectiveai::HttpClient) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<objectiveai::RemotePath>, crate::error::Error>> + Send>> + Send + 'static,
    OaiF: FnOnce(objectiveai::HttpClient) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<objectiveai::RemotePath>, crate::error::Error>> + Send>> + Send + 'static,
{
    crate::api::run(|http_client| async move {
        let favorites = get_favorites();

        let (fs_result, oai_result) = tokio::join!(
            list_filesystem(http_client.clone()),
            list_objectiveai(http_client),
        );
        let fs_items = fs_result?;
        let oai_items = oai_result?;

        let mut items: Vec<ListItem> = Vec::new();

        // Favorites first
        for fav in &favorites {
            items.push(ListItem::Favorite(fav.clone()));
        }

        // Filesystem items, skipping any that match a favorite
        for item in fs_items {
            if !favorites.iter().any(|fav| favorite_matches(fav, &item)) {
                items.push(ListItem::Item(item));
            }
        }

        // Objectiveai items, skipping any that match a favorite or filesystem item
        for item in oai_items {
            let dominated = favorites.iter().any(|fav| favorite_matches(fav, &item))
                || items.iter().any(|existing| match existing {
                    ListItem::Item(p) => p == &item,
                    _ => false,
                });
            if !dominated {
                items.push(ListItem::Item(item));
            }
        }

        Ok(format_items(items))
    }).await
}
