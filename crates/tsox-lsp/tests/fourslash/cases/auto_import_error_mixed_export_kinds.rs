use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: // Verify we don't crash from the mixed exports"]
#[test]
fn auto_import_error_mixed_export_kinds() {
    let content = r#"// @Filename: a.ts
export function foo(): number {
	return 10
}

const bar = 20;
export { bar as foo };

// @Filename: b.ts
foo/**/
"#;
    let mut s = Session::new_for_test("autoImportErrorMixedExportKinds", content);
    // TODO: // Verify we don't crash from the mixed exports
    fourslash::unsupported("BaselineAutoImportsCompletions"); // f.BaselineAutoImportsCompletions(t, []string{""})
}
