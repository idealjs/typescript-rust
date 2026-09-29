use tsox_lsp::fourslash::{self, Session};


#[test]
fn incremental_parsing_dynamic_import4() {
    let content = r#"// @lib: es2015
// @Filename: ./foo.ts
export function bar() { return 1; }
// @Filename: ./0.ts
/*1*/
import { bar } from "./foo""#;
    let mut s = Session::new_for_test("incrementalParsingDynamicImport4", content);
    fourslash::verify_number_of_errors_in_current_file(&mut s, 0);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "import");
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
}
