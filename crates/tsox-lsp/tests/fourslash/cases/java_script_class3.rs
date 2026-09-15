use tsox_lsp::fourslash::Session;


#[test]
fn java_script_class3() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
class Foo {
   constructor() {
       this./*dst1*/alpha = 10;
       this./*dst2*/beta = 'gamma';
   }
   method() { return this.alpha; }
}
var x = new Foo();
x.[|alpha/*src1*/|];
x.[|beta/*src2*/|];"#;
    let _s = Session::new_for_test("javaScriptClass3", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "src1", "src2")
}
