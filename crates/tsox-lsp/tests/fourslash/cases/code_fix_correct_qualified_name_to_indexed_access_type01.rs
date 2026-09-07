use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_correct_qualified_name_to_indexed_access_type01() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"export interface Foo {
  bar: string;
}
export const x: [|Foo.bar|] = """#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `Foo["bar"]`, false, 0, 0)
}
