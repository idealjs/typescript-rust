use tsox_lsp::fourslash::Session;


#[test]
fn quickinfo_verbosity_typeof() {
    let content = r#"interface Apple {
    color: string;
    weight: number;
}
const a: Apple = { color: "red", weight: 150 };
const b/*b*/: typeof a = { color: "green", weight: 120 };
class Banana {
    length: number;
    constructor(length: number) {
        this.length = length;
    }
}
const c/*c*/: typeof Banana = Banana;"#;
    let _s = Session::new_for_test("quickinfoVerbosityTypeof", content);
    // TODO: f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"b": {0, 1}, "c": {0, 1}})
}
