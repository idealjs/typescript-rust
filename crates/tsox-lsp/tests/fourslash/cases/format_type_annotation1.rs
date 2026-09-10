use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: opts207 := f.GetOptions()"]
#[test]
fn format_type_annotation1() {
    let content = r#"function foo(x: number, y?: string): number {}
interface Foo {
    x: number;
    y?: number;
}"#;
    let mut s = Session::new_for_test("formatTypeAnnotation1", content);
    // TODO: opts207 := f.GetOptions()
    // TODO: opts207.FormatCodeSettings.InsertSpaceBeforeTypeAnnotation = core.TSTrue
    fourslash::unsupported("Configure"); // f.Configure(t, opts207)
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"function foo(x : number, y ?: string) : number { }
interface Foo {
    x : number;
    y ?: number;
}"#);
}
