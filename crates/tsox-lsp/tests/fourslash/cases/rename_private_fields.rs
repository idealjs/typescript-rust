use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyRenameSucceeded"]
#[test]
fn rename_private_fields() {
    let content = r#"class Foo {
   [|/**/#foo|] = 1;

   getFoo() {
       return this.#foo;
   }
}"#;
    let mut s = Session::new_for_test("renamePrivateFields", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyRenameSucceeded"); // f.VerifyRenameSucceeded(t, nil /*preferences*/)
}
