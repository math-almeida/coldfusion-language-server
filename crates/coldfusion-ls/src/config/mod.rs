use std::{iter, path::PathBuf};
use virtual_fs::AbsPathBuf;

use serde::de::DeserializeOwned;
#[derive(Debug, Clone)]
pub struct ManifestPath {
    file: AbsPathBuf,
}
#[derive(Debug, Clone)]
pub enum ProjectManifest {
    BoxJson(ManifestPath),
}

#[derive(Debug)]
pub struct ConfigError {
    errors: Vec<(String, serde_json::Error)>,
}

#[derive(Debug, Clone)]
pub struct Config {
    root_path: AbsPathBuf,
    capabilities: lsp_types::ClientCapabilities,
    workspace_roots: Vec<AbsPathBuf>,
    detached_files: Vec<AbsPathBuf>,
    discovered_projects: Vec<ProjectManifest>,
}
impl Config {
    pub fn new(
        root_path: AbsPathBuf,
        capabilities: lsp_types::ClientCapabilities,
        workspace_roots: Vec<AbsPathBuf>,
    ) -> Self {
        Config {
            root_path,
            capabilities,
            workspace_roots,
            detached_files: Vec::new(),
            discovered_projects: Vec::new(),
        }
    }

    pub fn update(&mut self, mut json: serde_json::Value) -> Result<(), ConfigError> {
        if json.is_null() || json.as_object().map_or(false, |it| it.is_empty()) {
            return Ok(());
        }

        let mut errors = Vec::new();
        self.detached_files =
            get_field::<Vec<PathBuf>>(&mut json, &mut errors, "detachedFiles", None, "[]")
                .into_iter()
                .map(AbsPathBuf::assert)
                .collect();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ConfigError { errors })
        }
    }
}

fn get_field<T: DeserializeOwned>(
    json: &mut serde_json::Value,
    error_sink: &mut Vec<(String, serde_json::Error)>,
    field: &'static str,
    alias: Option<&'static str>,
    default: &str,
) -> T {
    alias
        .into_iter()
        .chain(iter::once(field))
        .filter_map(move |field| {
            let mut pointer = field.replace('_', "/");
            pointer.insert(0, '/');
            json.pointer_mut(&pointer)
                .map(|it| serde_json::from_value(it.take()).map_err(|e| (e, pointer)))
        })
        .find(Result::is_ok)
        .and_then(|res| match res {
            Ok(it) => Some(it),
            Err((e, pointer)) => {
                tracing::warn!("Failed to deserialize config field at {}: {:?}", pointer, e);
                error_sink.push((pointer, e));
                None
            }
        })
        .unwrap_or_else(|| {
            serde_json::from_str(default).unwrap_or_else(|e| panic!("{e} on: `{default}`"))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_config_new() {
        let root_path = AbsPathBuf::try_from("/tmp").unwrap();
        let capabilities = lsp_types::ClientCapabilities::default();
        let workspace_roots = vec![AbsPathBuf::try_from("/tmp").unwrap()];
        let config = Config::new(
            root_path.clone(),
            capabilities.clone(),
            workspace_roots.clone(),
        );
        assert_eq!(config.root_path, root_path);
        assert_eq!(config.capabilities, capabilities);
        assert_eq!(config.workspace_roots, workspace_roots);
    }

    #[test]
    fn test_manifest_path() {
        let file = AbsPathBuf::try_from("/tmp/box.json").unwrap();
        let manifest_path = ManifestPath { file: file.clone() };
        assert_eq!(manifest_path.file, file);
    }

    #[test]
    fn test_project_manifest() {
        let file = AbsPathBuf::try_from("/tmp/box.json").unwrap();
        let manifest_path = ManifestPath { file: file.clone() };
        let project_manifest = ProjectManifest::BoxJson(manifest_path);
        match project_manifest {
            ProjectManifest::BoxJson(manifest_path) => {
                assert_eq!(manifest_path.file, file);
            }
        }
    }

    #[test]
    fn test_config_discovered_projects() {
        let file = AbsPathBuf::try_from("/tmp/box.json").unwrap();
        let manifest_path = ManifestPath { file: file.clone() };
        let project_manifest = ProjectManifest::BoxJson(manifest_path);
        let mut config = Config::new(
            AbsPathBuf::try_from("/tmp").unwrap(),
            lsp_types::ClientCapabilities::default(),
            vec![AbsPathBuf::try_from("/tmp").unwrap()],
        );
        config.discovered_projects.push(project_manifest);
        assert_eq!(config.discovered_projects.len(), 1);
    }

    #[test]
    fn test_config_update() {
        let mut config = Config::new(
            AbsPathBuf::try_from("/tmp").unwrap(),
            lsp_types::ClientCapabilities::default(),
            vec![AbsPathBuf::try_from("/tmp").unwrap()],
        );
        let json = serde_json::json!({
            "detachedFiles": ["/tmp/box.json"]
        });
        let result = config.update(json);
        assert!(result.is_ok());
        assert_eq!(config.detached_files.len(), 1);
    }

    #[test]
    fn test_config_update_error() {
        let mut config = Config::new(
            AbsPathBuf::try_from("/tmp").unwrap(),
            lsp_types::ClientCapabilities::default(),
            vec![AbsPathBuf::try_from("/tmp").unwrap()],
        );
        let json = serde_json::json!({
            "detachedFiles": ["/tmp/box.json"]
        });
        let result = config.update(json);
        assert!(result.is_ok());
        assert_eq!(config.detached_files.len(), 1);
    }

    #[test]
    fn test_get_field() {
        let mut json = serde_json::json!({
            "detachedFiles": ["/tmp/box.json"]
        });
        let mut errors = Vec::new();
        let result: Vec<PathBuf> =
            get_field::<Vec<PathBuf>>(&mut json, &mut errors, "detachedFiles", None, "[]");
        assert_eq!(result.len(), 1);
    }
}
