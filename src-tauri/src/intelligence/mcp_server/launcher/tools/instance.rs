use crate::instance::commands::*;
use crate::intelligence::mcp_server::launcher::McpContext;
use crate::mcp_tool;
use rmcp::handler::server::tool::ToolRoute;

pub fn tool_routes() -> Vec<ToolRoute<McpContext>> {
  vec![
    mcp_tool!(
      "retrieve_instance_list",
      retrieve_instance_list,
      "Primary tool for listing local Minecraft instances. Returns instance IDs and metadata for selecting an instance."
    ),
    mcp_tool!(
      "retrieve_world_list",
      retrieve_world_list,
      "Retrieve local world metadata for a Minecraft instance.",
      #[serde(deny_unknown_fields)]
      {
        #[schemars(description = "Minecraft instance ID.")]
        instance_id: String,
      }
    ),
    mcp_tool!(
      "retrieve_world_details",
      retrieve_world_details,
      "Retrieve detailed level.dat data for a local world in a Minecraft instance.",
      #[serde(deny_unknown_fields)]
      {
        #[schemars(description = "Minecraft instance ID returned by `retrieve_instance_list`.")]
        instance_id: String,
        #[schemars(description = "World directory name returned by `retrieve_world_list`.")]
        world_name: String,
      }
    ),
    mcp_tool!(
      "retrieve_game_server_list",
      "Retrieve configured servers for a Minecraft instance and query their online status.",
      |app, params|
      #[serde(deny_unknown_fields)]
      {
        #[schemars(description = "Minecraft instance ID.")]
        instance_id: String,
      } => async move {
        // always query online status in MCP context.
        retrieve_game_server_list(app, params.instance_id, true).await
      }
    ),
    mcp_tool!(
      "delete_game_server",
      "Delete a saved multiplayer server from a Minecraft instance's server list. Requires confirm=true.",
      |app, params|
      #[serde(deny_unknown_fields)]
      {
        #[schemars(description = "Minecraft instance ID returned by `retrieve_instance_list`.")]
        instance_id: String,
        #[schemars(description = "Server address returned as `ip` by `retrieve_game_server_list`.")]
        server_addr: String,
        #[schemars(description = "Must be true to confirm deleting this saved server entry.")]
        confirm: bool,
      } => async move {
        if !params.confirm {
          return Err(crate::instance::models::misc::InstanceError::InvalidSourcePath.into());
        }

        delete_game_server(app, params.instance_id, params.server_addr).await
      }
    ),
    mcp_tool!(
      "add_game_server",
      add_game_server,
      "Add a saved multiplayer server to a Minecraft instance's server list.",
      #[serde(deny_unknown_fields)]
      {
        #[schemars(description = "Minecraft instance ID returned by `retrieve_instance_list`.")]
        instance_id: String,
        #[schemars(description = "Server address, for example `mc.example.com` or `127.0.0.1:25565`.")]
        server_addr: String,
        #[schemars(description = "Display name saved for this server entry.")]
        server_name: String,
      }
    ),
    mcp_tool!(
      sync "retrieve_instance_game_config",
      retrieve_instance_game_config,
      "Retrieve the effective game configuration for a Minecraft instance. If the instance does not use a dedicated config, this returns the global game configuration currently applied to it.",
      #[serde(deny_unknown_fields)]
      {
        #[schemars(description = "Minecraft instance ID returned by `retrieve_instance_list`.")]
        instance_id: String,
      }
    ),
    mcp_tool!(
      "restore_instance_game_config",
      restore_instance_game_config,
      "Restore a Minecraft instance's dedicated game configuration to the current global game configuration.",
      #[serde(deny_unknown_fields)]
      {
        #[schemars(description = "Minecraft instance ID returned by `retrieve_instance_list`.")]
        instance_id: String,
      }
    ),
    mcp_tool!(
      "delete_instance",
      "Delete a Minecraft instance and its instance directory. Requires confirm=true.",
      |app, params|
      #[serde(deny_unknown_fields)]
      {
        #[schemars(description = "Minecraft instance ID returned by `retrieve_instance_list`.")]
        instance_id: String,
        #[schemars(description = "Must be true to confirm deleting this instance and its files.")]
        confirm: bool,
      } => async move {
        if !params.confirm {
          return Err(crate::instance::models::misc::InstanceError::InvalidSourcePath.into());
        }

        delete_instance(app, params.instance_id)
      }
    ),
    mcp_tool!(
      "rename_instance",
      rename_instance,
      "Rename a Minecraft instance and return its new instance path.",
      #[serde(deny_unknown_fields)]
      {
        #[schemars(description = "Minecraft instance ID returned by `retrieve_instance_list`.")]
        instance_id: String,
        #[schemars(description = "New display name for the instance.")]
        new_name: String,
      }
    ),
    mcp_tool!(
      "create_launch_desktop_shortcut",
      "Create a desktop shortcut that launches a Minecraft instance. Uses the instance custom icon by default.",
      |app, params|
      #[serde(deny_unknown_fields)]
      {
        #[schemars(description = "Minecraft instance ID returned by `retrieve_instance_list`.")]
        instance_id: String,
        #[schemars(description = "Shortcut icon source. Omit to use `custom`; pass an asset path to use a built-in icon.")]
        icon_src: Option<String>,
      } => async move {
        create_launch_desktop_shortcut(
          app,
          params.instance_id,
          params.icon_src.unwrap_or_else(|| "custom".to_string()),
        )
      }
    ),
    mcp_tool!(
      "retrieve_local_mod_list",
      "Retrieve local mod metadata for a Minecraft instance.",
      |app, params|
      #[serde(deny_unknown_fields)]
      {
        #[schemars(description = "Minecraft instance ID.")]
        instance_id: String,
      } => async move {
        let mut mods = retrieve_local_mod_list(app, params.instance_id).await?;
        // strip icon binary payload in MCP responses to reduce context length.
        for mod_info in &mut mods {
          mod_info.icon_src = Default::default();
        }
        Ok(mods)
      }
    ),
    mcp_tool!(
      "retrieve_resource_pack_list",
      retrieve_resource_pack_list,
      "Retrieve resource packs for a Minecraft instance.",
      #[serde(deny_unknown_fields)]
      {
        #[schemars(description = "Minecraft instance ID.")]
        instance_id: String,
      }
    ),
    mcp_tool!(
      "retrieve_server_resource_pack_list",
      retrieve_server_resource_pack_list,
      "Retrieve server resource packs for a Minecraft instance.",
      #[serde(deny_unknown_fields)]
      {
        #[schemars(description = "Minecraft instance ID.")]
        instance_id: String,
      }
    ),
    mcp_tool!(
      sync "retrieve_schematic_list",
      retrieve_schematic_list,
      "Retrieve schematics for a Minecraft instance.",
      #[serde(deny_unknown_fields)]
      {
        #[schemars(description = "Minecraft instance ID.")]
        instance_id: String,
      }
    ),
    mcp_tool!(
      sync "retrieve_shader_pack_list",
      retrieve_shader_pack_list,
      "Retrieve shader packs for a Minecraft instance.",
      #[serde(deny_unknown_fields)]
      {
        #[schemars(description = "Minecraft instance ID.")]
        instance_id: String,
      }
    ),
    mcp_tool!(
      sync "retrieve_screenshot_list",
      retrieve_screenshot_list,
      "Retrieve screenshots for a Minecraft instance.",
      #[serde(deny_unknown_fields)]
      {
        #[schemars(description = "Minecraft instance ID.")]
        instance_id: String,
      }
    ),
    mcp_tool!(
      "toggle_mod_by_extension",
      "Enable or disable a mod file by toggling its `.disabled` extension.",
      |_app, params|
      #[serde(deny_unknown_fields)]
      {
        #[schemars(description = "File path from `retrieve_local_mod_list`.")]
        file_path: String,
        #[schemars(description = "Set to true to enable the mod file, or false to disable it.")]
        enable: bool,
      } => async move {
        toggle_mod_by_extension(std::path::PathBuf::from(params.file_path), params.enable)
      }
    ),
  ]
}
