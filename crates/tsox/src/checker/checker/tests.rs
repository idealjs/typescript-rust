use super::*;

#[test]
fn link_store_basic() {
    let store: LinkStore<Node, NodeLinks> = LinkStore::new();

    assert!(store.data.is_empty());
}

#[test]
fn ternary_and_or() {
    assert_eq!(Ternary::True.and(Ternary::False), Ternary::False);
    assert_eq!(Ternary::True.or(Ternary::False), Ternary::True);
}

#[test]
fn get_symbol_at_location() {
    use crate::astnav::get_token_at_position;
    use crate::bundled::{BundledFS, lib_path};
    use crate::compiler::{CompilerHostImpl, Program, ProgramOptions};
    use crate::tsoptions::ParsedCommandLine;
    use crate::vfs::InMemoryFS;

    let content = "interface Foo {\n  bar: string;\n}\ndeclare const foo: Foo;\nfoo.bar;";
    let inner = Arc::new(InMemoryFS::new());
    inner.insert_file("/foo.ts", content);
    inner.insert_file(
        "/tsconfig.json",
        "{\n  \"compilerOptions\": {},\n  \"files\": [\"foo.ts\"]\n}",
    );
    let fs = Arc::new(BundledFS::new(inner));

    let parsed = ParsedCommandLine {
        file_names: vec!["/foo.ts".to_string()],
        ..Default::default()
    };
    let host = Arc::new(CompilerHostImpl::new(fs, "/".to_string(), lib_path()));
    let program = Arc::new(Program::new(ProgramOptions {
        config: parsed,
        host,
    }));

    let mut checker = program.build_checker();
    let file = program.get_source_file("/foo.ts").expect("foo.ts");
    checker.check_source_file(&file);

    let interface_name = get_token_at_position(&file.node, 10).expect("interface name");
    let sym = checker.get_symbol_at_location(&interface_name);
    assert!(sym.is_some(), "Expected symbol for interface name 'Foo'");

    let var_name = get_token_at_position(&file.node, 47).expect("variable name");
    let sym = checker.get_symbol_at_location(&var_name);
    assert!(sym.is_some(), "Expected symbol for variable name 'foo'");

    let prop_access = get_token_at_position(&file.node, 60).expect("property access");
    let sym = checker.get_symbol_at_location(&prop_access);
    assert!(
        sym.is_some(),
        "Expected symbol for property access 'foo.bar'"
    );
}

#[test]
fn tracer_push_preserves_end_arg_mutations() {
    use crate::tracing::{Phase, TraceArg, Tracer};

    let tr = Tracer::new();

    let args = vec![
        ("checkerId".to_string(), TraceArg::Int(7)),
        ("id".to_string(), TraceArg::Int(1)),
    ];
    let outer = tr.push(Phase::CheckTypes, "getVariancesWorker", args.clone());

    assert_eq!(args.len(), 2);

    let inner_args = vec![("checkerId".to_string(), TraceArg::Int(7))];
    let inner = tr.push(Phase::Check, "checkSourceFile", inner_args);

    drop(inner);
    drop(outer);

    let events = tr.take_events();

    let outer_begin = events
        .iter()
        .find(|e| e.ph == "B" && e.name == "getVariancesWorker")
        .expect("outer begin event");
    let outer_end = events
        .iter()
        .find(|e| e.ph == "E" && e.name == "getVariancesWorker")
        .expect("outer end event");

    assert_eq!(outer_begin.cat, "checkTypes");
    assert_eq!(
        outer_begin.args,
        vec![
            ("checkerId".to_string(), TraceArg::Int(7)),
            ("id".to_string(), TraceArg::Int(1)),
        ]
    );

    assert_eq!(outer_end.args, outer_begin.args);

    assert_eq!(outer_begin.tid, outer_end.tid);

    let inner_begin = events
        .iter()
        .find(|e| e.ph == "B" && e.name == "checkSourceFile")
        .expect("inner begin event");
    assert_eq!(inner_begin.tid, outer_begin.tid);

    let outer_begin_idx = events
        .iter()
        .position(|e| std::ptr::eq(e, outer_begin))
        .unwrap();
    let inner_begin_idx = events
        .iter()
        .position(|e| std::ptr::eq(e, inner_begin))
        .unwrap();
    let inner_end_idx = events
        .iter()
        .position(|e| e.ph == "E" && e.name == "checkSourceFile")
        .unwrap();
    let outer_end_idx = events
        .iter()
        .position(|e| std::ptr::eq(e, outer_end))
        .unwrap();
    assert!(outer_begin_idx < inner_begin_idx);
    assert!(inner_begin_idx < inner_end_idx);
    assert!(inner_end_idx < outer_end_idx);
}
