use tsox_lsp::fourslash::{self, Session};


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
    // TODO: f.Configure(t, opts207)
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"function foo(x : number, y ?: string) : number { }
interface Foo {
    x : number;
    y ?: number;
}"#);
}
