use tsox_lsp::fourslash::{self, Session};


#[test]
fn js_doc_snippet_completion_for_function() {
    let content = r#"/*completion*/ */
function abcdef(x, y) { }
"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(true)
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::go_to_marker(&mut s, "completion");
    fourslash::insert(&mut s, "/**");
    // TODO: list := f.GetCompletions(t, nil /*userPreferences*/)
    // TODO: assert.Assert(t, list != nil)
    // TODO: assert.Equal(t, len(list.Items), 1)
    // TODO: item := list.Items[0]
    // TODO: assert.Equal(t, item.Label, "/** */")
    // TODO: assert.DeepEqual(t, item.Kind, new(lsproto.CompletionItemKindText))
    // TODO: assert.DeepEqual(t, item.Detail, new("JSDoc comment"))
    // TODO: assert.DeepEqual(t, item.SortText, new("\x00"))
    // TODO: assert.DeepEqual(t, item.CommitCharacters, &[]string{})
    // TODO: assert.DeepEqual(t, item.InsertTextFormat, new(lsproto.InsertTextFormatSnippet))
    // TODO: assert.Assert(t, item.TextEdit != nil)
    // TODO: assert.Assert(t, item.TextEdit.InsertReplaceEdit != nil)
    // TODO: assert.Equal(t, item.TextEdit.InsertReplaceEdit.NewText, "/**\n * $0\n * @param x ${1}\n * @param y 
    // TODO: assert.DeepEqual(t, item.TextEdit.InsertReplaceEdit.Insert, lsproto.Range{
    // TODO: assert.DeepEqual(t, item.TextEdit.InsertReplaceEdit.Replace, lsproto.Range{
}

#[test]
fn js_doc_snippet_completion_for_return() {
    let content = r#"/*completion*/ */
function abcdef(x) { return x; }
"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(true)
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::go_to_marker(&mut s, "completion");
    fourslash::insert(&mut s, "/**");
    // TODO: list := f.GetCompletions(t, nil /*userPreferences*/)
    // TODO: assert.Assert(t, list != nil)
    // TODO: assert.Equal(t, len(list.Items), 1)
    // TODO: assert.Equal(t, list.Items[0].TextEdit.InsertReplaceEdit.NewText, "/**\n * $0\n * @param x ${1}\n * 
}

#[test]
fn js_doc_snippet_completion_preserves_crlf() {
    let content = r#"/*completion*/ */
function abcdef(x) { return x; }
"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(true)
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::go_to_marker(&mut s, "completion");
    fourslash::insert(&mut s, "/**");
    // TODO: userPreferences := lsutil.NewDefaultUserPreferences()
    // TODO: userPreferences.FormatCodeSettings.NewLineCharacter = "\r\n"
    // TODO: f.Configure(t, userPreferences)
    // TODO: list := f.GetCompletions(t, nil /*userPreferences*/)
    // TODO: assert.Assert(t, list != nil)
    // TODO: assert.Equal(t, len(list.Items), 1)
    // TODO: assert.Equal(t, list.Items[0].TextEdit.InsertReplaceEdit.NewText, "/**\r\n * $0\r\n * @param x ${1}\
}

#[test]
fn js_doc_snippet_completion_respects_generate_return_preference() {
    let content = r#"/*completion*/ */
function abcdef(x) { return x; }
"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(true)
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::go_to_marker(&mut s, "completion");
    fourslash::insert(&mut s, "/**");
    // TODO: userPreferences := lsutil.NewDefaultUserPreferences()
    // TODO: userPreferences.GenerateReturnInDocTemplate = core.TSFalse
    // TODO: list := f.GetCompletions(t, &userPreferences)
    // TODO: assert.Assert(t, list != nil)
    // TODO: assert.Equal(t, len(list.Items), 1)
    // TODO: assert.Equal(t, list.Items[0].TextEdit.InsertReplaceEdit.NewText, "/**\n * $0\n * @param x ${1}\n */
}

#[test]
fn js_doc_snippet_completion_respects_enabled_preference() {
    let content = r#"/*completion*/ */
function abcdef(x) { }
"#;
    let mut s = Session::new_for_test("jSDocSnippetCompletionRespectsEnabledPreference", content);
    fourslash::go_to_marker(&mut s, "completion");
    fourslash::insert(&mut s, "/**");
    // TODO: userPreferences := lsutil.NewDefaultUserPreferences()
    // TODO: userPreferences.EnableJSDocCompletions = core.TSFalse
    // TODO: list := f.GetCompletions(t, &userPreferences)
    // TODO: if list != nil {
}

#[test]
fn js_doc_snippet_completion_for_class() {
    let content = r#"/*completion*/
class C {
}
"#;
    // TODO: capabilities := fourslash.GetDefaultCapabilities()
    // TODO: capabilities.TextDocument.Completion.CompletionItem.SnippetSupport = new(true)
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::go_to_marker(&mut s, "completion");
    fourslash::insert(&mut s, "/**");
    // TODO: list := f.GetCompletions(t, nil /*userPreferences*/)
    // TODO: assert.Assert(t, list != nil)
    // TODO: assert.Equal(t, len(list.Items), 1)
    // TODO: assert.Equal(t, list.Items[0].TextEdit.InsertReplaceEdit.NewText, "/**\n * $0\n */")
}

#[test]
fn js_doc_snippet_completion_not_in_non_empty_comment() {
    let content = r#"/** text /*completion*/ */
function abcdef(x) { }
"#;
    let mut s = Session::new_for_test("jSDocSnippetCompletionNotInNonEmptyComment", content);
    fourslash::verify_completions_empty_at(&mut s, Some("completion"));
}
