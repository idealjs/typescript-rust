use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyWorkspaceSymbol"]
#[test]
fn navigation_items_prefix_match2() {
    let content = r#"// @lib: es5
namespace Shapes {
    export class Point {
        private [|originality|] = 0.0;
        private [|distanceFromOrig|] = 0.0;
        get [|distanceFarFarAway|](distanceFarFarAwayParam: number): number {
            var [|distanceFarFarAwayLocal|];
            return 0;
        }
    }
}
var pointsSquareBox = new Shapes.Point();
function PointsFunc(): void {
 var pointFuncLocal;
}
interface [|OriginI|] {
    123;
    [|origin1|];
    public [|_distance|](distanceParam): void;
}"#;
    let mut s = Session::new_for_test("navigationItemsPrefixMatch2", content);
    fourslash::unsupported("VerifyWorkspaceSymbol"); // f.VerifyWorkspaceSymbol(t, []*fourslash.VerifyWorkspaceSymbolCase{
}
