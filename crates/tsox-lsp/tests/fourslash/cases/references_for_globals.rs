use tsox_lsp::fourslash::Session;


#[test]
fn references_for_globals() {
    let content = r#"// @Filename: referencesForGlobals_1.ts
/*1*/var /*2*/global = 2;

class foo {
    constructor (public global) { }
    public f(global) { }
    public f2(global) { }
}

class bar {
    constructor () {
        var n = /*3*/global;

        var f = new foo('');
        f.global = '';
    }
}

var k = /*4*/global;
// @Filename: referencesForGlobals_2.ts
var m = /*5*/global;"#;
    let _s = Session::new_for_test("referencesForGlobals", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5")
}
