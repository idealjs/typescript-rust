use super::json::{JsonValue, JsonValueType};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ObjectKind {
    #[default]
    Unknown,
    Subpaths,
    Conditions,
    Imports,
    Invalid,
}

#[derive(Clone, Debug, Default)]
pub struct ExportsOrImports {
    pub json_value: JsonValue,
    pub(super) object_kind: ObjectKind,
}

impl ExportsOrImports {
    pub fn from_json_value(json_value: JsonValue) -> Self { ::tsox_core::fntrace::enter("from_json_value"); 
        ExportsOrImports {
            json_value,
            object_kind: ObjectKind::Unknown,
        }
    }

    pub fn is_subpaths(&self) -> bool { ::tsox_core::fntrace::enter("is_subpaths"); 
        self.compute_object_kind() == ObjectKind::Subpaths
    }

    pub fn is_imports(&self) -> bool { ::tsox_core::fntrace::enter("is_imports"); 
        self.compute_object_kind() == ObjectKind::Imports
    }

    pub fn is_conditions(&self) -> bool { ::tsox_core::fntrace::enter("is_conditions"); 
        self.compute_object_kind() == ObjectKind::Conditions
    }

    pub fn compute_object_kind(&self) -> ObjectKind { ::tsox_core::fntrace::enter("compute_object_kind"); 
        if self.object_kind != ObjectKind::Unknown {
            return self.object_kind.clone();
        }
        if self.json_value.value_type != JsonValueType::Object {
            return ObjectKind::Unknown;
        }
        if let Some(obj) = &self.json_value.object_value {
            if obj.is_empty() {
                return ObjectKind::Conditions;
            }
            let mut seen_dot = false;
            let mut seen_hash = false;
            let mut seen_other = false;
            for (k, _) in obj {
                if let Some(first) = k.chars().next() {
                    if first == '.' {
                        seen_dot = true;
                    } else if first == '#' {
                        seen_hash = true;
                    } else {
                        seen_other = true;
                    }
                    if seen_other && (seen_dot || seen_hash) {
                        return ObjectKind::Invalid;
                    }
                }
            }
            if seen_dot {
                return ObjectKind::Subpaths;
            } else if seen_hash {
                return ObjectKind::Imports;
            } else {
                return ObjectKind::Conditions;
            }
        }
        ObjectKind::Unknown
    }
}
