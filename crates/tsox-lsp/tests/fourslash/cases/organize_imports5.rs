use tsox_lsp::fourslash::{self, Session};


#[test]
fn organize_imports5() {
    let content = r#"import * as something from "path";/** 
 * some comment here
 * and there
 */
import * as somethingElse from "anotherpath";
import * as AnotherThing from "somepath";/** 
 * some comment here
 * and there
 */
import * as AnotherThingElse from "someotherpath";"#;
    let mut s = Session::new_for_test("organizeImports5", content);
    // TODO: f.VerifyOrganizeImports(t,
}
