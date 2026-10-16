use tsox_lsp::fourslash::{self, Session};


#[test]
fn quickinfo_wrong_comment() {
    let content = r#"// @stableTypeOrdering: true
// @lib: es5
interface I {
    /** The colour */
    readonly colour: string
}
interface A extends I {
    readonly colour: "red" | "green";
}
interface B extends I {
    readonly colour: "yellow" | "green";
}
type F = A | B
const f: F = { colour: "green" }
f.colour/*1*/"#;
    let mut s = Session::new_for_test("quickinfoWrongComment", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyQuickInfoIs(t, "(property) colour: \"green\" | \"red\" | \"yellow\"", "The colour")
}
