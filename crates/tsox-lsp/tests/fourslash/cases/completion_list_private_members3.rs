use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_private_members3() {
    let content = r#"class Other {
    public p;
    protected p2
    private p3;
}

class Self {
    private other: Other;

    method() {
        this.other./*1*/;

        this.other.p/*2*/;

        this.other.p/*3*/.toString();
    }
}"#;
    let mut s = Session::new_for_test("completionListPrivateMembers3", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1", "2", "3"}, &fourslash.CompletionsExpectedList{
}
