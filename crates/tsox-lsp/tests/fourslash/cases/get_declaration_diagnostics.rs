use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_declaration_diagnostics() {
    let content = r#"// @strict: false
// @declaration: true
// @outDir: out
// @Filename: inputFile1.ts
namespace m {
   export function foo() {
       class C implements I { private a; }
       interface I { }
       return C;
   }
} /*1*/
// @Filename: input2.ts
var x = "hello world"; /*2*/"#;
    let mut s = Session::new_for_test("getDeclarationDiagnostics", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_number_of_errors_in_current_file(&mut s, 0);
}
