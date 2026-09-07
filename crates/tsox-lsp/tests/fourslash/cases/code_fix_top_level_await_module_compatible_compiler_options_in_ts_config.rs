use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_top_level_await_module_compatible_compiler_options_in_ts_config() {
    let content = r#"// @filename: /dir/a.ts
declare const p: Promise<number>;
await p;
export {};
// @filename: /dir/tsconfig.json
{
    "compilerOptions": {
        "target": "es2017",
        "module": "esnext"
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t, "fixModuleOption")
}
