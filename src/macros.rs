#[macro_export]
macro_rules! define_attrs {
    ($( ($name:ident, $attr:literal, $prop:literal, $q:tt, $format:tt) ),* ) => {
        $(
            mod $name {
                pub const ATTRIBUTE: &str = $attr;
                pub const PROPERTY: &str = $prop;

                pub const INTERNAL_ATTR: crate::internal::InternalAttr = crate::internal::InternalAttr {
                    attribute: ATTRIBUTE,
                    property: PROPERTY,
                    format: crate::attr_type::AttrFormat::$format,
                    attr_type: crate::attr_type::AttrType {
                        primitives: &[],
                        quantifier: crate::attr_type::Quantifier::$q,
                    },
                };
            }
        )*

        pub fn internal_attr_by_name(attribute: &str) -> Option<&'static crate::internal::InternalAttr> {
            match attribute {
                $(
                    $name::ATTRIBUTE => Some(&$name::INTERNAL_ATTR),
                )*
                _ => None
            }
        }

        pub fn internal_attr_by_property(property: &str) -> Option<&'static crate::internal::InternalAttr> {
            match property {
                $(
                    $name::PROPERTY => Some(&$name::INTERNAL_ATTR),
                )*
                _ => None
            }
        }
    };
}
