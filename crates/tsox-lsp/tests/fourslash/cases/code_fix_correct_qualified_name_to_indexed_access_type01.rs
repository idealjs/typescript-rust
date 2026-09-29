use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_correct_qualified_name_to_indexed_access_type01() {
    let content = r#"export interface Foo {
  bar: string;
}
export const x: [|Foo.bar|] = """#;
    let _s = Session::new_for_test("codeFixCorrectQualifiedNameToIndexedAccessType01", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `Foo["bar"]`, false, 0, 0)
}
