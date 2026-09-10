use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: // Baseline auto-import completions at both markers"]
#[test]
fn prefer_type_only_auto_imports() {
    let content = r#"// @Filename: types.ts
export type MyType = { x: number };
export const MyValue = 123;
// @Filename: main.ts
let x: MyT/*type*/;
let y = MyV/*value*/;
"#;
    let mut s = Session::new_for_test("preferTypeOnlyAutoImports", content);
    fourslash::unsupported("Configure"); // f.Configure(t, lsutil.UserPreferences{
    // TODO: // Baseline auto-import completions at both markers
    fourslash::unsupported("BaselineAutoImportsCompletions"); // f.BaselineAutoImportsCompletions(t, []string{"type", "value"})
}
