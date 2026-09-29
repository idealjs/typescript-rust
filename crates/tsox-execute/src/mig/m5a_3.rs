#![allow(unused_imports, dead_code)]

use std::io::Write;

use super::m5a::{CommandLineTesting, ContentMapperTimings};
use super::m5a_2::{CompileTimes, EmitInput};

struct TableRow {
    name: String,
    value: String,
}

#[derive(Default)]
struct Table {
    rows: Vec<TableRow>,
}

impl Table {
    fn add(&mut self, name: &str, value: impl std::fmt::Display) {
        self.rows.push(TableRow {
            name: name.to_string(),
            value: value.to_string(),
        });
    }

    fn add_duration(&mut self, name: &str, value: f64) {
        self.add(name, format_duration(value));
    }

    fn print(&self, w: &mut dyn Write) {
        let name_width = self.rows.iter().map(|r| r.name.len()).max().unwrap_or(0);
        let value_width = self.rows.iter().map(|r| r.value.len()).max().unwrap_or(0);
        for r in &self.rows {
            let _ = writeln!(
                w,
                "{:<width1$} {:>width2$}",
                format!("{}:", r.name),
                r.value,
                width1 = name_width + 1,
                width2 = value_width
            );
        }
    }
}

pub fn format_duration(d: f64) -> String {
    format!("{:.3}s", d)
}

pub fn identifier_count(program: &EmitProgram) -> usize {
    program
        .get_source_files()
        .iter()
        .map(|f| f.identifier_count)
        .sum()
}

#[derive(Default)]
pub struct Statistics {
    is_aggregate: bool,
    pub projects: usize,
    pub projects_built: usize,
    pub timestamp_updates: usize,
    files: usize,
    lines: usize,
    identifiers: usize,
    symbols: usize,
    types: usize,
    instantiations: usize,
    memory_used: u64,
    memory_allocs: u64,
    compile_times: CompileTimes,
}

pub fn statistics_from_program(
    input: &EmitInput,
    memory_used: u64,
    memory_allocs: u64,
) -> Statistics {
    Statistics {
        is_aggregate: false,
        projects: 0,
        projects_built: 0,
        timestamp_updates: 0,
        files: input.program.get_source_files().len(),
        lines: input.program.line_count(),
        identifiers: input.program.identifier_count(),
        symbols: input.program.symbol_count(),
        types: input.program.type_count(),
        instantiations: input.program.instantiation_count(),
        memory_used,
        memory_allocs,
        compile_times: input.compile_times.clone(),
    }
}

impl Statistics {
    pub fn report(&self, w: &mut dyn Write, testing: Option<&dyn CommandLineTesting>) {
        if let Some(testing) = testing {
            testing.on_statistics_start(w);
        }
        let mut table = Table::default();
        let prefix = if self.is_aggregate {
            table.add("Projects in scope", self.projects);
            table.add("Projects built", self.projects_built);
            table.add("Timestamps only updates", self.timestamp_updates);
            "Aggregate "
        } else {
            ""
        };
        table.add(&format!("{prefix}Files"), self.files);
        table.add(&format!("{prefix}Lines"), self.lines);
        table.add(&format!("{prefix}Identifiers"), self.identifiers);
        table.add(&format!("{prefix}Symbols"), self.symbols);
        table.add(&format!("{prefix}Types"), self.types);
        table.add(&format!("{prefix}Instantiations"), self.instantiations);
        table.add(&format!("{prefix}Memory used"), format!("{}K", self.memory_used / 1024));
        table.add(&format!("{prefix}Memory allocs"), self.memory_allocs);
        if self.compile_times.config_time != 0.0 {
            table.add_duration(&format!("{prefix}Config time"), self.compile_times.config_time);
        }
        if self.compile_times.build_info_read_time != 0.0 {
            table.add_duration(
                &format!("{prefix}BuildInfo read time"),
                self.compile_times.build_info_read_time,
            );
        }
        table.add_duration(&format!("{prefix}Parse time"), self.compile_times.parse_time);
        if self.compile_times.bind_time != 0.0 {
            table.add_duration(&format!("{prefix}Bind time"), self.compile_times.bind_time);
        }
        if self.compile_times.check_time != 0.0 {
            table.add_duration(&format!("{prefix}Check time"), self.compile_times.check_time);
        }
        if self.compile_times.emit_time != 0.0 {
            table.add_duration(&format!("{prefix}Emit time"), self.compile_times.emit_time);
        }
        if self.compile_times.changes_compute_time != 0.0 {
            table.add_duration(
                &format!("{prefix}Changes compute time"),
                self.compile_times.changes_compute_time,
            );
        }
        self.add_content_mapper_statistics(&mut table, prefix);
        table.add_duration(&format!("{prefix}Total time"), self.compile_times.total_time);
        table.print(w);
        if let Some(testing) = testing {
            testing.on_statistics_end(w);
        }
    }

    fn add_content_mapper_statistics(&self, table: &mut Table, prefix: &str) {
        let timings = &self.compile_times.content_mapper_times;
        if timings.request_wait != 0.0 {
            table.add_duration(
                &format!("{prefix}Content mapper request wait time"),
                timings.request_wait,
            );
        }
        for (identity, mapper) in &timings.mappers {
            let initialization_count = mapper.spawn.0;
            if initialization_count != 0 {
                table.add_duration(
                    &format!("{prefix}{identity} initialization time"),
                    mapper.spawn.1 + mapper.initialize.1,
                );
            }
            if mapper.transform.0 != 0 {
                table.add_duration(
                    &format!("{prefix}{identity} transform time"),
                    mapper.transform.1,
                );
            }
            if mapper.open_project.0 != 0 {
                table.add_duration(
                    &format!("{prefix}{identity} openProject time"),
                    mapper.open_project.1,
                );
            }
            if mapper.close_project.0 != 0 {
                table.add_duration(
                    &format!("{prefix}{identity} closeProject time"),
                    mapper.close_project.1,
                );
            }
        }
    }

    pub fn aggregate(&mut self, stat: &Statistics) {
        self.is_aggregate = true;
        self.files += stat.files;
        self.lines += stat.lines;
        self.identifiers += stat.identifiers;
        self.symbols += stat.symbols;
        self.types += stat.types;
        self.instantiations += stat.instantiations;
        self.memory_used += stat.memory_used;
        self.memory_allocs += stat.memory_allocs;
        self.compile_times.config_time += stat.compile_times.config_time;
        self.compile_times.build_info_read_time += stat.compile_times.build_info_read_time;
        self.compile_times.parse_time += stat.compile_times.parse_time;
        self.compile_times.bind_time += stat.compile_times.bind_time;
        self.compile_times.check_time += stat.compile_times.check_time;
        self.compile_times.emit_time += stat.compile_times.emit_time;
        self.compile_times.changes_compute_time += stat.compile_times.changes_compute_time;
    }

    pub fn set_total_time(&mut self, total_time: f64) {
        self.compile_times.total_time = total_time;
    }
}
pub struct EmitProgram;

impl EmitProgram {
    pub fn options(&self) -> tsox_core::core::compiler_options::CompilerOptions {
        Default::default()
    }
    pub fn line_count(&self) -> usize {
        0
    }
    pub fn identifier_count(&self) -> usize {
        0
    }
    pub fn symbol_count(&self) -> usize {
        0
    }
    pub fn type_count(&self) -> usize {
        0
    }
    pub fn instantiation_count(&self) -> usize {
        0
    }
    pub fn get_source_files(&self) -> Vec<SourceFileRef> {
        Vec::new()
    }
    pub fn current_directory(&self) -> &str {
        "."
    }
    pub fn explain_files(&self, w: &mut dyn Write, locale: &tsox_core::locale::Locale) {
        let _ = (w, locale);
    }
}


pub struct SourceFileRef {
    pub file_name: String,
    pub identifier_count: usize,
}

