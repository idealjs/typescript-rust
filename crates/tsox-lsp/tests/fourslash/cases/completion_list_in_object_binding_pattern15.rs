use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_object_binding_pattern15() {
    let content = r#"class Foo {
    private   xxx1 = 1;
    protected xxx2 = 2;
    public    xxx3 = 3;
    private   static xxx4 = 4;
    protected static xxx5 = 5;
    public    static xxx6 = 6;
    foo() {
        const { /*1*/ } = this;
        const { /*2*/ } = Foo;
    }
}

const { /*3*/ } = new Foo();
const { /*4*/ } = Foo;"#;
    let mut s = Session::new_for_test("completionListInObjectBindingPattern15", content);
    fourslash::verify_completions_unsorted_at(&mut s, Some("1"), &["xxx1", "xxx2", "xxx3", "foo"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("2"), &["prototype", "xxx4", "xxx5", "xxx6"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("3"), &["xxx3", "foo"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("4"), &["prototype", "xxx6"]);
}
