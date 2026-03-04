use crate::error::SJMCLResult;
use crate::instance::models::misc::ModLoaderType;
use crate::resource::helpers::misc::get_download_api;
use crate::resource::models::{ModLoaderResourceInfo, ResourceError, ResourceType, SourceType};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_plugin_http::reqwest;

#[derive(Serialize, Deserialize, Debug)]
struct CleanroomMetaItem {
  pub name: String,
  pub created_at: String,
}

lazy_static! {
  static ref OLD_VERSION_REGEX: Regex = Regex::new(r"^(\d+)\.(\d+)\.(\d+)(?:-.*)?$").unwrap();
}

async fn get_cleanroom_meta_by_game_version_official(
  app: &AppHandle,
) -> SJMCLResult<Vec<ModLoaderResourceInfo>> {
  let client = app.state::<reqwest::Client>();

  let url = get_download_api(SourceType::Official, ResourceType::CleanroomMeta)?;
  let response = client
    .get(url)
    .send()
    .await
    .map_err(|_| ResourceError::NetworkError)?;

  if !response.status().is_success() {
    return Err(ResourceError::NetworkError.into());
  }

  let versions: Vec<CleanroomMetaItem> = response
    .json()
    .await
    .map_err(|_| ResourceError::ParseError)?;

  let mut results = Vec::new();

  for item in versions {
    if let Some(cap) = OLD_VERSION_REGEX.captures(&item.name) {
      let major: i32 = cap[1].parse()?;
      let minor: i32 = cap[2].parse()?;
      let patch: i32 = cap[3].parse()?;

      results.push((
        (major, minor, patch),
        ModLoaderResourceInfo {
          loader_type: ModLoaderType::Cleanroom,
          version: item.name.clone(),
          description: item.created_at.clone(),
          stable: !item.name.contains("alpha"),
          branch: None,
        },
      ));
    }
  }

  results.sort_by(|a, b| b.0.cmp(&a.0));

  Ok(results.into_iter().map(|r| r.1).collect())
}

pub async fn get_cleanroom_meta_by_game_version(
  app: &AppHandle,
  priority_list: &[SourceType],
  game_version: &str,
) -> SJMCLResult<Vec<ModLoaderResourceInfo>> {
  if game_version != "1.12.2" {
    return Ok(Vec::new());
  }

  if let Ok(meta) = get_cleanroom_meta_by_game_version_official(app).await {
    return Ok(meta);
  }

  Err(ResourceError::NetworkError.into())
}
