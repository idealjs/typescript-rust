use tsox_lsp::fourslash::Session;


#[test]
fn completion_list_at_invalid_locations() {
    let content = r#"var v1 = '';
" /*openString1*/
var v2 = '';
"/*openString2*/
var v3 = '';
" bar./*openString3*/
var v4 = '';
// bar./*inComment1*/
var v6 = '';
// /*inComment2*/
var v7 = '';
/* /*inComment3*/
var v11 = '';
  // /*inComment4*/
var v12 = '';
type htm/*inTypeAlias*/

//  /*inComment5*/
foo;
var v10 = /reg/*inRegExp1*/ex/;"#;
    let _s = Session::new_for_test("completionListAtInvalidLocations", content);
    // TODO: f.VerifyCompletions(t, []string{"openString1", "openString2", "openString3"}, &fourslash.Completions
    // TODO: f.VerifyCompletions(t, []string{"inComment1", "inComment2", "inComment3", "inComment4", "inTypeAlias
}
