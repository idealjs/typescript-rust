use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_class_member_import_type_node_parameter1() {
    let content = r#"// @module: node18
// @Filename: /generation.d.ts
export type GenerationConfigType = { max_length?: number };
// @FileName: /index.d.ts
export declare class PreTrainedModel {
  _get_generation_config(
    param: import("./generation.js").GenerationConfigType,
  ): import("./generation.js").GenerationConfigType;
}

export declare class BlenderbotSmallPreTrainedModel extends PreTrainedModel {
  /*1*/
}"#;
    let mut s = Session::new_for_test("completionsClassMemberImportTypeNodeParameter1", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
