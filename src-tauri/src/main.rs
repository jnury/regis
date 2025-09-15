// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::fs;
use tauri::{command, Manager};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Server {
    pub id: String,
    pub name: String,
    pub url: String,
    pub description: String,
    pub environment: String,
    pub region: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub servers: Vec<Server>,
}

#[command]
async fn load_servers(app: tauri::AppHandle) -> Result<Vec<Server>, String> {
    println!("Loading servers from servers.json...");

    // Try multiple paths for servers.json (development vs production)
    let mut config_content = None;
    let mut last_error = String::new();

    // First try: resource directory (production)
    if let Ok(resource_dir) = app.path().resource_dir() {
        let resource_path = resource_dir.join("servers.json");
        println!("Trying resource path: {:?}", resource_path);

        match fs::read_to_string(&resource_path) {
            Ok(content) => {
                config_content = Some(content);
                println!("Successfully read from resource directory");
            }
            Err(e) => {
                last_error = format!("Resource dir failed: {}", e);
                println!("Resource dir failed: {}", e);
            }
        }
    }

    // Second try: relative to project root (development)
    if config_content.is_none() {
        let project_path = std::path::Path::new("../servers.json");
        println!("Trying project path: {:?}", project_path);

        match fs::read_to_string(project_path) {
            Ok(content) => {
                config_content = Some(content);
                println!("Successfully read from project directory");
            }
            Err(e) => {
                last_error = format!("{} | Project dir failed: {}", last_error, e);
                println!("Project dir failed: {}", e);
            }
        }
    }

    // Third try: current directory
    if config_content.is_none() {
        let current_path = std::path::Path::new("servers.json");
        println!("Trying current path: {:?}", current_path);

        match fs::read_to_string(current_path) {
            Ok(content) => {
                config_content = Some(content);
                println!("Successfully read from current directory");
            }
            Err(e) => {
                last_error = format!("{} | Current dir failed: {}", last_error, e);
                println!("Current dir failed: {}", e);
            }
        }
    }

    let config_content = config_content
        .ok_or_else(|| format!("Failed to find servers.json in any location: {}", last_error))?;

    // Parse the JSON content
    let config: ServerConfig = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse servers.json: {}", e))?;

    println!("Successfully loaded {} servers", config.servers.len());

    Ok(config.servers)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![load_servers])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}