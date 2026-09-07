use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn member_list_of_enum_from_external_module() {
    let content = r#"// @Filename: memberListOfEnumFromExternalModule_file0.ts
export enum Topic{ One, Two }
var topic = Topic.One;
// @Filename: memberListOfEnumFromExternalModule_file1.ts
import t = require('./memberListOfEnumFromExternalModule_file0');
var topic = t.Topic./*1*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
