use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_import_clause03() {
    let content = r#"declare module "M1" {
    export var abc: number;
    export var def: string;
}

declare module "M2" {
    import { abc/**/ } from "M1";
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
