use crate::utils;
use std::path::Path;
use toml::Value;

/**
 * Extract the [build.target-dir] string from cargo config file
 * 
 * - Receive the workspace root path
 */
pub fn get_targer_dir_name(path: &Path) -> String {
    // Check if the file exists
    if let Some(config_file_path) = utils::find_cargo_config(&path).unwrap() {
        // Get file content as string
        let contents = std::fs::read_to_string(config_file_path).unwrap();
        // Parse file content to toml::Value
        let config = contents.parse::<Value>();

        // Check if parsed sucessfully
        if config.is_ok() {
            let config = config.ok();

            match config {
                Some(c) => {
                    // Check if [build] exists
                    if let Some(build) = c.get("build") {
                        // Check if [build] has target-dir
                        if let Some(target_dir) = build.get("target-dir") {
                            // Remove trailing "
                            return target_dir.to_string().replace("\"", "");
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Return target as default folder name
    "target".to_string()
}
