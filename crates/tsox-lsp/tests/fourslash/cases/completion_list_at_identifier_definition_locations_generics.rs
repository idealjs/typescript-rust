use tsox_lsp::fourslash::Session;


#[test]
fn completion_list_at_identifier_definition_locations_generics() {
    let content = r#"interface A</*genericName1*/
class A</*genericName2*/
class B<T, /*genericName3*/
class A{
     f</*genericName4*/
function A</*genericName5*/"#;
    let _s = Session::new_for_test("completionListAtIdentifierDefinitionLocations_Generics", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), nil)
    // TODO: }
}
