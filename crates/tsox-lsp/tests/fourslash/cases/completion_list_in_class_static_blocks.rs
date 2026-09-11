use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_class_static_blocks() {
    let content = r#"// @lib: es5
// @target: esnext
class Foo {
    static #a = 1;
    static a() {
        this./*1*/
    }
    static b() {
        Foo./*2*/
    }
    static {
        this./*3*/
    }
    static {
        Foo./*4*/
    }
}"#;
    let mut s = Session::new_for_test("completionListInClassStaticBlocks", content);
    // TODO: f.VerifyCompletions(t, []string{"1", "2", "3", "4"}, &fourslash.CompletionsExpectedList{
}
