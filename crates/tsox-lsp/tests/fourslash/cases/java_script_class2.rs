use tsox_lsp::fourslash::Session;


#[test]
fn java_script_class2() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
class Foo {
   constructor() {
       [|this.[|{| "contextRangeIndex": 0 |}union|] = 'foo';|]
       [|this.[|{| "contextRangeIndex": 2 |}union|] = 100;|]
   }
   method() { return this.[|union|]; }
}
var x = new Foo();
x.[|union|];"#;
    let _s = Session::new_for_test("javaScriptClass2", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "union")
}
