use tsox_lsp::fourslash::Session;


#[test]
fn navigation_items_exact_match2() {
    let content = r#"module Shapes {
    class [|Point|] {
        private [|_origin|] = 0.0;
        private [|distanceFromA|] = 0.0;

        get [|distance1|](distanceParam): number {
            var [|distanceLocal|];
            return 0;
        }
    }
}

var [|point|] = new Shapes.Point();
function [|distance2|](distanceParam1): void {
    var [|distanceLocal1|];
}"#;
    let _s = Session::new_for_test("navigationItemsExactMatch2", content);
    // TODO: f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
}
