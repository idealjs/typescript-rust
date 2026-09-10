use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.Backspace"]
#[test]
fn js_doc_function_signatures3() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
var someObject = {
    /**
     * @param {string} param1 Some string param.
     * @param {number} parm2  Some number param.
     */
    someMethod: function(param1, param2) {
        console.log(param1/*1*/);
        return false;
    },
    /**
     * @param {number} p1  Some number param.
     */
    otherMethod(p1) {
        p1/*2*/
    }

};"#;
    let mut s = Session::new_for_test("jsDocFunctionSignatures3", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, ".");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("Backspace"); // f.Backspace(t, 1)
    fourslash::go_to_marker(&mut s, "2");
    fourslash::insert(&mut s, ".");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("Backspace"); // f.Backspace(t, 1)
}
