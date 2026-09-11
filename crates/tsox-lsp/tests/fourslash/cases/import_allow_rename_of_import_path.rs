use tsox_lsp::fourslash::{self, Session};


#[test]
fn allow_rename_of_import_path() {
    let content = r#"// @Filename: /a.ts
export const x = 0;
// @Filename: /dir/index.ts
export const x = 0;
// @Filename: /b.ts
import * as a from "./[|a|]";
import * as dir from "./[|dir|]";
import * as dir2 from "./dir/[|index|]";
// @Filename: /c.js
const a = require("./[|a|]");
"#;
    let mut s = Session::new_for_test("allowRenameOfImportPath", content);
    // TODO: prefsTrue := lsutil.UserPreferences{
    // TODO: prefsFalse := lsutil.UserPreferences{
    // TODO: markers := []string{"a", "dir", "index"}
    // TODO: f.Configure(t, prefsTrue)
    // TODO: f.GoToEachMarker(t, markers, func(marker *fourslash.Marker, index int) {
    // TODO: f.Configure(t, prefsFalse)
    // TODO: f.GoToEachMarker(t, markers, func(marker *fourslash.Marker, index int) {
}

#[test]
fn rename_info_for_import_path_trigger_span() {
    let content = r#"// @Filename: /library.ts
export const foo = "bar";
// @Filename: /index.ts
export * from "./[|lib/*rename*/rary|]";
"#;
    let mut s = Session::new_for_test("renameInfoForImportPathTriggerSpan", content);
    fourslash::go_to_marker(&mut s, "rename");
    // TODO: f.VerifyRenameRange(t, f.Ranges()[0].LSRange, "library", &lsutil.UserPreferences{
}
