use tsox_lsp::fourslash::Session;


#[test]
fn inlay_hints_interactive_variable_types2() {
    let content = r#"const object = { foo: 1, bar: 2 }
const array = [1, 2]
const a = object;
const { foo, bar } = object;
const {} = object;
const b = array;
const [ first, second ] = array;
const [] = array;
declare function foo<T extends number>(t: T): T
const x = foo(1)"#;
    let _s = Session::new_for_test("inlayHintsInteractiveVariableTypes2", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
