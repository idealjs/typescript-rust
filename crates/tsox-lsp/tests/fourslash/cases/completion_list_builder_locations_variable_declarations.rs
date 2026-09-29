use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_list_builder_locations_variable_declarations() {
    let content = r#"// @lib: es5
var x = a/*var1*/
var x = (b/*var2*/
var x = (c, d/*var3*/
 var y : any = "", x = a/*var4*/
 var y : any = "", x = (a/*var5*/
class C{}
var y = new C(/*var6*/
 class C{}
 var y = new C(0, /*var7*/
var y = [/*var8*/
var y = [0, /*var9*/
var y = `${/*var10*/
var y = `${10} dd ${ /*var11*/
var y = 10; y=/*var12*/"#;
    let _s = Session::new_for_test("completionListBuilderLocations_VariableDeclarations", content);
    // TODO: f.VerifyCompletions(t, []string{"var1"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"var2", "var3", "var4", "var5", "var6", "var7", "var8", "var9", "var
    // TODO: }
}
