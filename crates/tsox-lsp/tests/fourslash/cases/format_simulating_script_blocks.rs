use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_simulating_script_blocks() {
    let content = r#"/* BEGIN EXTERNAL SOURCE */
/*begin5*/
                        var a = 1;
                        alert("/*end5*//********//*begin4*/");
                    /*end4*/
/* END EXTERNAL SOURCE */

/* BEGIN EXTERNAL SOURCE */
/*begin3*/
                            var b = 1;

                        var c = "/*end3*//********//*begin2*/";
       var d = 1;

            var e = "/*end2*//********//*begin1*/";
            var f = 1;
        /*end1*/
/* END EXTERNAL SOURCE */"#;
    let mut s = Session::new_for_test("formatSimulatingScriptBlocks", content);
    // TODO: opts640 := f.GetOptions()
    // TODO: opts640.FormatCodeSettings.BaseIndentSize = 12
    // TODO: f.Configure(t, opts640)
    fourslash::format_selection(&mut s, "begin1", "end1");
    fourslash::format_selection(&mut s, "begin2", "end2");
    fourslash::format_selection(&mut s, "begin3", "end3");
    // TODO: opts794 := f.GetOptions()
    // TODO: opts794.FormatCodeSettings.BaseIndentSize = 24
    // TODO: f.Configure(t, opts794)
    fourslash::format_selection(&mut s, "begin4", "end4");
    fourslash::format_selection(&mut s, "begin5", "end5");
    fourslash::verify_current_file_content(&mut s, r#"/* BEGIN EXTERNAL SOURCE */

                        var a = 1;
                        alert("/********/");

/* END EXTERNAL SOURCE */

/* BEGIN EXTERNAL SOURCE */

            var b = 1;

            var c = "/********/";
            var d = 1;

            var e = "/********/";
            var f = 1;

/* END EXTERNAL SOURCE */"#);
}
