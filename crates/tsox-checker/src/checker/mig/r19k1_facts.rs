use crate::checker::types::TypeFacts;

impl TypeFacts {
    pub const NONE: TypeFacts = TypeFacts::empty();

    pub const STRING_STRICT_FACTS: TypeFacts = TypeFacts::from_bits_retain(
        TYPE_FACS_BASE_STRING_STRICT.bits() | TypeFacts::TRUTHY.bits() | TypeFacts::FALSY.bits(),
    );
    pub const STRING_FACTS: TypeFacts =
        TypeFacts::from_bits_retain(TYPE_FACS_BASE_STRING.bits() | TypeFacts::TRUTHY.bits());
    pub const EMPTY_STRING_STRICT_FACTS: TypeFacts =
        TypeFacts::from_bits_retain(TYPE_FACS_BASE_STRING_STRICT.bits() | TypeFacts::FALSY.bits());
    pub const EMPTY_STRING_FACTS: TypeFacts = TYPE_FACS_BASE_STRING;
    pub const NON_EMPTY_STRING_STRICT_FACTS: TypeFacts = TypeFacts::from_bits_retain(
        TYPE_FACS_BASE_STRING_STRICT.bits() | TypeFacts::TRUTHY.bits(),
    );
    pub const NON_EMPTY_STRING_FACTS: TypeFacts =
        TypeFacts::from_bits_retain(TYPE_FACS_BASE_STRING.bits() | TypeFacts::TRUTHY.bits());

    pub const NUMBER_STRICT_FACTS: TypeFacts = TypeFacts::from_bits_retain(
        TYPE_FACS_BASE_NUMBER_STRICT.bits() | TypeFacts::TRUTHY.bits() | TypeFacts::FALSY.bits(),
    );
    pub const NUMBER_FACTS: TypeFacts =
        TypeFacts::from_bits_retain(TYPE_FACS_BASE_NUMBER.bits() | TypeFacts::TRUTHY.bits());
    pub const ZERO_NUMBER_STRICT_FACTS: TypeFacts =
        TypeFacts::from_bits_retain(TYPE_FACS_BASE_NUMBER_STRICT.bits() | TypeFacts::FALSY.bits());
    pub const ZERO_NUMBER_FACTS: TypeFacts = TYPE_FACS_BASE_NUMBER;
    pub const NON_ZERO_NUMBER_STRICT_FACTS: TypeFacts = TypeFacts::from_bits_retain(
        TYPE_FACS_BASE_NUMBER_STRICT.bits() | TypeFacts::TRUTHY.bits(),
    );
    pub const NON_ZERO_NUMBER_FACTS: TypeFacts =
        TypeFacts::from_bits_retain(TYPE_FACS_BASE_NUMBER.bits() | TypeFacts::TRUTHY.bits());

    pub const BIGINT_STRICT_FACTS: TypeFacts = TypeFacts::from_bits_retain(
        TYPE_FACS_BASE_BIGINT_STRICT.bits() | TypeFacts::TRUTHY.bits() | TypeFacts::FALSY.bits(),
    );
    pub const BIGINT_FACTS: TypeFacts =
        TypeFacts::from_bits_retain(TYPE_FACS_BASE_BIGINT.bits() | TypeFacts::TRUTHY.bits());
    pub const ZERO_BIGINT_STRICT_FACTS: TypeFacts =
        TypeFacts::from_bits_retain(TYPE_FACS_BASE_BIGINT_STRICT.bits() | TypeFacts::FALSY.bits());
    pub const ZERO_BIGINT_FACTS: TypeFacts = TYPE_FACS_BASE_BIGINT;
    pub const NON_ZERO_BIGINT_STRICT_FACTS: TypeFacts = TypeFacts::from_bits_retain(
        TYPE_FACS_BASE_BIGINT_STRICT.bits() | TypeFacts::TRUTHY.bits(),
    );
    pub const NON_ZERO_BIGINT_FACTS: TypeFacts =
        TypeFacts::from_bits_retain(TYPE_FACS_BASE_BIGINT.bits() | TypeFacts::TRUTHY.bits());

    pub const BOOLEAN_STRICT_FACTS: TypeFacts = TypeFacts::from_bits_retain(
        TYPE_FACS_BASE_BOOLEAN_STRICT.bits() | TypeFacts::TRUTHY.bits() | TypeFacts::FALSY.bits(),
    );
    pub const BOOLEAN_FACTS: TypeFacts =
        TypeFacts::from_bits_retain(TYPE_FACS_BASE_BOOLEAN.bits() | TypeFacts::TRUTHY.bits());
    pub const FALSE_STRICT_FACTS: TypeFacts =
        TypeFacts::from_bits_retain(TYPE_FACS_BASE_BOOLEAN_STRICT.bits() | TypeFacts::FALSY.bits());
    pub const FALSE_FACTS: TypeFacts = TYPE_FACS_BASE_BOOLEAN;
    pub const TRUE_STRICT_FACTS: TypeFacts = TypeFacts::from_bits_retain(
        TYPE_FACS_BASE_BOOLEAN_STRICT.bits() | TypeFacts::TRUTHY.bits(),
    );
    pub const TRUE_FACTS: TypeFacts =
        TypeFacts::from_bits_retain(TYPE_FACS_BASE_BOOLEAN.bits() | TypeFacts::TRUTHY.bits());

    pub const SYMBOL_STRICT_FACTS: TypeFacts = TypeFacts::from_bits_retain(
        TypeFacts::TYPEOF_EQ_SYMBOL.bits()
            | TypeFacts::TYPEOF_NE_STRING.bits()
            | TypeFacts::TYPEOF_NE_NUMBER.bits()
            | TypeFacts::TYPEOF_NE_BIGINT.bits()
            | TypeFacts::TYPEOF_NE_BOOLEAN.bits()
            | TypeFacts::TYPEOF_NE_OBJECT.bits()
            | TypeFacts::TYPEOF_NE_FUNCTION.bits()
            | TypeFacts::TYPEOF_NE_HOST_OBJECT.bits()
            | TypeFacts::NE_UNDEFINED.bits()
            | TypeFacts::NE_NULL.bits()
            | TypeFacts::NE_UNDEFINED_OR_NULL.bits()
            | TypeFacts::TRUTHY.bits(),
    );
    pub const SYMBOL_FACTS: TypeFacts = TypeFacts::from_bits_retain(
        TypeFacts::SYMBOL_STRICT_FACTS.bits()
            | TypeFacts::EQ_UNDEFINED.bits()
            | TypeFacts::EQ_NULL.bits()
            | TypeFacts::EQ_UNDEFINED_OR_NULL.bits()
            | TypeFacts::FALSY.bits(),
    );

    pub const OBJECT_STRICT_FACTS: TypeFacts = TypeFacts::from_bits_retain(
        TypeFacts::TYPEOF_EQ_OBJECT.bits()
            | TypeFacts::TYPEOF_EQ_HOST_OBJECT.bits()
            | TypeFacts::TYPEOF_NE_STRING.bits()
            | TypeFacts::TYPEOF_NE_NUMBER.bits()
            | TypeFacts::TYPEOF_NE_BIGINT.bits()
            | TypeFacts::TYPEOF_NE_BOOLEAN.bits()
            | TypeFacts::TYPEOF_NE_SYMBOL.bits()
            | TypeFacts::TYPEOF_NE_FUNCTION.bits()
            | TypeFacts::NE_UNDEFINED.bits()
            | TypeFacts::NE_NULL.bits()
            | TypeFacts::NE_UNDEFINED_OR_NULL.bits()
            | TypeFacts::TRUTHY.bits(),
    );
    pub const OBJECT_FACTS: TypeFacts = TypeFacts::from_bits_retain(
        TypeFacts::OBJECT_STRICT_FACTS.bits()
            | TypeFacts::EQ_UNDEFINED.bits()
            | TypeFacts::EQ_NULL.bits()
            | TypeFacts::EQ_UNDEFINED_OR_NULL.bits()
            | TypeFacts::FALSY.bits(),
    );

    pub const FUNCTION_STRICT_FACTS: TypeFacts = TypeFacts::from_bits_retain(
        TypeFacts::TYPEOF_EQ_FUNCTION.bits()
            | TypeFacts::TYPEOF_EQ_HOST_OBJECT.bits()
            | TypeFacts::TYPEOF_NE_STRING.bits()
            | TypeFacts::TYPEOF_NE_NUMBER.bits()
            | TypeFacts::TYPEOF_NE_BIGINT.bits()
            | TypeFacts::TYPEOF_NE_BOOLEAN.bits()
            | TypeFacts::TYPEOF_NE_SYMBOL.bits()
            | TypeFacts::TYPEOF_NE_OBJECT.bits()
            | TypeFacts::NE_UNDEFINED.bits()
            | TypeFacts::NE_NULL.bits()
            | TypeFacts::NE_UNDEFINED_OR_NULL.bits()
            | TypeFacts::TRUTHY.bits(),
    );
    pub const FUNCTION_FACTS: TypeFacts = TypeFacts::from_bits_retain(
        TypeFacts::FUNCTION_STRICT_FACTS.bits()
            | TypeFacts::EQ_UNDEFINED.bits()
            | TypeFacts::EQ_NULL.bits()
            | TypeFacts::EQ_UNDEFINED_OR_NULL.bits()
            | TypeFacts::FALSY.bits(),
    );

    pub const VOID_FACTS: TypeFacts = TypeFacts::from_bits_retain(
        TypeFacts::TYPEOF_NE_STRING.bits()
            | TypeFacts::TYPEOF_NE_NUMBER.bits()
            | TypeFacts::TYPEOF_NE_BIGINT.bits()
            | TypeFacts::TYPEOF_NE_BOOLEAN.bits()
            | TypeFacts::TYPEOF_NE_SYMBOL.bits()
            | TypeFacts::TYPEOF_NE_OBJECT.bits()
            | TypeFacts::TYPEOF_NE_FUNCTION.bits()
            | TypeFacts::TYPEOF_NE_HOST_OBJECT.bits()
            | TypeFacts::EQ_UNDEFINED.bits()
            | TypeFacts::EQ_UNDEFINED_OR_NULL.bits()
            | TypeFacts::NE_NULL.bits()
            | TypeFacts::FALSY.bits(),
    );
    pub const UNDEFINED_FACTS: TypeFacts = TypeFacts::from_bits_retain(
        TypeFacts::TYPEOF_NE_STRING.bits()
            | TypeFacts::TYPEOF_NE_NUMBER.bits()
            | TypeFacts::TYPEOF_NE_BIGINT.bits()
            | TypeFacts::TYPEOF_NE_BOOLEAN.bits()
            | TypeFacts::TYPEOF_NE_SYMBOL.bits()
            | TypeFacts::TYPEOF_NE_OBJECT.bits()
            | TypeFacts::TYPEOF_NE_FUNCTION.bits()
            | TypeFacts::TYPEOF_NE_HOST_OBJECT.bits()
            | TypeFacts::EQ_UNDEFINED.bits()
            | TypeFacts::EQ_UNDEFINED_OR_NULL.bits()
            | TypeFacts::NE_NULL.bits()
            | TypeFacts::FALSY.bits()
            | TypeFacts::IS_UNDEFINED.bits(),
    );
    pub const NULL_FACTS: TypeFacts = TypeFacts::from_bits_retain(
        TypeFacts::TYPEOF_EQ_OBJECT.bits()
            | TypeFacts::TYPEOF_NE_STRING.bits()
            | TypeFacts::TYPEOF_NE_NUMBER.bits()
            | TypeFacts::TYPEOF_NE_BIGINT.bits()
            | TypeFacts::TYPEOF_NE_BOOLEAN.bits()
            | TypeFacts::TYPEOF_NE_SYMBOL.bits()
            | TypeFacts::TYPEOF_NE_FUNCTION.bits()
            | TypeFacts::TYPEOF_NE_HOST_OBJECT.bits()
            | TypeFacts::EQ_NULL.bits()
            | TypeFacts::EQ_UNDEFINED_OR_NULL.bits()
            | TypeFacts::NE_UNDEFINED.bits()
            | TypeFacts::FALSY.bits()
            | TypeFacts::IS_NULL.bits(),
    );

    pub const EMPTY_OBJECT_STRICT_FACTS: TypeFacts = TypeFacts::from_bits_retain(
        TypeFacts::ALL.bits()
            & !(TypeFacts::EQ_UNDEFINED
                .union(TypeFacts::EQ_NULL)
                .union(TypeFacts::EQ_UNDEFINED_OR_NULL)
                .union(TypeFacts::IS_UNDEFINED_OR_NULL))
                .bits(),
    );
    pub const EMPTY_OBJECT_FACTS: TypeFacts = TypeFacts::from_bits_retain(
        TypeFacts::ALL.bits() & !TypeFacts::IS_UNDEFINED_OR_NULL.bits(),
    );
    pub const UNKNOWN_FACTS: TypeFacts = TypeFacts::from_bits_retain(
        TypeFacts::ALL.bits() & !TypeFacts::IS_UNDEFINED_OR_NULL.bits(),
    );
}

const TYPE_FACS_BASE_STRING_STRICT: TypeFacts = TypeFacts::from_bits_retain(
    TypeFacts::TYPEOF_EQ_STRING.bits()
        | TypeFacts::TYPEOF_NE_NUMBER.bits()
        | TypeFacts::TYPEOF_NE_BIGINT.bits()
        | TypeFacts::TYPEOF_NE_BOOLEAN.bits()
        | TypeFacts::TYPEOF_NE_SYMBOL.bits()
        | TypeFacts::TYPEOF_NE_OBJECT.bits()
        | TypeFacts::TYPEOF_NE_FUNCTION.bits()
        | TypeFacts::TYPEOF_NE_HOST_OBJECT.bits()
        | TypeFacts::NE_UNDEFINED.bits()
        | TypeFacts::NE_NULL.bits()
        | TypeFacts::NE_UNDEFINED_OR_NULL.bits(),
);
const TYPE_FACS_BASE_STRING: TypeFacts = TypeFacts::from_bits_retain(
    TYPE_FACS_BASE_STRING_STRICT.bits()
        | TypeFacts::EQ_UNDEFINED.bits()
        | TypeFacts::EQ_NULL.bits()
        | TypeFacts::EQ_UNDEFINED_OR_NULL.bits()
        | TypeFacts::FALSY.bits(),
);

const TYPE_FACS_BASE_NUMBER_STRICT: TypeFacts = TypeFacts::from_bits_retain(
    TypeFacts::TYPEOF_EQ_NUMBER.bits()
        | TypeFacts::TYPEOF_NE_STRING.bits()
        | TypeFacts::TYPEOF_NE_BIGINT.bits()
        | TypeFacts::TYPEOF_NE_BOOLEAN.bits()
        | TypeFacts::TYPEOF_NE_SYMBOL.bits()
        | TypeFacts::TYPEOF_NE_OBJECT.bits()
        | TypeFacts::TYPEOF_NE_FUNCTION.bits()
        | TypeFacts::TYPEOF_NE_HOST_OBJECT.bits()
        | TypeFacts::NE_UNDEFINED.bits()
        | TypeFacts::NE_NULL.bits()
        | TypeFacts::NE_UNDEFINED_OR_NULL.bits(),
);
const TYPE_FACS_BASE_NUMBER: TypeFacts = TypeFacts::from_bits_retain(
    TYPE_FACS_BASE_NUMBER_STRICT.bits()
        | TypeFacts::EQ_UNDEFINED.bits()
        | TypeFacts::EQ_NULL.bits()
        | TypeFacts::EQ_UNDEFINED_OR_NULL.bits()
        | TypeFacts::FALSY.bits(),
);

const TYPE_FACS_BASE_BIGINT_STRICT: TypeFacts = TypeFacts::from_bits_retain(
    TypeFacts::TYPEOF_EQ_BIGINT.bits()
        | TypeFacts::TYPEOF_NE_STRING.bits()
        | TypeFacts::TYPEOF_NE_NUMBER.bits()
        | TypeFacts::TYPEOF_NE_BOOLEAN.bits()
        | TypeFacts::TYPEOF_NE_SYMBOL.bits()
        | TypeFacts::TYPEOF_NE_OBJECT.bits()
        | TypeFacts::TYPEOF_NE_FUNCTION.bits()
        | TypeFacts::TYPEOF_NE_HOST_OBJECT.bits()
        | TypeFacts::NE_UNDEFINED.bits()
        | TypeFacts::NE_NULL.bits()
        | TypeFacts::NE_UNDEFINED_OR_NULL.bits(),
);
const TYPE_FACS_BASE_BIGINT: TypeFacts = TypeFacts::from_bits_retain(
    TYPE_FACS_BASE_BIGINT_STRICT.bits()
        | TypeFacts::EQ_UNDEFINED.bits()
        | TypeFacts::EQ_NULL.bits()
        | TypeFacts::EQ_UNDEFINED_OR_NULL.bits()
        | TypeFacts::FALSY.bits(),
);

const TYPE_FACS_BASE_BOOLEAN_STRICT: TypeFacts = TypeFacts::from_bits_retain(
    TypeFacts::TYPEOF_EQ_BOOLEAN.bits()
        | TypeFacts::TYPEOF_NE_STRING.bits()
        | TypeFacts::TYPEOF_NE_NUMBER.bits()
        | TypeFacts::TYPEOF_NE_BIGINT.bits()
        | TypeFacts::TYPEOF_NE_SYMBOL.bits()
        | TypeFacts::TYPEOF_NE_OBJECT.bits()
        | TypeFacts::TYPEOF_NE_FUNCTION.bits()
        | TypeFacts::TYPEOF_NE_HOST_OBJECT.bits()
        | TypeFacts::NE_UNDEFINED.bits()
        | TypeFacts::NE_NULL.bits()
        | TypeFacts::NE_UNDEFINED_OR_NULL.bits(),
);
const TYPE_FACS_BASE_BOOLEAN: TypeFacts = TypeFacts::from_bits_retain(
    TYPE_FACS_BASE_BOOLEAN_STRICT.bits()
        | TypeFacts::EQ_UNDEFINED.bits()
        | TypeFacts::EQ_NULL.bits()
        | TypeFacts::EQ_UNDEFINED_OR_NULL.bits()
        | TypeFacts::FALSY.bits(),
);
