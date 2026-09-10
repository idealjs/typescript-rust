use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn js_doc_see1() {
    let content = r#"interface [|/*def1*/Foo|] {
    foo: string
}
namespace NS {
    export interface [|/*def2*/Bar|] {
        baz: Foo
    }
}
/** @see {/*use1*/[|Foo|]} foooo*/
const a = ""
/** @see {NS./*use2*/[|Bar|]} ns.bar*/
const b = ""
/** @see /*use3*/[|Foo|] f1*/
const c = ""
/** @see NS./*use4*/[|Bar|] ns.bar*/
const [|/*def3*/d|] = ""
/** @see /*use5*/[|d|] dd*/
const e = """#;
    let mut s = Session::new_for_test("jsDocSee1", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, false, "use1", "use2", "use3", "use4", "use5")
}
