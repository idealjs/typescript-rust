bitflags::bitflags! {

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    #[allow(non_camel_case_types)]
    pub struct SymbolFlags: u32 {
        const None                   = 0;
        const FunctionScopedVariable = 1 << 0;
        const BlockScopedVariable    = 1 << 1;
        const Property               = 1 << 2;
        const EnumMember             = 1 << 3;
        const Function               = 1 << 4;
        const Class                  = 1 << 5;
        const Interface              = 1 << 6;
        const ConstEnum              = 1 << 7;
        const RegularEnum            = 1 << 8;
        const ValueModule            = 1 << 9;
        const NamespaceModule        = 1 << 10;
        const TypeLiteral            = 1 << 11;
        const ObjectLiteral          = 1 << 12;
        const Method                 = 1 << 13;
        const Constructor            = 1 << 14;
        const GetAccessor            = 1 << 15;
        const SetAccessor            = 1 << 16;
        const Signature              = 1 << 17;
        const TypeParameter          = 1 << 18;
        const TypeAlias              = 1 << 19;
        const ExportValue            = 1 << 20;
        const Alias                  = 1 << 21;
        const Prototype              = 1 << 22;
        const ExportStar             = 1 << 23;
        const Optional               = 1 << 24;
        const Transient              = 1 << 25;
        const Assignment             = 1 << 26;
        const ModuleExports          = 1 << 27;
        const ConstEnumOnlyModule    = 1 << 28;
        const ReplaceableByMethod    = 1 << 29;
        const GlobalLookup           = 1 << 30;
    }
}

#[allow(non_upper_case_globals)]
impl SymbolFlags {
    pub const ENUM: Self = Self::RegularEnum.union(Self::ConstEnum);
    pub const VARIABLE: Self = Self::FunctionScopedVariable.union(Self::BlockScopedVariable);
    pub const VALUE: Self = Self::VARIABLE
        .union(Self::Property)
        .union(Self::EnumMember)
        .union(Self::ObjectLiteral)
        .union(Self::Function)
        .union(Self::Class)
        .union(Self::ENUM)
        .union(Self::ValueModule)
        .union(Self::Method)
        .union(Self::GetAccessor)
        .union(Self::SetAccessor);
    pub const TYPE: Self = Self::Class
        .union(Self::Interface)
        .union(Self::ENUM)
        .union(Self::EnumMember)
        .union(Self::TypeLiteral)
        .union(Self::TypeParameter)
        .union(Self::TypeAlias);
    pub const NAMESPACE: Self = Self::ValueModule
        .union(Self::NamespaceModule)
        .union(Self::ENUM);
    pub const MODULE: Self = Self::ValueModule.union(Self::NamespaceModule);
    pub const ACCESSOR: Self = Self::GetAccessor.union(Self::SetAccessor);
    pub const BLOCK_SCOPED: Self = Self::BlockScopedVariable
        .union(Self::Class)
        .union(Self::ENUM);
    pub const PROPERTY_OR_ACCESSOR: Self = Self::Property.union(Self::ACCESSOR);
    pub const CLASS_MEMBER: Self = Self::Method.union(Self::ACCESSOR).union(Self::Property);
    pub const MODULE_MEMBER: Self = Self::VARIABLE
        .union(Self::Function)
        .union(Self::Class)
        .union(Self::Interface)
        .union(Self::ENUM)
        .union(Self::MODULE)
        .union(Self::TypeAlias)
        .union(Self::Alias);
    pub const EXPORT_HAS_LOCAL: Self = Self::Function
        .union(Self::Class)
        .union(Self::ENUM)
        .union(Self::ValueModule);

    pub const FunctionScopedVariableExcludes: Self =
        Self::VALUE.difference(Self::FunctionScopedVariable);
    pub const BlockScopedVariableExcludes: Self = Self::VALUE;
    pub const ParameterExcludes: Self = Self::VALUE;
    pub const PropertyExcludes: Self = Self::VALUE.difference(Self::Property.union(Self::ACCESSOR));
    pub const EnumMemberExcludes: Self = Self::VALUE.union(Self::TYPE);
    pub const FunctionExcludes: Self =
        Self::VALUE.difference(Self::Function.union(Self::ValueModule).union(Self::Class));
    pub const ClassExcludes: Self = (Self::VALUE.union(Self::TYPE)).difference(
        Self::ValueModule
            .union(Self::Interface)
            .union(Self::Function),
    );
    pub const InterfaceExcludes: Self = Self::TYPE.difference(Self::Interface.union(Self::Class));
    pub const RegularEnumExcludes: Self =
        (Self::VALUE.union(Self::TYPE)).difference(Self::RegularEnum.union(Self::ValueModule));
    pub const ConstEnumExcludes: Self = (Self::VALUE.union(Self::TYPE)).difference(Self::ConstEnum);
    pub const ValueModuleExcludes: Self = Self::VALUE.difference(
        Self::Function
            .union(Self::Class)
            .union(Self::RegularEnum)
            .union(Self::ValueModule),
    );
    pub const NamespaceModuleExcludes: Self = Self::None;
    pub const MethodExcludes: Self = Self::VALUE.difference(Self::Method);
    pub const GetAccessorExcludes: Self =
        Self::VALUE.difference(Self::SetAccessor.union(Self::Property));
    pub const SetAccessorExcludes: Self =
        Self::VALUE.difference(Self::GetAccessor.union(Self::Property));
    pub const AccessorExcludes: Self = Self::VALUE.difference(Self::Property);
    pub const TypeParameterExcludes: Self = Self::TYPE.difference(Self::TypeParameter);
    pub const TypeAliasExcludes: Self = Self::TYPE;
    pub const AliasExcludes: Self = Self::Alias;
}

bitflags::bitflags! {

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct CheckFlags: u32 {
        const None                   = 0;
        const Instantiated           = 1 << 0;
        const SyntheticProperty      = 1 << 1;
        const SyntheticMethod        = 1 << 2;
        const Readonly               = 1 << 3;
        const ReadPartial            = 1 << 4;
        const WritePartial           = 1 << 5;
        const HasNonUniformType      = 1 << 6;
        const HasLiteralType         = 1 << 7;
        const ContainsPublic         = 1 << 8;
        const ContainsProtected      = 1 << 9;
        const ContainsPrivate        = 1 << 10;
        const ContainsStatic         = 1 << 11;
        const Late                   = 1 << 12;
        const ReverseMapped          = 1 << 13;
        const OptionalParameter      = 1 << 14;
        const RestParameter          = 1 << 15;
        const DeferredType           = 1 << 16;
        const HasNeverType           = 1 << 17;
        const Mapped                 = 1 << 18;
        const StripOptional          = 1 << 19;
        const Unresolved             = 1 << 20;
        const IsDiscriminantComputed = 1 << 21;
        const IsDiscriminant         = 1 << 22;
        const IndexSymbol            = 1 << 23;
    }
}

impl CheckFlags {
    pub const SYNTHETIC: Self = Self::SyntheticProperty.union(Self::SyntheticMethod);
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ContainerFlags(u32);

impl ContainerFlags {
    pub const NONE: Self = Self(0);
    pub const IS_CONTAINER: Self = Self(1 << 0);
    pub const IS_BLOCK_SCOPED_CONTAINER: Self = Self(1 << 1);
    pub const IS_CONTROL_FLOW_CONTAINER: Self = Self(1 << 2);
    pub const IS_FUNCTION_LIKE: Self = Self(1 << 3);
    pub const IS_FUNCTION_EXPRESSION: Self = Self(1 << 4);
    pub const HAS_LOCALS: Self = Self(1 << 5);
    pub const IS_INTERFACE: Self = Self(1 << 6);
    pub const IS_OBJECT_LITERAL_OR_CLASS_EXPRESSION_METHOD_OR_ACCESSOR: Self = Self(1 << 7);
    pub const IS_THIS_CONTAINER: Self = Self(1 << 8);
    pub const PROPAGATES_THIS_KEYWORD: Self = Self(1 << 9);

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl std::ops::BitOr for ContainerFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
