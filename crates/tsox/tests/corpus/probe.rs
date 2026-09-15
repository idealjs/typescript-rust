//! 回归哨兵：空数组赋接口与 null 返回的诊断必须存在
//! （对应 Go arrayAssignmentTest1 基线前两条；曾因 shell 早退括号丢失被吞）。

use std::sync::Arc;
use tsox_checker::bundled::{lib_path, BundledFS};
use tsox_compile::compiler::{CompilerHost, CompilerHostImpl, Program, ProgramOptions};
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::tristate::Tristate;
use tsox_tsoptions::tsoptions::ParsedCommandLine;
use tsox_tsoptions::vfs::InMemoryFS;

#[test]
fn null_return_and_empty_array_diagnostics() {
    let content = r#"interface I1 { IM1(): void[] }
class C1 implements I1 {
    IM1(): void[] { return null; }
}
var i1_error: I1 = [];
"#;
    let inner = Arc::new(InMemoryFS::new());
    inner.insert_dir("/.src");
    inner.insert_file("/.src/probe.ts", content);
    let fs = Arc::new(BundledFS::new(inner));
    let mut options = CompilerOptions::default();
    options.skip_default_lib_check = Tristate::True;
    options.target = tsox_core::core::compiler_options::ScriptTarget::ES2015;
    let parsed = ParsedCommandLine {
        file_names: vec!["/.src/probe.ts".to_string()],
        compiler_options: options,
        ..Default::default()
    };
    let host: Arc<dyn CompilerHost> =
        Arc::new(CompilerHostImpl::new(fs, "/.src".to_string(), lib_path()));
    let program = Arc::new(Program::new(ProgramOptions {
        config: parsed,
        host,
    }));
    let checker = program.build_checker();
    let codes: Vec<i32> = checker
        .diagnostics
        .get_all()
        .iter()
        .filter(|d| {
            d.file
                .as_ref()
                .is_some_and(|f| f.file_name.ends_with("probe.ts"))
        })
        .map(|d| d.code)
        .collect();
    assert!(
        codes.contains(&2322),
        "null → void[] 返回位诊断缺失: {codes:?}"
    );
    assert!(
        codes.contains(&2741) || codes.contains(&2739),
        "[] → I1 缺属性诊断缺失: {codes:?}"
    );
}
