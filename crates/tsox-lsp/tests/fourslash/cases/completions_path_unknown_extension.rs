use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_path_unknown_extension() {
    let content = r##"// @filename: src/some-file.ruhroh
/* This is just a test file that needs to exist. */

// @filename: package.json
{
    "imports": {
        "#/*": "./src/*"
    }
}

// @filename: src/globals.d.ts
declare module "*.ruhroh";

// @filename: src/a.mts
import "#//*$*/"

// @filename: tsconfig.json
{
    "compilerOptions": {
        "module": "preserve",
        "moduleResolution": "bundler",
        "rootDir": "src"
    },
    "include": ["src"]
}"##;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "$", &fourslash.CompletionsExpectedList{
}
