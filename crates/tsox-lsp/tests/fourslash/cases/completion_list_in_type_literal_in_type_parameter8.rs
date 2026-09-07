use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_type_literal_in_type_parameter8() {
    let content = r#"interface Foo {
    one: string;
    two: {
        three: {
            four: number;
            five: string;
        }
    }
}

interface Bar<T extends Foo> {
    foo: T;
}

var foobar: Bar<{
    two: {
        three: {
            five: string,
            /*4*/
        },
        /*0*/
    },
    /*1*/
}>;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
