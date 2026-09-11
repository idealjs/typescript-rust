use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_top_level_for_await_module_blank_compiler_options_in_ts_config() {
    let content = r#"// @filename: /dir/a.ts
declare const p: number[];
for await (const _ of p);
export {};
// @filename: /dir/tsconfig.json
{
    "compilerOptions": {
        "module": "commonjs"
    }
}"#;
    let mut s = Session::new_for_test("codeFixTopLevelForAwait_module_blankCompilerOptionsInTsConfig", content);
    // TODO: f.VerifyCodeFixNotAvailable(t, "fixModuleOption")
}
