use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_string_property_names2() {
    let content = r#"type Props = {
  foo: boolean;
}

let { foo }: Props = null as any;
foo;

let asd: Props = { "foo"/**/: true }; // rename foo here"#;
    let mut s = Session::new_for_test("renameStringPropertyNames2", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
