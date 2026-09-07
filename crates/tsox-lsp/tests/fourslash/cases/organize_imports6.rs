use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyOrganizeImports"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}
