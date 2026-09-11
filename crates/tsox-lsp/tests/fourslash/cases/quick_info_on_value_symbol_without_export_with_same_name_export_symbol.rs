use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("quickInfoOnValueSymbolWithoutExportWithSameNameExportSymbol", content);
    fourslash::verify_quick_info_at(&mut s, "1", "const Unit: number", "");
    fourslash::verify_quick_info_at(&mut s, "2", "const Unit: number", "");
    fourslash::verify_quick_info_at(&mut s, "3", "function Fn(): void", "");
    fourslash::verify_quick_info_at(&mut s, "4", "const X: 1", "");
}
