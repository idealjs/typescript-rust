use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_enum_values() {
    let content = r#"enum Colors {
    Red,
    Green
}

Colors./*enumVariable*/;

var x = Colors.Red;
x./*variableOfEnumType*/;

function foo(): Colors { return null; }
foo()./*callOfEnumReturnType*/"#;
    let mut s = Session::new_for_test("completionListEnumValues", content);
    fourslash::verify_completions_exact_at(&mut s, Some("enumVariable"), &["Green", "Red"]);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"variableOfEnumType", "callOfEnumReturnType"}, &fourslash.Completion
}
