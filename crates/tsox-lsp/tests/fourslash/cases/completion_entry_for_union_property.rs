use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_entry_for_union_property() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"interface One {
    commonProperty: number;
    commonFunction(): number;
}

interface Two {
    commonProperty: string
    commonFunction(): number;
}

var x : One | Two;

x./**/"#;
    let mut s = Session::new_for_test("completionEntryForUnionProperty", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
