use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn member_list_of_exported_class() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"namespace M {
  export class C { public pub = 0; private priv = 1; }
  export var V = 0;
}


var c = new M.C();

c./**/ // test on c."#;
    let mut s = Session::new_for_test("memberListOfExportedClass", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
