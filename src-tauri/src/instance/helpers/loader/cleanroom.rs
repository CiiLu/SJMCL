use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;
use tauri::AppHandle;
use url::Url;
use zip::ZipArchive;

use crate::error::SJMCLResult;
use crate::instance::helpers::client_json::McClientInfo;
use crate::instance::helpers::loader::common::add_library_entry;
use crate::instance::helpers::loader::forge::LegacyInstallProfile;
use crate::instance::helpers::misc::get_instance_subdir_paths;
use crate::instance::models::misc::{Instance, InstanceError, InstanceSubdirType, ModLoader};
use crate::launch::helpers::file_validator::convert_library_name_to_path;
use crate::resource::helpers::misc::{convert_url_to_target_source, get_download_api};
use crate::resource::models::{ResourceType, SourceType};
use crate::tasks::commands::schedule_progressive_task_group;
use crate::tasks::download::DownloadParam;
use crate::tasks::PTaskParam;

pub async fn install_cleanroom_loader(
  priority: &[SourceType],
  loader: &ModLoader,
  lib_dir: PathBuf,
  task_params: &mut Vec<PTaskParam>,
) -> SJMCLResult<()> {
  let loader_ver = &loader.version;

  let (installer_url, installer_coord) = {
    (
      get_download_api(SourceType::Official, ResourceType::CleanroomInstall)?.join(&format!(
        "com/cleanroommc/cleanroom/{v}/cleanroom-{v}-installer.jar",
        v = loader_ver
      ))?,
      format!("com.cleanroommc:cleanroom:{}-installer", loader.version),
    )
  };

  let installer_rel = convert_library_name_to_path(&installer_coord, None)?;
  let installer_path = lib_dir.join(&installer_rel);

  task_params.push(PTaskParam::Download(DownloadParam {
    src: installer_url,
    dest: installer_path.clone(),
    filename: None,
    sha1: None,
  }));

  Ok(())
}

pub async fn download_cleanroom_libraries(
  app: &AppHandle,
  priority: &[SourceType],
  instance: &Instance,
  client_info: &mut McClientInfo,
) -> SJMCLResult<()> {
  let subdirs = get_instance_subdir_paths(
    app,
    instance,
    &[&InstanceSubdirType::Root, &InstanceSubdirType::Libraries],
  )
  .ok_or(InstanceError::InvalidSourcePath)?;
  let [_root_dir, lib_dir] = subdirs.as_slice() else {
    return Err(InstanceError::InvalidSourcePath.into());
  };
  let mut task_params = vec![];

  let installer_coord = format!(
    "com.cleanroommc:cleanroom:{}-installer",
    instance.mod_loader.version
  );
  let installer_rel = convert_library_name_to_path(&installer_coord, None)?;
  let installer_path = lib_dir.join(&installer_rel);

  let bin_patch_rel = convert_library_name_to_path(
    &format!(
      "com.cleanroommc:cleanroom:{}:clientdata@lzma",
      instance.mod_loader.version
    ),
    None,
  )?;
  let bin_patch = lib_dir.join(bin_patch_rel);

  if !installer_path.exists() {
    return Err(InstanceError::LoaderInstallerNotFound.into());
  }

  let (profile_data, libraries_to_process) = {
    let file = File::open(&installer_path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
      let mut file = archive.by_index(i)?;
      let path = file.mangled_name();
      let outpath = if path.starts_with("maven/") {
        let relative_path = path.strip_prefix("maven/").unwrap();
        lib_dir.join(relative_path)
      } else if path == *"data/client.lzma" {
        bin_patch.clone()
      } else {
        continue;
      };

      if file.is_file() {
        if let Some(p) = outpath.parent() {
          if !p.exists() {
            fs::create_dir_all(p)?;
          }
        }
        let mut outfile = File::create(&outpath)?;
        std::io::copy(&mut file, &mut outfile)?;
      }
    }

    let mut install_profile_str = String::new();
    if let Ok(mut f) = archive.by_name("install_profile.json") {
      f.read_to_string(&mut install_profile_str)?;
    }

    if install_profile_str.is_empty() {
      return Err(InstanceError::InstallProfileParseError.into());
    }

    let profile: LegacyInstallProfile = serde_json::from_str(&install_profile_str)
      .map_err(|_| InstanceError::InstallProfileParseError)?;

    {
      let mut file = archive.by_name(&profile.install.file_path)?;
      let dest_path = lib_dir.join(convert_library_name_to_path(&profile.install.path, None)?);
      if let Some(parent) = dest_path.parent() {
        if !parent.exists() {
          fs::create_dir_all(parent)?;
        }
      }
      let mut output = File::create(&dest_path)?;
      std::io::copy(&mut file, &mut output)?;
    }

    (profile, bin_patch)
  };

  let profile = profile_data;
  let main_class = profile.version_info.main_class;
  let libraries = profile.version_info.libraries;

  client_info.main_class = Some(main_class.clone());

  let mut new_patch = McClientInfo {
    id: "cleanroom".to_string(),
    version: Some(instance.mod_loader.version.clone()),
    priority: Some(30000),
    main_class: Some(main_class.to_string()),
    inherits_from: Some(profile.version_info.inherits_from),
    minecraft_arguments: Some(profile.version_info.minecraft_arguments.clone()),
    release_time: profile.version_info.release_time,
    time: profile.version_info.time,
    type_: profile.version_info.type_,
    assets: profile.version_info.assets,
    ..Default::default()
  };

  client_info.minecraft_arguments = Some(profile.version_info.minecraft_arguments.clone());

  for lib in libraries.iter() {
    let name = lib.name.clone();
    add_library_entry(&mut client_info.libraries, &name, None)?;
    add_library_entry(&mut new_patch.libraries, &name, None)?;

    if name == profile.install.path {
      continue;
    }

    let url = if let Some(u) = &lib.url {
      Url::parse(u)?
    } else {
      get_download_api(priority[0], ResourceType::Libraries)?
    };

    let rel = convert_library_name_to_path(&name, None)?;
    let src = convert_url_to_target_source(
      &url.join(&rel)?,
      &[ResourceType::CleanroomMaven, ResourceType::Libraries],
      &priority[0],
    )?;

    task_params.push(PTaskParam::Download(DownloadParam {
      src,
      dest: lib_dir.join(&rel),
      filename: None,
      sha1: None,
    }));
  }

  client_info.patches.push(new_patch);

  let mut seen = std::collections::HashSet::new();
  task_params.retain(|param| match param {
    PTaskParam::Download(dp) => seen.insert(dp.dest.clone()),
  });

  schedule_progressive_task_group(
    app.clone(),
    format!("cleanroom-libraries?{}", instance.id),
    task_params,
    true,
  )
  .await?;

  Ok(())
}
