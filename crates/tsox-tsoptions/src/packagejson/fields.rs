use super::expected::Expected;
use super::exports::ExportsOrImports;
use super::json::JsonValue;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct HeaderFields {
    pub name: Expected<String>,
    pub version: Expected<String>,
    pub r#type: Expected<String>,
}

#[derive(Clone, Debug, Default)]
pub struct PathFields {
    pub tsconfig: Expected<String>,
    pub main: Expected<String>,
    pub types: Expected<String>,
    pub typings: Expected<String>,
    pub types_versions: JsonValue,
    pub imports: ExportsOrImports,
    pub exports: ExportsOrImports,
}

#[derive(Clone, Debug, Default)]
pub struct DependencyFields {
    pub dependencies: Expected<HashMap<String, String>>,
    pub dev_dependencies: Expected<HashMap<String, String>>,
    pub peer_dependencies: Expected<HashMap<String, String>>,
    pub optional_dependencies: Expected<HashMap<String, String>>,
}

impl DependencyFields {
    pub fn has_dependency(&self, name: &str) -> bool {
        self.dependencies
            .get_value()
            .map_or(false, |d| d.contains_key(name))
            || self
                .dev_dependencies
                .get_value()
                .map_or(false, |d| d.contains_key(name))
            || self
                .peer_dependencies
                .get_value()
                .map_or(false, |d| d.contains_key(name))
            || self
                .optional_dependencies
                .get_value()
                .map_or(false, |d| d.contains_key(name))
    }

    pub fn for_each_dependency<F: FnMut(&str, &str, &str) -> bool>(&self, mut f: F) {
        if let Some(deps) = self.dependencies.get_value() {
            for (name, version) in deps {
                if !f(name, version, "dependencies") {
                    return;
                }
            }
        }
        if let Some(deps) = self.dev_dependencies.get_value() {
            for (name, version) in deps {
                if !f(name, version, "devDependencies") {
                    return;
                }
            }
        }
        if let Some(deps) = self.peer_dependencies.get_value() {
            for (name, version) in deps {
                if !f(name, version, "peerDependencies") {
                    return;
                }
            }
        }
        if let Some(deps) = self.optional_dependencies.get_value() {
            for (name, version) in deps {
                if !f(name, version, "optionalDependencies") {
                    return;
                }
            }
        }
    }

    pub fn get_runtime_dependency_names(&self) -> std::collections::HashSet<String> {
        let mut names = std::collections::HashSet::new();
        if let Some(deps) = self.dependencies.get_value() {
            names.extend(deps.keys().cloned());
        }
        if let Some(deps) = self.peer_dependencies.get_value() {
            names.extend(deps.keys().cloned());
        }
        if let Some(deps) = self.optional_dependencies.get_value() {
            names.extend(deps.keys().cloned());
        }
        names
    }
}

#[derive(Clone, Debug, Default)]
pub struct Fields {
    pub header_fields: HeaderFields,
    pub path_fields: PathFields,
    pub dependency_fields: DependencyFields,
}
