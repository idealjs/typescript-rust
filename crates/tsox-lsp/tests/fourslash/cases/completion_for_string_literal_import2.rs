use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_for_string_literal_import2() {
    let content = r#"// @typeRoots: my_typings
// @Filename: test.ts
/// <reference path="./[|some|]/*0*/
/// <reference types="[|some|]/*1*/
/// <reference path="./sub/[|some|]/*2*/" />
/// <reference types="[|some|]/*3*/" />
// @Filename: someFile.ts
/*someFile*/
// @Filename: sub/someOtherFile.ts
/*someOtherFile*/
// @Filename: my_typings/some-module/index.d.ts
export var x = 9;"#;
    let mut s = Session::new_for_test("completionForStringLiteralImport2", content);
    fourslash::verify_completions_unsorted_at(&mut s, Some("0"), &["someFile.ts", "my_typings", "sub"]);
    fourslash::verify_completions_exact_at(&mut s, Some("1"), &["some-module"]);
    fourslash::verify_completions_exact_at(&mut s, Some("2"), &["someOtherFile.ts"]);
    fourslash::verify_completions_exact_at(&mut s, Some("3"), &["some-module"]);
}
