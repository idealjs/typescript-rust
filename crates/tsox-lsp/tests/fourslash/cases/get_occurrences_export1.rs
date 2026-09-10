use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn get_occurrences_export1() {
    let content = r#"namespace m {
    [|export|] class C1 {
        public pub1;
        public pub2;
        private priv1;
        private priv2;
        protected prot1;
        protected prot2;

        public public;
        private private;
        protected protected;

        public constructor(public a, private b, protected c, public d, private e, protected f) {
            this.public = 10;
            this.private = 10;
            this.protected = 10;
        }

        public get x() { return 10; }
        public set x(value) { }

        public static statPub;
        private static statPriv;
        protected static statProt;
    }

    [|export|] interface I1 {
    }

    [|export|] declare namespace ma.m1.m2.m3 {
        interface I2 {
        }
    }

    [|export|] namespace mb.m1.m2.m3 {
        declare var foo;

        export class C2 {
            public pub1;
            private priv1;
            protected prot1;

            protected constructor(public public, protected protected, private private) {
            }
        }
    }

    declare var ambientThing: number;
    [|export|] var exportedThing = 10;
    declare function foo(): string;
}"#;
    let mut s = Session::new_for_test("getOccurrencesExport1", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
