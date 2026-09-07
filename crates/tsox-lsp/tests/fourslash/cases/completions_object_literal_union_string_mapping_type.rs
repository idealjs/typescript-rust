use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_object_literal_union_string_mapping_type() {
    let content = r#"type UnionType = {
  key1: string;
} | {
  key2: number;
} | Uppercase<string>;

const obj1: UnionType = {
  /*1*/
};

const obj2: UnionType = {
  key1: "abc",
  /*2*/
};"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
