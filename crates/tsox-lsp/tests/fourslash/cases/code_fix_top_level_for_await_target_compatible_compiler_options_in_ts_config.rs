use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_top_level_for_await_target_compatible_compiler_options_in_ts_config() {
    let content = r#"// @filename: /dir/a.ts
declare const p: number[];
for await (const _ of p);
export {};
// @filename: /dir/tsconfig.json
{
    "compilerOptions": {
        "target": "es2017",
        "module": "esnext"
    }
}"#;
    let _s = Session::new_for_test("codeFixTopLevelForAwait_target_compatibleCompilerOptionsInTsConfig", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
