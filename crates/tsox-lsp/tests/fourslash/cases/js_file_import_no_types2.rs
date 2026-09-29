use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn js_file_import_no_types2() {
    let content = r#"// @allowJs: true
// @Filename: /default.ts
export default class TestDefaultClass {}
// @Filename: /defaultType.ts
export default interface TestDefaultInterface {}
// @Filename: /reExport/toReExport.ts
export class TestClassReExport {}
export interface TestInterfaceReExport {}
// @Filename: /reExport/index.ts
export { TestClassReExport, TestInterfaceReExport } from './toReExport';
// @Filename: /exportList.ts
class TestClassExportList {};
interface TestInterfaceExportList {};
export { TestClassExportList, TestInterfaceExportList };
// @Filename: /baseline.ts
export class TestClassBaseline {}
export interface TestInterfaceBaseline {}
// @Filename: /a.js
import /**/"#;
    let mut s = Session::new_for_test("jsFileImportNoTypes2", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
