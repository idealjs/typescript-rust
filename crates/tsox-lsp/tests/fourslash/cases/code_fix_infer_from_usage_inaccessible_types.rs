use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_infer_from_usage_inaccessible_types() {
    let content = r#"// @strict: false
// @noImplicitAny: true
function f1(a) { a; }
function h1() {
    class C { p: number };
    f1({ ofTypeC: new C() });
}

function f2(a) { a; }
function h2() {
    interface I { a: number }
    var i: I = {a : 1};
    f2(i);
    f2(2);
    f2(false);
}
"#;
    let _s = Session::new_for_test("codeFixInferFromUsageInaccessibleTypes", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
