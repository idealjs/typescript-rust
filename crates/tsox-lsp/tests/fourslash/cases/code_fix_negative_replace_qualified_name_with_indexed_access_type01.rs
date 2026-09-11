use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_negative_replace_qualified_name_with_indexed_access_type01() {
    let content = r#"namespace Container {
    export interface Foo {
        bar: string;
    }
}
const x: [|Container.Foo.bar|] = """#;
    let mut s = Session::new_for_test("codeFixNegativeReplaceQualifiedNameWithIndexedAccessType01", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
