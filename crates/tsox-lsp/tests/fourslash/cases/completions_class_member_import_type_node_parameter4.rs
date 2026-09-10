use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_class_member_import_type_node_parameter4() {
    let content = r#"// @module: node18
// @FileName: /other/cls.d.ts
export declare class Cls {
  method(
    param: import("./doesntexist.js").Foo,
  ): import("./doesntexist.js").Foo;
}
// @FileName: /index.d.ts
import { Cls } from "./other/cls.js";

export declare class Derived extends Cls {
  /*1*/
}"#;
    let mut s = Session::new_for_test("completionsClassMemberImportTypeNodeParameter4", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
