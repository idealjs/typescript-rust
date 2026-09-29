use tsox_lsp::fourslash::Session;


#[test]
fn prefer_type_only_auto_imports() {
    let content = r#"// @Filename: types.ts
export type MyType = { x: number };
export const MyValue = 123;
// @Filename: main.ts
let x: MyT/*type*/;
let y = MyV/*value*/;
"#;
    let _s = Session::new_for_test("preferTypeOnlyAutoImports", content);
    // TODO: f.Configure(t, lsutil.UserPreferences{
    // TODO: // Baseline auto-import completions at both markers
    // TODO: f.BaselineAutoImportsCompletions(t, []string{"type", "value"})
}
