use super::*;
use crate::bundled::lib_path;
use crate::compiler::{CompilerHost, CompilerHostImpl, Program, ProgramOptions};
use crate::tsoptions::ParsedCommandLine;
use crate::vfs::InMemoryFS;

pub(crate) fn build_program_and_checker(
    source: &str,
    lib_spec: &[&str],
) -> (Arc<Program>, Checker) {
    use crate::bundled::BundledFS;
    let inner = Arc::new(InMemoryFS::new());
    inner.insert_file("/proj/entry.ts", source);
    let fs = Arc::new(BundledFS::new(inner));
    let mut compiler_options = CompilerOptions::default();
    compiler_options.lib = lib_spec.iter().map(|s| s.to_string()).collect();
    let parsed = ParsedCommandLine {
        file_names: vec!["/proj/entry.ts".to_string()],
        compiler_options,
        ..Default::default()
    };
    let host: Arc<dyn CompilerHost> =
        Arc::new(CompilerHostImpl::new(fs, "/proj".to_string(), lib_path()));
    let program = Arc::new(Program::new(ProgramOptions {
        config: parsed,
        host,
    }));
    let tracer = Arc::new(Tracer::new());
    let checker = Checker::new(Arc::clone(&program) as _, tracer);
    (program, checker)
}

pub(crate) fn error_codes(checker: &Checker) -> Vec<i32> {
    checker
        .diagnostics
        .get_all()
        .iter()
        .filter(|d| {
            !d.file
                .as_ref()
                .is_some_and(|f| f.file_name.starts_with("bundled://"))
        })
        .map(|d| d.code)
        .collect()
}

#[test]
fn any_base_is_not_degradation() {
    let (program, mut checker) = convergence_tests::build_program_and_checker(
        "type AnyAlias = any;\n\
             interface I extends AnyAlias { x: number; }\n\
             declare const i: I;\n\
             const n: number = i.x;\n\
             const m: number = i.x;\n\
             const k: number = i.x;",
        &["es5"],
    );
    for file in program.source_files() {
        checker.check_source_file(file);
    }
    assert_eq!(error_codes(&checker), Vec::<i32>::new());

    let entry = program
        .get_source_file("/proj/entry.ts")
        .expect("entry file");
    let iface = match &entry.node.data {
        crate::ast::NodeData::SourceFile(d) => d
            .statements
            .iter()
            .find(|s| matches!(s.data, crate::ast::NodeData::InterfaceDeclaration(_)))
            .expect("interface I declared")
            .clone(),
        _ => unreachable!(),
    };
    let sym = program
        .symbol_map()
        .symbol_of(&iface)
        .expect("interface symbol")
        .clone();

    assert!(
        checker
            .type_alias_links
            .get(&sym)
            .is_some_and(|l| l.declared_type.is_some()),
        "an any base must not disable the declared-type cache"
    );
    assert!(
        !checker
            .heritage_retry_counts
            .contains_key(&(Arc::as_ptr(&sym) as *const Symbol as usize)),
        "an any base must not record degraded retries for the interface"
    );
}

#[test]
fn cyclic_base_interfaces_converge() {
    let source = "interface A extends B { a: number; }\n\
                      interface B extends A { b: string; }\n\
                      declare const v1: A; declare const v2: A;\n\
                      declare const v3: A; declare const v4: A;\n\
                      declare const v5: A; declare const v6: A;\n\
                      const n: number = v6.a;";
    let (program, mut checker) = convergence_tests::build_program_and_checker(source, &["es5"]);
    for file in program.source_files() {
        checker.check_source_file(file);
    }

    assert_eq!(
        error_codes(&checker),
        Vec::<i32>::new(),
        "own-member access through the cyclic interface must stay clean"
    );

    assert!(
        !checker.heritage_retry_counts.is_empty(),
        "cyclic bases must have recorded degraded retries"
    );
    assert!(
        checker
            .heritage_retry_counts
            .values()
            .any(|&c| c > HERITAGE_RETRY_LIMIT),
        "repeated references must cross the retry limit and be accepted"
    );
}

#[test]
fn subst_cache_respects_capacity() {
    let (program, mut checker) = convergence_tests::build_program_and_checker(
        "declare const a: string[]; declare const b: number[];\n\
             var x = a.concat(b); var y = b.concat(a);\n\
             var z = x.concat(y);",
        &["es5"],
    );
    checker.type_node_subst_cache_limit = 8;
    checker.instantiated_member_type_cache_limit = 8;
    for file in program.source_files() {
        checker.check_source_file(file);
    }
    assert!(
        checker.type_node_subst_cache.len() <= 8,
        "subst cache must stay within its cap, got {}",
        checker.type_node_subst_cache.len()
    );
    assert!(
        checker.instantiated_member_type_cache.len() <= 8,
        "member-type cache must stay within its cap, got {}",
        checker.instantiated_member_type_cache.len()
    );
}

#[test]
fn deep_class_chain_bounded() {
    let mut source = String::from("class C0 { m0: number = 0; }\n");
    for i in 1..=260 {
        source.push_str(&format!(
            "class C{i} extends C{} {{ m{i}: number = {i}; }}\n",
            i - 1
        ));
    }
    source.push_str("declare const c: C260;\nconst n: number = c.m260;");
    let (program, mut checker) = convergence_tests::build_program_and_checker(&source, &["es5"]);
    for file in program.source_files() {
        checker.check_source_file(file);
    }

    assert_eq!(
        error_codes(&checker),
        Vec::<i32>::new(),
        "own-member access on the leaf of a deep chain must stay clean"
    );
}
