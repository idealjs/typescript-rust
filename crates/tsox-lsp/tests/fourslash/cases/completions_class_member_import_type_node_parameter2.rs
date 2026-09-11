use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_class_member_import_type_node_parameter2() {
    let content = r#"// @module: node18
// @FileName: /index.d.ts
export declare class Cls {
  method(
    param: import("./doesntexist.js").Foo,
  ): import("./doesntexist.js").Foo;
}

export declare class Derived extends Cls {
  /*1*/
}"#;
    let mut s = Session::new_for_test("completionsClassMemberImportTypeNodeParameter2", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
