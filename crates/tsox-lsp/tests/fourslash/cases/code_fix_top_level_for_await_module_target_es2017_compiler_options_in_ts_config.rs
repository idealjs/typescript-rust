use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_top_level_for_await_module_target_es2017_compiler_options_in_ts_config() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @filename: /dir/a.ts
declare const p: number[];
for await (const _ of p);
export {};
// @filename: /dir/tsconfig.json
{
    "compilerOptions": {
        "target": "es2017"
    }
}"#;
    let mut s = Session::new_for_test("codeFixTopLevelForAwait_module_targetES2017CompilerOptionsInTsConfig", content);
    // TODO: f.VerifyCodeFixNotAvailable(t, "fixTargetOption")
    // TODO: f.VerifyCodeFixAvailable(t, nil)
}
