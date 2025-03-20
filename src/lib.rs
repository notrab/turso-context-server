use serde::Deserialize;
use zed::settings::ContextServerSettings;
use zed_extension_api::{self as zed, serde_json, Command, ContextServerId, Project, Result};

const PACKAGE_NAME: &str = "mcp-turso";
const PACKAGE_VERSION: &str = "0.1.4";

/// Extension for running the Turso context server
struct TursoModelContextExtension;

#[derive(Debug, Deserialize)]
/// Settings required for the Turso context server
struct TursoContextServerSettings {
    /// The URL of the Turso database
    database_url: String,
    /// The authentication token for Turso
    auth_token: String,
}

impl zed::Extension for TursoModelContextExtension {
    fn new() -> Self {
        Self
    }

    fn context_server_command(
        &mut self,
        _context_server_id: &ContextServerId,
        project: &Project,
    ) -> Result<Command> {
        let version = match zed::npm_package_installed_version(PACKAGE_NAME) {
            Ok(v) => {
                eprintln!("npm package check result: {:?}", v);
                v
            }
            Err(e) => {
                eprintln!("npm package check error: {:?}", e);
                return Err(format!("Failed to check npm package: {}", e).into());
            }
        };

        if version.is_none() {
            eprintln!("Installing npm package: {}", PACKAGE_NAME);
            match zed::npm_install_package(PACKAGE_NAME, PACKAGE_VERSION) {
                Ok(_) => eprintln!("Successfully installed npm package"),
                Err(e) => {
                    eprintln!("Failed to install npm package: {:?}", e);
                    return Err(format!("Failed to install npm package: {}", e).into());
                }
            }
        }

        let settings = ContextServerSettings::for_project("turso-context-server", project)?;
        let Some(settings) = settings.settings else {
            return Err("missing Turso settings".into());
        };
        let settings: TursoContextServerSettings = serde_json::from_value(settings)
            .map_err(|e| format!("Invalid Turso settings format: {}", e))?;

        if settings.database_url.is_empty() {
            return Err("Turso database URL is required".into());
        }
        if settings.auth_token.is_empty() {
            return Err("Turso authentication token is required".into());
        }

        Ok(Command {
            command: "npx".to_string(),
            args: vec!["-y".to_string(), PACKAGE_NAME.to_string()],
            env: vec![
                ("TURSO_DATABASE_URL".into(), settings.database_url),
                ("TURSO_AUTH_TOKEN".into(), settings.auth_token),
            ],
        })
    }
}

zed::register_extension!(TursoModelContextExtension);
