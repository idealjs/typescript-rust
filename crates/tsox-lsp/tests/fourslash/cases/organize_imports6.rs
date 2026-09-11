use tsox_lsp::fourslash::{self, Session};


#[test]
fn organize_imports6() {
    let content = r#"import * as something from "path"; /* small comment */ // single line one.
/* some comment here
* and there
*/
import * as somethingElse from "anotherpath";
import * as anotherThing from "someopath"; /* small comment */ // single line one.
/* some comment here
* and there
*/
import * as anotherThingElse from "someotherpath";

anotherThing;"#;
    let mut s = Session::new_for_test("organizeImports6", content);
    // TODO: f.VerifyOrganizeImports(t,
}
