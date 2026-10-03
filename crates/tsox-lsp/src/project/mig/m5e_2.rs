#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use std::sync::Arc;

use tsox_core::tspath::Path;

use crate::project::session::*;

pub(crate) use super::m5e::background;

mod core {
    pub use tsox_core::core::compiler_options_kinds::{JsxEmit, ModuleKind, ModuleResolutionKind, ScriptTarget};
    pub use tsox_core::core::mig::m3k::version;
    pub use tsox_core::core::tristate::Tristate;
    pub use tsox_frontend::ast::node_source_file::ScriptKind;
}

use tsox_frontend::ast;

use crate::project::project::{Kind as ProjectKind, Project};

fn set_tristate(m: &mut HashMap<String, serde_json::Value>, key: &str, v: core::Tristate) { ::tsox_core::fntrace::enter("set_tristate"); 
    if v == core::Tristate::True {
        m.insert(key.to_string(), serde_json::Value::Bool(true));
    } else if v == core::Tristate::False {
        m.insert(key.to_string(), serde_json::Value::Bool(false));
    }
}

fn bool_telemetry(v: bool) -> String { ::tsox_core::fntrace::enter("bool_telemetry"); 
    if v {
        "true".to_string()
    } else {
        "false".to_string()
    }
}

fn count_file_stats(source_files: &[Arc<ast::SourceFile>]) -> lsproto::ProjectInfoTelemetryMeasurements { ::tsox_core::fntrace::enter("count_file_stats"); 
    let mut stats = lsproto::ProjectInfoTelemetryMeasurements::default();
    for sf in source_files {
        let size = sf.node.loc.end as f64;
        match sf.script_kind {
            core::ScriptKind::Js => {
                stats.js_file_count += 1.0;
                stats.js_file_size += size;
            }
            core::ScriptKind::Jsx => {
                stats.jsx_file_count += 1.0;
                stats.jsx_file_size += size;
            }
            core::ScriptKind::Ts => {
                if tsox_core::tspath::is_declaration_file_name(&sf.file_name) {
                    stats.dts_file_count += 1.0;
                    stats.dts_file_size += size;
                } else {
                    stats.ts_file_count += 1.0;
                    stats.ts_file_size += size;
                }
            }
            core::ScriptKind::Tsx => {
                stats.tsx_file_count += 1.0;
                stats.tsx_file_size += size;
            }
            _ => {}
        }
    }
    stats
}

impl Session {
    pub fn send_performance_telemetry(&self, ctx: &background::Context) { ::tsox_core::fntrace::enter("send_performance_telemetry"); 
        let Some(client) = &self.client else { return };
        if !self.options.telemetry_enabled {
            return;
        }
        let snapshot = self.snapshot().expect("snapshot");
        let mut measurements = lsproto::PerformanceStatsTelemetryMeasurements::default();
        measurements.open_file_count = snapshot
            .fs
            .as_ref()
            .map(|fs| fs.overlays.len())
            .unwrap_or(0) as f64;
        measurements.uptime_seconds = self.start_time.elapsed().as_secs_f64();
        measurements.project_count = snapshot
            .project_collection
            .as_ref()
            .map(|pc| pc.projects_by_path().len())
            .unwrap_or(0) as f64;
        measurements.config_count = snapshot
            .config_file_registry
            .as_ref()
            .map(|registry| registry.configs.len())
            .unwrap_or(0) as f64;
        measurements.cached_disk_file_count = snapshot
            .fs
            .as_ref()
            .map(|fs| fs.disk_files.len())
            .unwrap_or(0) as f64;
        if let Some(registry) = snapshot.auto_import_registry() {
            let auto_import_stats = registry.get_cache_stats();
            measurements.auto_import_project_bucket_count =
                auto_import_stats.project_buckets.len() as f64;
            measurements.auto_import_node_modules_bucket_count =
                auto_import_stats.node_modules_buckets.len() as f64;
            measurements.auto_import_unique_package_count =
                auto_import_stats.unique_package_count as f64;
            for b in &auto_import_stats.project_buckets {
                measurements.auto_import_project_export_count += b.export_count as f64;
                measurements.auto_import_project_file_count += b.file_count as f64;
            }
            for b in &auto_import_stats.node_modules_buckets {
                measurements.auto_import_node_modules_export_count += b.export_count as f64;
                measurements.auto_import_node_modules_file_count += b.file_count as f64;
                if b.dependency_names.is_none() {
                    measurements.auto_import_node_modules_unfiltered_bucket_count += 1.0;
                }
            }
        }
        let event = lsproto::TelemetryEvent {
            performance_stats_telemetry_event: Some(lsproto::PerformanceStatsTelemetryEvent {
                measurements,
            }),
            ..Default::default()
        };
        if let Err(err) = client.send_telemetry(&event) {
            if self.options.logging_enabled {
                self.sess_log(&format!("Error sending performance telemetry: {err}"));
            }
        }
    }

    pub fn send_project_info_telemetry_for_new_projects(
        &self,
        old_snapshot: &Snapshot,
        new_snapshot: &Snapshot,
    ) { ::tsox_core::fntrace::enter("send_project_info_telemetry_for_new_projects"); 
        if !self.options.telemetry_enabled {
            return;
        }
        let ctx = self.background_context();
        let old_projects = old_snapshot
            .project_collection
            .as_ref()
            .map(|pc| pc.projects_by_path())
            .unwrap_or_default();
        let new_projects = new_snapshot
            .project_collection
            .as_ref()
            .map(|pc| pc.projects_by_path())
            .unwrap_or_default();
        for (path, added_project) in &new_projects {
            if !old_projects.iter().any(|(p, _)| p == path) {
                self.send_project_info_telemetry(&ctx, added_project);
            }
        }
    }

    fn send_project_info_telemetry(&self, ctx: &background::Context, project: &Project) { ::tsox_core::fntrace::enter("send_project_info_telemetry"); 
        let Some(client) = &self.client else { return };
        if !self.options.telemetry_enabled {
            return;
        }
        if self.seen_projects.lock().unwrap().contains(&project.config_file_path) {
            return;
        }
        if project.program.is_none() || project.command_line.is_none() {
            return;
        }
        let info = self.collect_project_info_telemetry(project);
        if client.send_telemetry(&info).is_err() {
            if self.options.logging_enabled {
                self.sess_log("Error sending project info telemetry");
            }
            return;
        }
        self.seen_projects.lock().unwrap().insert(project.config_file_path.clone());
    }

    fn collect_project_info_telemetry(&self, project: &Project) -> lsproto::TelemetryEvent { ::tsox_core::fntrace::enter("collect_project_info_telemetry"); 
        let mut opts = project
            .command_line
            .as_ref()
            .map(|cl| cl.compiler_options().clone())
            .unwrap_or_default();
        let mut config_file_name = "other".to_string();
        if project.kind == ProjectKind::Configured {
            let base_name = tsox_core::tspath::get_base_file_name(&project.config_file_name);
            if base_name == "tsconfig.json" || base_name == "jsconfig.json" {
                config_file_name = base_name;
            }
        }
        let mut project_type = "inferred";
        if project.kind == ProjectKind::Configured {
            project_type = "configured";
        }
        let mut props: HashMap<String, String> = HashMap::new();
        props.insert("configFileName".into(), config_file_name);
        props.insert("projectType".into(), project_type.into());
        props.insert("version".into(), core::version().to_string());
        let mut compiler_options: HashMap<String, serde_json::Value> = HashMap::new();
        set_tristate(&mut compiler_options, "strict", opts.strict);
        set_tristate(&mut compiler_options, "noImplicitAny", opts.no_implicit_any);
        set_tristate(&mut compiler_options, "noImplicitThis", opts.no_implicit_this);
        set_tristate(&mut compiler_options, "strictNullChecks", opts.strict_null_checks);
        set_tristate(&mut compiler_options, "strictFunctionTypes", opts.strict_function_types);
        set_tristate(&mut compiler_options, "strictBindCallApply", opts.strict_bind_call_apply);
        set_tristate(&mut compiler_options, "strictPropertyInitialization", opts.strict_property_initialization);
        set_tristate(&mut compiler_options, "strictBuiltinIteratorReturn", opts.strict_builtin_iterator_return);
        set_tristate(&mut compiler_options, "useUnknownInCatchVariables", opts.use_unknown_in_catch_variables);
        set_tristate(&mut compiler_options, "exactOptionalPropertyTypes", opts.exact_optional_property_types);
        set_tristate(&mut compiler_options, "allowJs", opts.allow_js);
        set_tristate(&mut compiler_options, "checkJs", opts.check_js);
        set_tristate(&mut compiler_options, "noEmit", opts.no_emit);
        set_tristate(&mut compiler_options, "declaration", opts.declaration);
        set_tristate(&mut compiler_options, "composite", opts.composite);
        set_tristate(&mut compiler_options, "isolatedModules", opts.isolated_modules);
        set_tristate(&mut compiler_options, "skipLibCheck", opts.skip_lib_check);
        set_tristate(&mut compiler_options, "incremental", opts.incremental);
        if opts.target != core::ScriptTarget::None {
            compiler_options.insert("target".into(), serde_json::Value::String(format!("{:?}", opts.target)));
        }
        if opts.module != core::ModuleKind::None {
            compiler_options.insert("module".into(), serde_json::Value::String(format!("{:?}", opts.module)));
        }
        if opts.module_resolution != core::ModuleResolutionKind::Unknown {
            compiler_options.insert(
                "moduleResolution".into(),
                serde_json::Value::String(opts.module_resolution.to_string()),
            );
        }
        if opts.jsx != core::JsxEmit::None {
            compiler_options.insert("jsx".into(), serde_json::Value::String(opts.jsx.to_string()));
        }
        props.insert(
            "compilerOptions".into(),
            serde_json::to_string(&compiler_options).unwrap_or_default(),
        );
        if let Some(raw) = project.command_line.as_ref().and_then(|cl| cl.raw_options.as_ref()) {
            props.insert("extends".into(), bool_telemetry(raw.get("extends").is_some()));
            props.insert("files".into(), bool_telemetry(raw.get("files").is_some()));
            props.insert("include".into(), bool_telemetry(raw.get("include").is_some()));
            props.insert("exclude".into(), bool_telemetry(raw.get("exclude").is_some()));
        }
        let measurements = project
            .program
            .as_ref()
            .map(|p| count_file_stats(&p.get_source_files()))
            .unwrap_or_default();
        lsproto::TelemetryEvent {
            project_info_telemetry_event: Some(lsproto::ProjectInfoTelemetryEvent {
                properties: props,
                measurements: Some(measurements),
            }),
            ..Default::default()
        }
    }
}
