use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_partial_implementation() {
    let content = r#"// @Filename: goToDefinitionPartialImplementation_1.ts
namespace A {
    export interface /*Part1Definition*/IA {
        y: string;
    }
}
// @Filename: goToDefinitionPartialImplementation_2.ts
namespace A {
    export interface /*Part2Definition*/IA {
        x: number;
    }

    var x: [|/*Part2Use*/IA|];
}"#;
    let mut s = Session::new_for_test("goToDefinitionPartialImplementation", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "Part2Use")
}
