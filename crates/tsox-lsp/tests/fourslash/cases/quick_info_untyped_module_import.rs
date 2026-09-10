use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_untyped_module_import() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @strict: false
// @Filename: node_modules/foo/index.js
 /*index*/{}
// @Filename: a.ts
import /*foo*/foo from /*fooModule*/"foo";
/*fooCall*/foo();"#;
    let mut s = Session::new_for_test("quickInfoUntypedModuleImport", content);
    fourslash::go_to_file(&mut s, "a.ts");
    fourslash::verify_number_of_errors_in_current_file(&mut s, 0);
    fourslash::go_to_marker(&mut s, "fooModule");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "", "")
    fourslash::go_to_marker(&mut s, "foo");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "import foo", "")
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "foo", "fooModule", "fooCall")
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, false, "fooModule", "foo")
}
