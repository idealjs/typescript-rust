use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_on_value_symbol_without_export_with_same_name_export_symbol() {
    let content = r#"// @strict: true

declare function num(): number
const /*1*/Unit = num()
export type Unit = number
const value = /*2*/Unit

function Fn() {}
export type Fn = () => void
/*3*/Fn()

// repro from #41897
const /*4*/X = 1;
export interface X {}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "const Unit: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "const Unit: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "function Fn(): void", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "const X: 1", "")
}
