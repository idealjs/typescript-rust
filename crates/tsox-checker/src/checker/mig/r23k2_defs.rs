use std::sync::Arc;

use crate::checker::checker::Checker;
use crate::checker::types::Type;

impl Checker {
    pub fn string_or_number_type(&mut self) -> Arc<Type> {
        let string_type = self.string_type();
        let number_type = self.number_type();
        self.get_union_type(vec![string_type, number_type])
    }
}
