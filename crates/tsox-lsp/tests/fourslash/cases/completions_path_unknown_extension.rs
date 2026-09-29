use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("completionsPathUnknownExtension", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("$"), &["some-file.ruhroh"], &[]);
}
