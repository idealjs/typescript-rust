use tsox_lsp::fourslash::{self, Session};


#[test]
fn references_for_no_context() {
    let content = r#"namespace modTest {
    //Declare
    export var modVar:number;
    /*1*/

    //Increments
    modVar++;

    class testCls{
        /*2*/
    }

    function testFn(){
        //Increments
        modVar++;
    }  /*3*/
/*4*/
    namespace testMod {
    }
}"#;
    let mut s = Session::new_for_test("referencesForNoContext", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
