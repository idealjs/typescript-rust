use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_try_catch_finally_broken() {
    let content = r#"t /*1*/ry {
    t/*2*/ry {
    }
    ctch (x) {
    }

    tr {
    }
    fin/*3*/ally {
    }
}
c/*4*/atch (e) {
}
f/*5*/inally {
}

// Missing catch variable
t/*6*/ry {
}
catc/*7*/h {
}
/*8*/finally {
}

// Missing try entirely
cat/*9*/ch (x) {
}
final/*10*/ly {
}"#;
    let _s = Session::new_for_test("getOccurrencesTryCatchFinallyBroken", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Markers())...)
}
