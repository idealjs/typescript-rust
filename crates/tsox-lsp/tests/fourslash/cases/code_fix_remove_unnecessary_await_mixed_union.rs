use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_remove_unnecessary_await_mixed_union() {
    let content = r#"// @target: esnext
async function fn1(a: Promise<void> | void) {
  await a;
}

async function fn2<T extends Promise<void> | void>(a: T) {
  await a;
}"#;
    let _s = Session::new_for_test("codeFixRemoveUnnecessaryAwait_mixedUnion", content);
    // TODO: f.VerifySuggestionDiagnostics(t, nil)
}
