use tsox_lsp::fourslash::Session;


#[test]
fn implementation01() {
    let content = r#"// @lib: es5
interface Fo/*1*/o {}
class /*2*/Bar implements Foo {}"#;
    let _s = Session::new_for_test("implementation01", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineGoToImplementation(t, "1")
}
