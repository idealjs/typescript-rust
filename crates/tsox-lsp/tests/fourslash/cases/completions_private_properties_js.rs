use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_private_properties_js() {
    let content = r#"// @allowJs: true
// @Filename: a.d.ts
declare namespace A {
    class Foo {
        constructor();

        private m1(): void;
        protected m2(): void;

        m3(): void;
    }
}
// @filename: b.js
let foo = new A.Foo();
foo./**/"#;
    let mut s = Session::new_for_test("completionsPrivateProperties_Js", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{""}, &fourslash.CompletionsExpectedList{
}
