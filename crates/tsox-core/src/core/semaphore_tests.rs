use crate::core::semaphore::*;

#[test]
fn unlimited() { crate::fntrace::enter("unlimited"); 
    let s = UnlimitedSemaphore;
    let _g = s.acquire();
}

#[test]
fn limited() { crate::fntrace::enter("limited"); 
    let s = LimitedSemaphore::new(2);
    let g1 = s.acquire();
    let g2 = s.acquire();
    drop(g1);
    drop(g2);
    let _g3 = s.acquire();
}
