use tsox_lsp::fourslash::{self, Session};


#[test]
fn js_file_import_no_types2() {
    // TODO: t.Skip("Known failing fourslash test")
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
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
