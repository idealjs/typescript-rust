use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_object_binding_pattern03() {
    let content = r#"interface I {
    property1: number;
    property2: string;
}

var foo: I;
var { property1: /**/ } = foo;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", nil)
}
