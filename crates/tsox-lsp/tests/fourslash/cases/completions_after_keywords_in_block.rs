use tsox_lsp::fourslash::Session;


#[test]
fn completions_after_keywords_in_block() {
    let content = r#"class C1 {
    method(map: Map<string, string>, key: string, defaultValue: string) {
        try {
            return map.get(key)!;
        }
        catch {
            return default/*1*/
        }
    }
}
class C2 {
    method(map: Map<string, string>, key: string, defaultValue: string) {
        if (map.has(key)) {
            return map.get(key)!;
        }
        else {
            return default/*2*/
        }
    }
}
class C3 {
    method(map: Map<string, string>, key: string, returnValue: string) {
        try {
            return map.get(key)!;
        }
        catch {
            return return/*3*/
        }
    }
}
class C4 {
    method(map: Map<string, string>, key: string, returnValue: string) {
        if (map.has(key)) {
            return map.get(key)!;
        }
        else {
            return return/*4*/
        }
    }
}"#;
    let _s = Session::new_for_test("completionsAfterKeywordsInBlock", content);
    // TODO: f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"3", "4"}, &fourslash.CompletionsExpectedList{
}
