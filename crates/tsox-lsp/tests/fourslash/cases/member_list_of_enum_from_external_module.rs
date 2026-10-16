use tsox_lsp::fourslash::{self, Session};


#[test]
fn member_list_of_enum_from_external_module() {
    let content = r#"// @Filename: memberListOfEnumFromExternalModule_file0.ts
export enum Topic{ One, Two }
var topic = Topic.One;
// @Filename: memberListOfEnumFromExternalModule_file1.ts
import t = require('./memberListOfEnumFromExternalModule_file0');
var topic = t.Topic./*1*/"#;
    let mut s = Session::new_for_test("memberListOfEnumFromExternalModule", content);
    fourslash::verify_completions_exact_at(&mut s, Some("1"), &["One", "Two"]);
}
