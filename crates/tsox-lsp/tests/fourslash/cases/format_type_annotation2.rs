use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_type_annotation2() {
    let content = r#"function foo(x : number, y ?: string) : number {}
interface Foo {
    x : number;
    y ?: number;
}"#;
    let mut s = Session::new_for_test("formatTypeAnnotation2", content);
    // TODO: f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"function foo(x: number, y?: string): number { }
interface Foo {
    x: number;
    y?: number;
}"#);
}
