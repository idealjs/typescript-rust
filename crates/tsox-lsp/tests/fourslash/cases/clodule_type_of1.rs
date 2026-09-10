use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn clodule_type_of1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @strict: false
class C<T> {
    static foo(x: number) { }
    x: T;
}

namespace C {
    export function f(x: typeof C) {
        x./*1*/
        var /*3*/r = new /*2*/x<number>();
        var /*5*/r2 = r./*4*/
        return typeof r;
    }
}"#;
    let mut s = Session::new_for_test("cloduleTypeOf1", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "foo(1);");
    fourslash::verify_completions_include_exclude_at(&mut s, Some("2"), &["x"], &[]);
    fourslash::verify_quick_info_at(&mut s, "3", "(local var) r: C<number>", "");
    fourslash::verify_completions_include_exclude_at(&mut s, Some("4"), &["x"], &[]);
    fourslash::insert(&mut s, "x;");
    fourslash::verify_quick_info_at(&mut s, "5", "(local var) r2: number", "");
    fourslash::verify_no_errors(&mut s, );
}
