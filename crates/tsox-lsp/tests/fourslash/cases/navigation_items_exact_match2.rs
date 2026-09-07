use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyWorkspaceSymbol"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyWorkspaceSymbol"); // f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
}
