use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNumberOfErrorsInCurrentFile"]
#[test]
fn incremental_parsing_dynamic_import4() {
    let content = r#"// @lib: es2015
// @Filename: ./foo.ts
export function bar() { return 1; }
// @Filename: ./0.ts
/*1*/
import { bar } from "./foo""#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 0)
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "import");
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 1)
}
