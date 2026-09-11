use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_pre_processed_file() {
    let content = r#"// @moduleResolution: classic
// @Filename: refFile1.ts
class D { }
// @Filename: refFile2.ts
export class E {}
// @Filename: main.ts
// @ResolveReference: true
///<reference path="refFile1.ts" />
///<reference path = "/*1*/NotExistRef.ts/*2*/" />
/*3*////<reference path "invalidRefFile1.ts" />/*4*/
import ref2 = require("refFile2");
import noExistref2 = require(/*5*/"NotExistRefFile2"/*6*/);
import invalidRef1  /*7*/require/*8*/("refFile2");
import invalidRef2 = /*9*/requi/*10*/(/*10A*/"refFile2");
var obj: /*11*/C/*12*/;
var obj1: D;
var obj2: ref2.E;"#;
    let mut s = Session::new_for_test("getPreProcessedFile", content);
    fourslash::go_to_file(&mut s, "main.ts");
    fourslash::verify_number_of_errors_in_current_file(&mut s, 7);
    // TODO: f.VerifyErrorExistsBetweenMarkers(t, "1", "2")
    // TODO: f.VerifyErrorExistsBetweenMarkers(t, "3", "4")
    // TODO: f.VerifyErrorExistsBetweenMarkers(t, "5", "6")
    // TODO: f.VerifyErrorExistsBetweenMarkers(t, "7", "8")
    // TODO: f.VerifyErrorExistsBetweenMarkers(t, "9", "10")
    // TODO: f.VerifyErrorExistsBetweenMarkers(t, "10", "10A")
    // TODO: f.VerifyErrorExistsBetweenMarkers(t, "11", "12")
}
