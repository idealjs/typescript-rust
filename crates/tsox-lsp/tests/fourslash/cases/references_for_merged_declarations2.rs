use tsox_lsp::fourslash::Session;


#[test]
fn references_for_merged_declarations2() {
    let content = r#"namespace ATest {
    export interface Bar { }
}

function ATest() { }

/*1*/import /*2*/alias = ATest; // definition

var a: /*3*/alias.Bar; // namespace
/*4*/alias.call(this); // value"#;
    let _s = Session::new_for_test("referencesForMergedDeclarations2", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
