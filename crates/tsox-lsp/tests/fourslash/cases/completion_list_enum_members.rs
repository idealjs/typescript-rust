use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_enum_members() {
    let content = r#"enum Foo {
    bar,
    baz
}

var v = Foo./*valueReference*/ba;
var t :Foo./*typeReference*/ba;
Foo.bar./*enumValueReference*/;"#;
    let mut s = Session::new_for_test("completionListEnumMembers", content);
    // TODO: f.VerifyCompletions(t, []string{"valueReference", "typeReference"}, &fourslash.CompletionsExpectedLi
    fourslash::verify_completions_unsorted_at(&mut s, Some("enumValueReference"), &["toString", "toFixed", "toExponential", "toPrecision", "valueOf", "toLocaleString"]);
}
