use super::*;

#[test]
fn unlimited() {
    let s = UnlimitedSemaphore;
    let _g = s.acquire();
}

#[test]
fn limited() {
    let s = LimitedSemaphore::new(2);
    let g1 = s.acquire();
    let g2 = s.acquire();
    drop(g1);
    drop(g2);
    let _g3 = s.acquire();
}
