//! Web namespaces.
//!
//! # Usage
//!
//! ```
//! use web_ns::*;
//!
//! let element = html5::HTML5_NS.element_by_local_name("div").unwrap();
//! assert_eq!(element, html5::tags::HtmlTag::Div);
//! ```
//!

#![forbid(unsafe_code)]

pub mod attr;

pub mod html5;

mod static_unicase;

pub use attr::*;

#[derive(Clone, Copy, Eq, PartialEq)]
enum WebNS {
    HTML5,
}

impl WebNS {
    pub fn web_namespace(&self) -> &'static dyn WebNamespace {
        match self {
            Self::HTML5 => &html5::HTML5_NS,
        }
    }
}

struct Private;

pub trait LocalName {
    fn local_name(&self) -> &str;
}

pub trait PropertyName {
    fn property_name(&self) -> &str;
}

///
/// A web namespace.
///
pub trait WebNamespace {
    /// The name of this webspace.
    fn name(&self) -> &'static str;
}

// TODO: Choose a better name? Get rid of untyped Namespace
pub trait TypedWebNamespace: WebNamespace {
    type Element;
    type Attribute;

    fn element_by_local_name(&self, local_name: &str) -> Result<Self::Element, Error>;

    fn attribute_by_local_name(
        &self,
        element: &Self::Element,
        local_name: &str,
    ) -> Result<Self::Attribute, Error>;
}

#[derive(Debug)]
pub enum Error {
    InvalidAttribute,
    InvalidAttributeValue,
}
