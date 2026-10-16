use std::panic::catch_unwind;

use tsox_tsoptions::tsoptions::apply_test_settings;

use crate::case_parser::{extract_settings, split_units};
use crate::compiler_run::build_and_check;
use crate::config_expand::compute_configurations;
use crate::errors_render::render_errors_baseline;
use crate::program_setup::detect_tsconfig;
use crate::skip_rules::{should_skip, static_skip_reason, ts_go_skip_reason};

pub enum CaseOutcome {

    Skip(String),

    Output(String),
}

pub struct ConfigOutcome {

    pub suffix: String,

    pub outcome: CaseOutcome,
}

pub fn process_case(content: &str, basename: &str) -> Vec<ConfigOutcome> {
    let configs = process_case_inner(content, basename);
    crate::probe::emit_case(basename);
    configs
}

fn process_case_inner(content: &str, basename: &str) -> Vec<ConfigOutcome> {
    let settings = {
        let settings_phase = crate::probe::enter(crate::probe::Slot::Read);
        extract_settings(content)
    };

    if let Some(reason) = static_skip_reason(basename) {
        return vec![ConfigOutcome {
            suffix: String::new(),
            outcome: CaseOutcome::Skip(reason),
        }];
    }

    if let Some(reason) = ts_go_skip_reason(basename) {
        return vec![ConfigOutcome {
            suffix: String::new(),
            outcome: CaseOutcome::Skip(reason),
        }];
    }

    let parsed = {
        let parse_phase = crate::probe::enter(crate::probe::Slot::Read);
        split_units(content, basename)
    };

    let tsconfig = detect_tsconfig(&parsed.units);
    match compute_configurations(&settings) {
        Err(reason) => vec![ConfigOutcome {
            suffix: String::new(),
            outcome: CaseOutcome::Skip(reason),
        }],
        Ok(configs) => configs
            .into_iter()
            .map(|(suffix, config_settings)| {
                let (mut compiler_options, unrecognized) = match &tsconfig {
                    Some((parsed_config, config_path)) => {
                        let (mut opts, unrec) = tsox_tsoptions::tsoptions::apply_test_settings_with_base(
                            &config_settings,
                            parsed_config.compiler_options.clone(),
                        );

                        if opts.config_file_path.is_empty() {
                            opts.config_file_path = config_path.clone();
                        }
                        (opts, unrec)
                    }
                    None => apply_test_settings(&config_settings),
                };
                if std::env::var_os("TSOX_PROBE_SKIPLIB").is_some() {
                    compiler_options.skip_default_lib_check = tsox_core::core::tristate::Tristate::True;
                }
                if std::env::var_os("TSOX_DEBUG_OPTIONS").is_some() {
                    eprintln!(
                        "[opts] module={:?} modres={:?} target={:?} decl={} out={} jsxisrc={:?} nia={:?} strict={:?}",
                        compiler_options.module,
                        compiler_options.module_resolution,
                        compiler_options.target,
                        compiler_options.declaration.is_true(),
                        compiler_options.out_dir,
                        compiler_options.jsx_import_source,
                        compiler_options.no_implicit_any,
                        compiler_options.strict,
                    );
                }
                let outcome = if let Some(reason) = should_skip(&compiler_options, &unrecognized) {
                    CaseOutcome::Skip(reason)
                } else {

                    let no_implicit_refs = content.lines().any(|l| {
                        let t = l.trim_start();
                        t.starts_with("//")
                            && t
                                .to_ascii_lowercase()
                                .starts_with("// @noimplicitreferences:")
                            && t.contains("true")
                    });
                    match catch_unwind(|| {
                        let (diags, program_len, types_text) = build_and_check(
                            &compiler_options,
                            &parsed.units,
                            no_implicit_refs,
                            tsconfig.as_ref().map(|(c, _)| c.file_names.as_slice()),
                            &parsed.symlinks,
                        );
                        if let (Some(dir), Some(text)) =
                            (std::env::var_os("TSOX_TYPES_EMIT_DIR"), types_text.as_ref())
                        {
                            let stem = std::env::var("TSOX_TYPES_STEM").unwrap_or_default();
                            let file = if suffix.is_empty() {
                                format!("{stem}.types")
                            } else {
                                format!("{stem}({suffix}).types")
                            };
                            let _ = std::fs::write(std::path::Path::new(&dir).join(file), text);
                        }
                        let output = {
                            let render_phase = crate::probe::enter(crate::probe::Slot::Render);
                            render_errors_baseline(
                                &diags,
                                program_len,
                                compiler_options.pretty.is_true(),
                            )
                        };
                        output
                    }) {
                        Ok(actual) => CaseOutcome::Output(actual),
                        Err(payload) => {
                            let msg = payload
                                .downcast_ref::<&str>()
                                .copied()
                                .or_else(|| payload.downcast_ref::<String>().map(|s| s.as_str()))
                                .unwrap_or("<non-string panic>");
                            CaseOutcome::Skip(format!("panicked: {msg}"))
                        }
                    }
                };
                ConfigOutcome { suffix, outcome }
            })
            .collect(),
    }
}

pub fn payload_entries(configs: &[ConfigOutcome]) -> Vec<serde_json::Value> {
    configs
        .iter()
        .map(|c| {
            let mut entry = serde_json::json!({ "suffix": c.suffix });
            match &c.outcome {
                CaseOutcome::Skip(reason) => {
                    entry["skip"] = serde_json::json!(reason);
                }
                CaseOutcome::Output(text) => {
                    entry["text"] = serde_json::json!(text);
                }
            }
            entry
        })
        .collect()
}
