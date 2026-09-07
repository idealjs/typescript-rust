#![allow(unused_imports)]

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JSDocState {
    BeginningOfLine,
    SawAsterisk,
    SavingComments,
    SavingBackticks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PropertyLikeParse(pub(crate) u8);

impl PropertyLikeParse {
    pub(crate) const PROPERTY: u8 = 1;
    pub(crate) const PARAMETER: u8 = 2;
    pub(crate) const CALLBACK_PARAMETER: u8 = 4;

    pub(crate) fn contains(self, flag: u8) -> bool {
        self.0 & flag != 0
    }
}
