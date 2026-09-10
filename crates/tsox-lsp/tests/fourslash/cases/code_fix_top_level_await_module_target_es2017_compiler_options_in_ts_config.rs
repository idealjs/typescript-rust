use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_top_level_await_module_target_es2017_compiler_options_in_ts_config() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @filename: /dir/a.ts
declare const p: Promise<number>;
await p;
export {};
// @filename: /dir/tsconfig.json
{
    "compilerOptions": {
        "target": "es2017"
    }
}"#;
    let mut s = Session::new_for_test("codeFixTopLevelAwait_module_targetES2017CompilerOptionsInTsConfig", content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t, "fixTargetOption")
    fourslash::unsupported("VerifyCodeFixAvailable"); // f.VerifyCodeFixAvailable(t, nil)
}
