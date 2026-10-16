use std::sync::Arc;

use tsox_checker::bundled::{BundledFS, lib_path};
use tsox_compile::compiler::{CompilerHostImpl, Program, ProgramOptions};
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_frontend::ast::Diagnostic;
use tsox_tsoptions::tsoptions::ParsedCommandLine;
use tsox_tsoptions::vfs::InMemoryFS;

use crate::case_parser::TestUnit;
use crate::program_setup::{mount_test_lib_fixtures, unit_abs_path};

pub fn build_and_check(
    options: &CompilerOptions,
    units: &[TestUnit],
    no_implicit_references: bool,
    tsconfig_file_names: Option<&[String]>,
    symlinks: &[(String, String)],
) -> (Vec<Diagnostic>, usize, Option<String>) {
    let fs = {
        let mount_phase = crate::probe::enter(crate::probe::Slot::Mount);
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");

        mount_test_lib_fixtures(&fs);
        fs
    };

    let fsbuild_phase = crate::probe::enter(crate::probe::Slot::FsBuild);
    let config_roots: Option<Vec<String>> =
        tsconfig_file_names.map(|names| names.to_vec());
    let refs_only = config_roots.is_none()
        && (no_implicit_references
            || units.last().is_some_and(|u| {
                u.content.contains("require(") || u.content.contains("reference path")
            }));
    let rooted: Vec<&TestUnit> = match &config_roots {
        Some(names) => units
            .iter()
            .filter(|u| names.iter().any(|n| n == &unit_abs_path(&u.name)))
            .collect(),
        None if refs_only => units.last().into_iter().collect(),
        None => units.iter().collect(),
    };

    let insert_unit = |unit: &TestUnit| {
        let abs = unit_abs_path(&unit.name);

        let mut parent = tsox_core::tspath::get_directory_path(&abs);
        while !parent.is_empty() {
            fs.insert_dir(&parent);
            let next = tsox_core::tspath::get_directory_path(&parent);
            if next == parent {
                break;
            }
            parent = next;
        }
        fs.insert_file(&abs, &unit.content);
    };

    // Go harnessutil.CompileFilesEx：inputFiles（root）先写入 testfs，otherFiles 后写入；
    // 同名 unit 并存时（重复 @filename 指令）otherFiles 覆盖 root，编译的是覆盖后的内容
    for unit in &rooted {
        insert_unit(unit);
    }
    for unit in units {
        if !rooted.iter().any(|r| std::ptr::eq(*r, unit)) {
            insert_unit(unit);
        }
    }

    // Go compiler_runner.go newCompilerTest：toBeCompiled = 全量 rooted unit
    //（refs_only 时 lastUnit），无扩展过滤；.js 是否参与编译由编译器按
    // allowJs/checkJs 决定，不由 harness 预筛。
    // Go harnessutil.CompileFilesEx（harnessutil.go:125-133）：programFileNames
    // 跳过 .json/.tsbuildinfo 扩展的 inputFile，两类 unit 仅落 testfs 供
    // 模块解析/增量 buildinfo 读取，不作为编译根
    let file_names: Vec<String> = rooted
        .iter()
        .map(|unit| unit_abs_path(&unit.name))
        .filter(|abs| {
            !tsox_core::tspath::file_extension_is(abs, tsox_core::tspath::EXTENSION_JSON)
                && !tsox_core::tspath::file_extension_is(
                    abs,
                    tsox_core::tspath::EXTENSION_TS_BUILD_INFO,
                )
        })
        .collect();

    // Go harnessutil：@symlink/@link 挂为指向真实 unit 的符号链接（InMemoryFS
    // 原生支持，resolver 列目录与读取均穿透；program 侧按 realpath 规范化
    // 回真实路径，同一文件不会重复加载）
    for (link, unit_name) in symlinks {
        let target_abs = if tsox_core::tspath::is_rooted_disk_path(unit_name) {
            tsox_core::tspath::normalize_path(unit_name)
        } else {
            tsox_core::tspath::normalize_path(&format!("/proj/{unit_name}"))
        };
        let link_abs = if tsox_core::tspath::is_rooted_disk_path(link) {
            tsox_core::tspath::normalize_path(link)
        } else {
            tsox_core::tspath::normalize_path(&format!("/proj/{link}"))
        };
        let target_exists = units.iter().any(|u| {
            let abs = if tsox_core::tspath::is_rooted_disk_path(&u.name) {
                u.name.clone()
            } else {
                format!("/proj/{}", u.name)
            };
            abs == target_abs || abs.starts_with(&format!("{target_abs}/"))
        });
        if !target_exists {
            continue;
        }
        let mut link_parent = tsox_core::tspath::get_directory_path(&link_abs);
        while !link_parent.is_empty() {
            fs.insert_dir(&link_parent);
            let next = tsox_core::tspath::get_directory_path(&link_parent);
            if next == link_parent {
                break;
            }
            link_parent = next;
        }
        fs.create_symlink(&link_abs, &target_abs);
    }

    let bf = Arc::new(BundledFS::new(fs.clone()));

    let all_rooted = units.iter().any(|u| {
        tsox_core::tspath::is_rooted_disk_path(&u.name)
    });
    let cwd = if all_rooted && !units.is_empty() {
        "/.src"
    } else {
        "/proj"
    };
    let host = Arc::new(CompilerHostImpl::new(bf, cwd.to_string(), lib_path()));

    let mut config = ParsedCommandLine::default();
    config.compiler_options = options.clone();
    config.file_names = file_names;

    drop(fsbuild_phase);

    let t_program = std::time::Instant::now();
    let program = Arc::new(Program::new(ProgramOptions {
        config,
        host,
        use_source_of_project_reference: false,
        single_threaded: Default::default(),
        create_checker_pool: None,
        typings_location: String::new(),
        project_name: String::new(),
        tracing: None,
        skip_module_resolution: false,
    }));
    let program_elapsed = t_program.elapsed();
    crate::probe::add(crate::probe::Slot::Program, program_elapsed);
    if std::env::var_os("TSOX_DEBUG_FILES").is_some() {
        eprintln!(
            "[files] refs_only={} roots={}",
            refs_only,
            units
                .iter()
                .filter(|u| rooted.iter().any(|r| std::ptr::eq(*r, *u)))
                .map(|u| u.name.as_str())
                .collect::<Vec<_>>()
                .join(",")
        );
        eprintln!(
            "[files] {}",
            program
                .source_files()
                .iter()
                .map(|f| f.file_name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    let t_check = std::time::Instant::now();
    let mut all = Vec::new();
    for d in program.diagnostics() {
        all.push((**d).clone());
    }
    let program_len = all.len();
    all.extend(
        program
            .get_program_diagnostics()
            .iter()
            .map(|d| (**d).clone()),
    );
    all.extend(program.get_semantic_diagnostics());
    let check_elapsed = t_check.elapsed();
    crate::probe::add(crate::probe::Slot::Check, check_elapsed);

    let mut types_text: Option<String> = None;
    if std::env::var_os("TSOX_TYPES_EMIT_DIR").is_some() {
        let types_phase = crate::probe::enter(crate::probe::Slot::Types);
        let mut units_sf: Vec<(String, String, Arc<tsox_frontend::ast::SourceFile>)> = Vec::new();
        // Go compiler_runner.go:531-537：类型基线按 allFiles = toBeCompiled ++
        // otherFiles 迭代（toBeCompiled=上列 rooted，末 unit 含 require/引用时
        // 为 lastUnit，compiler_runner.go:325-341）；纯顺序，节内内容不受影响
        let mut ordered_units: Vec<&TestUnit> = rooted.clone();
        ordered_units.extend(
            units
                .iter()
                .filter(|u| !rooted.iter().any(|r| std::ptr::eq(*r, *u))),
        );
        for u in ordered_units {
            let abs = if tsox_core::tspath::is_rooted_disk_path(&u.name) {
                tsox_core::tspath::normalize_path(&u.name)
            } else {
                tsox_core::tspath::normalize_path(&format!("/proj/{}", u.name))
            };
            if let Some(sf) = program.get_source_file_by_path(&abs) {
                // harness 专项（波收尾主 agent 授权，限 header 归一）：Go
                // iterateBaseline 的 unit header 写 UnitName 后经
                // removeTestPathPrefixes（tsc/internal/testutil/tsbaseline/
                // util.go:25-33）剥测试根前缀（type_symbol_baseline.go:205-211，
                // Go 虚拟根 /.src/）。本仓虚拟根为 /proj/：GetNormalizedAbsolutePath
                // 结果统一剥 /proj/ 作 header 名，内容无关、对全部 unit 一致
                let header_name = abs.replace("/proj/", "");
                units_sf.push((header_name, u.content.clone(), sf));
            }
        }
        let header = std::env::var("TSOX_TYPES_HEADER").unwrap_or_default();
        if !units_sf.is_empty() && !header.is_empty() {
            types_text = Some(crate::types_baseline::generate(
                &program, &header, &units_sf,
            ));
        }
    }
    if let Some(path) = std::env::var_os("TSOX_PROBE_PHASES") {
        use std::io::Write as _;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();
        writeln!(
            f,
            "program_build={:?} check={:?}",
            program_elapsed, check_elapsed
        )
        .ok();
    }
    (all, program_len, types_text)
}
