#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::{Arc, Mutex};

use super::m5a2_2::fs_baseline_util;
use super::m5b::TestSys;
use super::tsctests::baseline;
use super::tsctests::{TscEdit, TscInput};
use crate::execute::version::{self, System};
use tsox_core::tspath::{self, EXTENSION_TS_BUILD_INFO};
use tsox_tsoptions::vfs::FS;

struct BaselineWriter<'a> {
    buffer: &'a mut String,
}

impl std::io::Write for BaselineWriter<'_> {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> { ::tsox_core::fntrace::enter("write"); 
        self.buffer.push_str(&String::from_utf8_lossy(data));
        Ok(data.len())
    }

    fn flush(&mut self) -> std::io::Result<()> { ::tsox_core::fntrace::enter("flush"); 
        Ok(())
    }
}

fn fs_differ_changed_paths(differ: &fs_baseline_util::FsDiffer) -> Vec<fs_baseline_util::FileChange> { ::tsox_core::fntrace::enter("fs_differ_changed_paths"); 
    let Some(previous) = differ.serialized_diff() else {
        return Vec::new();
    };
    let map_fs = differ.map_fs();
    let mut changes = Vec::new();
    for (path, file) in map_fs.entries() {
        if file.is_symlink || file.is_dir {
            continue;
        }
        let changed = match previous.snap.get(&path) {
            None => true,
            Some(old) => file.data != old.content || file.mod_time != old.m_time,
        };
        if changed {
            changes.push(fs_baseline_util::FileChange {
                path: path.clone(),
                deleted: false,
            });
        }
    }
    for path in previous.snap.keys() {
        if map_fs.get_file_info(path).is_none() {
            changes.push(fs_baseline_util::FileChange {
                path: path.clone(),
                deleted: true,
            });
        }
    }
    changes
}

impl TscInput {
    pub fn execute_command(
        &self,
        sys: &TestSys,
        baseline_builder: &mut String,
        command_line_args: &[String],
    ) -> crate::execute::CommandLineResult { ::tsox_core::fntrace::enter("execute_command"); 
        baseline_builder.push_str(&format!("tsgo {}\n", command_line_args.join(" ")));
        let result = crate::execute::command_line(sys, command_line_args);
        match result.status {
            crate::execute::ExitStatus::Success => {
                baseline_builder.push_str("ExitStatus:: Success");
            }
            crate::execute::ExitStatus::DiagnosticsPresent_OutputsSkipped => {
                baseline_builder
                    .push_str("ExitStatus:: DiagnosticsPresent_OutputsSkipped");
            }
            crate::execute::ExitStatus::DiagnosticsPresent_OutputsGenerated => {
                baseline_builder
                    .push_str("ExitStatus:: DiagnosticsPresent_OutputsGenerated");
            }
            crate::execute::ExitStatus::InvalidProject_OutputsSkipped => {
                baseline_builder
                    .push_str("ExitStatus:: InvalidProject_OutputsSkipped");
            }
            crate::execute::ExitStatus::ProjectReferenceCycle_OutputsSkipped => {
                baseline_builder
                    .push_str("ExitStatus:: ProjectReferenceCycle_OutputsSkipped");
            }
            crate::execute::ExitStatus::NotImplemented => {
                baseline_builder.push_str("ExitStatus:: NotImplemented");
            }
            _ => panic!("UnknownExitStatus {}", result.status.as_i32()),
        }
        result
    }

    pub fn get_baseline_sub_folder(&self) -> String { ::tsox_core::fntrace::enter("get_baseline_sub_folder"); 
        let mut command_name = "tsc";
        let command_line_args = self.command_line_args.as_deref().unwrap_or(&[]);
        if command_line_args
            .iter()
            .any(|arg| matches!(arg.as_str(), "-b" | "--b" | "-build" | "--build"))
        {
            command_name = "tsbuild";
        }
        let mut w = "";
        if command_line_args
            .iter()
            .any(|arg| matches!(arg.as_str(), "-w" | "--w" | "-watch" | "--watch"))
        {
            w = "Watch";
        }
        format!("{}{}", command_name, w)
    }

    pub fn run(&self, scenario: &str) { ::tsox_core::fntrace::enter("run"); 
        let mut baseline_builder = String::new();
        let mut sys = super::m5b::new_test_sys(self, false);
        baseline_builder.push_str(&format!(
            "currentDirectory::{}\nuseCaseSensitiveFileNames::{}\nInput::\n",
            sys.current_directory(),
            sys.fs().use_case_sensitive_file_names(),
        ));
        sys.baseline_fs_with_diff(&mut BaselineWriter {
            buffer: &mut baseline_builder,
        });
        let mut result = self.execute_command(
            &sys,
            &mut baseline_builder,
            self.command_line_args.as_deref().unwrap_or(&[]),
        );
        sys.serialize_state(&mut baseline_builder);
        if result.watcher.is_some() && sys.mock_watch_backend.has_watches() {
            baseline_builder.push_str(&sys.mock_watch_backend.watch_state());
        }
        let mut unexpected_diff = String::new();
        unexpected_diff.push_str(&sys.baseline_programs(&mut baseline_builder, "Initial build"));

        for (index, do_edit) in self.edits.iter().enumerate() {
            sys.clear_output();
            let command_line_args = match &do_edit.command_line_args {
                Some(args) => args.clone(),
                None => self.command_line_args.clone().unwrap_or_default(),
            };
            baseline_builder
                .push_str(&format!("\n\nEdit [{}]:: {}\n", index, do_edit.caption));
            if let Some(edit) = &do_edit.edit {
                edit(&sys);
            }
            let changed_paths =
                fs_differ_changed_paths(sys.fs_differ.as_ref().unwrap());
            sys.baseline_fs_with_diff(&mut BaselineWriter {
                buffer: &mut baseline_builder,
            });

            if result.watcher.is_none() {
                self.execute_command(&sys, &mut baseline_builder, &command_line_args);
            } else {
                sys.mock_watch_backend.send_changed_paths(&changed_paths);
                result.watcher.as_mut().unwrap().do_cycle();
            }
            sys.serialize_state(&mut baseline_builder);
            if result.watcher.is_some() && sys.mock_watch_backend.has_watches() {
                baseline_builder.push_str(&sys.mock_watch_backend.watch_state());
            }
            unexpected_diff.push_str(&sys.baseline_programs(
                &mut baseline_builder,
                &format!("Edit [{}]:: {}\n", index, do_edit.caption),
            ));

            let mut non_incremental_sys = super::m5b::new_test_sys(self, true);
            for edit in self.edits.iter().take(index + 1) {
                if let Some(edit_fn) = &edit.edit {
                    edit_fn(&non_incremental_sys);
                }
            }
            crate::execute::command_line(&non_incremental_sys, &command_line_args);

            let diff = get_diff_for_incremental(&sys, &non_incremental_sys);
            if !diff.is_empty() {
                let explanation = if do_edit.expected_diff.is_empty() {
                    "!!! Unexpected diff, please review and either fix or write explanation as expectedDiff !!!"
                } else {
                    &do_edit.expected_diff
                };
                baseline_builder.push_str(&format!("\n\nDiff:: {}\n", explanation));
                baseline_builder.push_str(&diff);
                if do_edit.expected_diff.is_empty() {
                    unexpected_diff.push_str(&format!(
                        "Edit [{}]:: {}\n!!! Unexpected diff, please review and either fix or write explanation as expectedDiff !!!\n{}\n",
                        index, do_edit.caption, diff
                    ));
                }
            } else if !do_edit.expected_diff.is_empty() {
                baseline_builder.push_str(&format!(
                    "\n\nDiff:: {} !!! Diff not found but explanation present, please review and remove the explanation !!!\n",
                    do_edit.expected_diff
                ));
                unexpected_diff.push_str(&format!(
                    "Edit [{}]:: {}\n!!! Diff not found but explanation present, please review and remove the explanation !!!\n",
                    index, do_edit.caption
                ));
            }
        }
        let _ = baseline::run(
            &format!("{}.js", self.sub_scenario.replace(' ', "-")),
            &baseline_builder,
            &baseline::BaselineOptions::new(&format!(
                "{}/{}",
                self.get_baseline_sub_folder(),
                scenario
            )),
        );
        if !unexpected_diff.is_empty() {
            panic!(
                "Test {} has unexpected diff {} with incremental build, please review the baseline file",
                self.sub_scenario, unexpected_diff
            );
        }
    }
}

pub fn get_diff_for_incremental(incremental_sys: &TestSys, non_incremental_sys: &TestSys) -> String { ::tsox_core::fntrace::enter("get_diff_for_incremental"); 
    let mut diff_builder = String::new();

    let mut non_incremental_outputs = non_incremental_sys.fs.written_files_keys();
    non_incremental_outputs.sort();
    for non_incremental_output in &non_incremental_outputs {
        if tspath::file_extension_is(non_incremental_output, EXTENSION_TS_BUILD_INFO)
            || non_incremental_output.ends_with(".readable.baseline.txt")
        {
            if !incremental_sys
                .fs_from_file_map()
                .file_exists(non_incremental_output)
            {
                diff_builder.push_str(&baseline::diff_text(
                    &format!("nonIncremental {}", non_incremental_output),
                    &format!("incremental {}", non_incremental_output),
                    "Exists",
                    "",
                ));
                diff_builder.push('\n');
            }
        } else {
            let non_incremental_text = non_incremental_sys
                .fs_from_file_map()
                .read_file(non_incremental_output)
                .unwrap_or_else(|| panic!("Written file not found {}", non_incremental_output));
            let incremental_text = incremental_sys
                .fs_from_file_map()
                .read_file(non_incremental_output);
            if incremental_text.as_deref() != Some(non_incremental_text.as_str()) {
                diff_builder.push_str(&baseline::diff_text(
                    &format!("nonIncremental {}", non_incremental_output),
                    &format!("incremental {}", non_incremental_output),
                    &non_incremental_text,
                    incremental_text.as_deref().unwrap_or(""),
                ));
                diff_builder.push('\n');
            }
        }
    }

    let incremental_output = incremental_sys.get_output(true);
    let non_incremental_output = non_incremental_sys.get_output(true);
    if incremental_output != non_incremental_output {
        diff_builder.push_str(&baseline::diff_text(
            "nonIncremental.output.txt",
            "incremental.output.txt",
            &non_incremental_output,
            &incremental_output,
        ));
    }
    diff_builder
}

impl TestSys {
    pub fn error_writer(&self) -> Arc<Mutex<Vec<u8>>> { ::tsox_core::fntrace::enter("error_writer"); 
        Arc::clone(&self.current_write)
    }
}

struct SharedBufferWriter {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl std::io::Write for SharedBufferWriter {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> { ::tsox_core::fntrace::enter("write"); 
        self.buf.lock().unwrap().extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> std::io::Result<()> { ::tsox_core::fntrace::enter("flush"); 
        Ok(())
    }
}

impl System for TestSys {
    fn writer(&self) -> Box<dyn std::io::Write + Send> { ::tsox_core::fntrace::enter("writer"); 
        Box::new(SharedBufferWriter {
            buf: Arc::clone(&self.current_write),
        })
    }

    fn fs(&self) -> Arc<dyn FS> { ::tsox_core::fntrace::enter("fs"); 
        TestSys::fs(self)
    }

    fn default_library_path(&self) -> &str { ::tsox_core::fntrace::enter("default_library_path"); 
        TestSys::default_library_path(self)
    }

    fn current_directory(&self) -> &str { ::tsox_core::fntrace::enter("current_directory"); 
        TestSys::current_directory(self)
    }

    fn write_output_is_tty(&self) -> bool { ::tsox_core::fntrace::enter("write_output_is_tty"); 
        self.output_is_tty
    }

    fn width_of_terminal(&self) -> usize { ::tsox_core::fntrace::enter("width_of_terminal"); 
        match self.environment_variable("TS_TEST_TERMINAL_WIDTH") {
            Some(width) if !width.is_empty() => width.parse().unwrap_or(0),
            _ => 0,
        }
    }

    fn environment_variable(&self, name: &str) -> Option<String> { ::tsox_core::fntrace::enter("environment_variable"); 
        self.env.get(name).cloned()
    }
}
