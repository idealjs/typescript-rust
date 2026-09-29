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
    fourslash::configure_format_settings(&mut s, &[("insert_space_before_type_annotation", "true")]);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"function foo(x : number, y ?: string) : number { }
interface Foo {
    x : number;
    y ?: number;
}"#);
}
