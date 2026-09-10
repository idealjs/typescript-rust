use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_for_non_exported_member_in_ambient_module_with_export_assignment1() {
    let content = r#"// @Filename: completionListForNonExportedMemberInAmbientModuleWithExportAssignment1_file0.ts
var x: Date;
export = x;
// @Filename: completionListForNonExportedMemberInAmbientModuleWithExportAssignment1_file1.ts
///<reference path='completionListForNonExportedMemberInAmbientModuleWithExportAssignment1_file0.ts'/>
 import test = require("completionListForNonExportedMemberInAmbientModuleWithExportAssignment1_file0");
 test./**/"#;
    let mut s = Session::new_for_test("completionListForNonExportedMemberInAmbientModuleWithExportAssignment1", content);
    fourslash::verify_completions_empty_at(&mut s, Some(""));
}
