use tsox_lsp::fourslash::Session;


#[test]
fn references_for_merged_declarations3() {
    let content = r#"[|class /*class*/[|testClass|] {
    static staticMethod() { }
    method() { }
}|]

[|module /*module*/[|testClass|] {
    export interface Bar {

    }
}|]

var c1: [|testClass|];
var c2: [|testClass|].Bar;
[|testClass|].staticMethod();
[|testClass|].prototype.method();
[|testClass|].bind(this);
new [|testClass|]();"#;
    let _s = Session::new_for_test("referencesForMergedDeclarations3", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "module", "class")
}
