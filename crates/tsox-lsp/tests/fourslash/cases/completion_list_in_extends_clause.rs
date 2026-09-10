use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_extends_clause() {
    let content = r#"// @lib: es5
interface IFoo {
    method();
}

class Foo {
    property: number;
    method() { }
    static staticMethod() { }
}
class test1 extends Foo./*1*/ {}
class test2 implements IFoo./*2*/ {}
interface test3 extends IFoo./*3*/ {}
interface test4 implements Foo./*4*/ {}"#;
    let mut s = Session::new_for_test("completionListInExtendsClause", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"2", "3", "4"}, nil)
}
