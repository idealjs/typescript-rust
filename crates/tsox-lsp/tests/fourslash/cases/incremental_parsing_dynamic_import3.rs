use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.Insert"]
#[test]
fn incremental_parsing_dynamic_import3() {
    let content = r#"// @lib: es2015
// @Filename: ./foo.ts
export function bar() { return 1; }
// @Filename: ./0.ts
var x = import/*1*/"#;
    let mut s = Session::new(content);
    fourslash::verify_number_of_errors_in_current_file(&mut s, 0);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("Insert"); // f.Insert(t, "(")
}
