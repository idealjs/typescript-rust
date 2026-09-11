use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_top_level_await_module_missing_compiler_options_in_ts_config() {
    let content = r#"// @filename: /dir/a.ts
declare const p: Promise<number>;
await p;
export {};
// @filename: /dir/tsconfig.json
{
    "compilerOptions": {
        "module": "commonjs"
    }
}"#;
    let mut s = Session::new_for_test("codeFixTopLevelAwait_module_missingCompilerOptionsInTsConfig", content);
    // TODO: f.VerifyCodeFixNotAvailable(t, "fixModuleOption")
}
