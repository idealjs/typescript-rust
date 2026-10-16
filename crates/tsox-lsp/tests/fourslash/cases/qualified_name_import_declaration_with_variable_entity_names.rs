use tsox_lsp::fourslash::{self, Session};


#[test]
fn qualified_name_import_declaration_with_variable_entity_names() {
    let content = r#"namespace Alpha {
    export var [|{| "name" : "def" |}x|] = 100;
}

namespace Beta {
    import p = Alpha.[|{| "name" : "import" |}x|];
}

var x = Alpha.[|{| "name" : "mem" |}x|]"#;
    let mut s = Session::new_for_test("qualifiedName_import_declaration_with_variable_entity_names", content);
    fourslash::go_to_marker(&mut s, "import");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "import")
    // TODO: f.VerifyBaselineGoToDefinition(t, false, "import")
}
