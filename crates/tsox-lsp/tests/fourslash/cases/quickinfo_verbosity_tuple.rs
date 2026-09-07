use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
#[test]
fn quickinfo_verbosity_tuple() {
    let content = r#"interface Orange {
    color: string;
}
interface Apple {
    color: string;
    other: Orange;
}
type TwoFruits/*T*/ = [Orange, Apple];
const tf/*f*/: TwoFruits = [
    { color: "orange" },
    { color: "red", other: { color: "orange" } }
];
const tf2/*f2*/: [Orange, Apple] = [
    { color: "orange" },
    { color: "red", other: { color: "orange" } }
];
type ManyFruits/*m*/ = (Orange | Apple)[];
const mf/*mf*/: ManyFruits = [];"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"T": {0, 1, 2}, "f": {0, 1, 2, 3}, "f2": {0, 
}
