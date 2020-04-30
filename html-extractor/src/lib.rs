#![allow(clippy::needless_doctest_main)]
//! This crate provides an easy way to extract data from HTML.
//! 
//! [`HtmlExtractor`] is neither a parser nor a deserializer.
//! It picks up only the desired data from HTML.
//!
//! [`html_extractor!`](macro.html_extractor.html) will help to implement [`HtmlExtractor`].
//!
//! # Examples
//! ## Extracting a simple value from HTML
//! ```
//! use html_extractor::{html_extractor, HtmlExtractor};
//! html_extractor! {
//!     #[derive(Debug, PartialEq)]
//!     Foo {
//!         foo: usize = (text of "#foo"),
//!     }
//! }
//!
//! fn main() {
//!     let input = r#"
//!         <div id="foo">1</div>
//!     "#;
//!     let foo = Foo::extract_from_str(input).unwrap();
//!     assert_eq!(foo, Foo { foo: 1 });
//! }
//! ```
//!
//! ## Extracting a collection from HTML
//! ```
//! use html_extractor::{html_extractor, HtmlExtractor};
//! html_extractor! {
//!     #[derive(Debug, PartialEq)]
//!     Foo {
//!         foo: Vec<usize> = (text of ".foo", collect),
//!     }
//! }
//!
//! fn main() {
//!     let input = r#"
//!         <div class="foo">1</div>
//!         <div class="foo">2</div>
//!         <div class="foo">3</div>
//!         <div class="foo">4</div>
//!     "#;
//!     let foo = Foo::extract_from_str(input).unwrap();
//!     assert_eq!(foo, Foo { foo: vec![1, 2, 3, 4] });
//! }
//! ```
//!
//! ## Extracting with regex
//! ```
//! use html_extractor::{html_extractor, HtmlExtractor};
//! html_extractor! {
//!     #[derive(Debug, PartialEq)]
//!     Foo {
//!         (foo: usize,) = (text of "#foo", capture with "^foo=(.*)$"),
//!     }
//! }
//!
//! fn main() {
//!     let input = r#"
//!         <div id="foo">foo=1</div>
//!     "#;
//!     let foo = Foo::extract_from_str(input).unwrap();
//!     assert_eq!(foo, Foo { foo: 1 });
//! }
//! ```

#[doc(hidden)]
pub extern crate lazy_static;
#[doc(hidden)]
pub extern crate regex;
#[doc(hidden)]
pub extern crate scraper;
pub use error::Error;
pub mod error;

/// Generates structures that implement [`HtmlExtractor`].
///
/// # Syntax
/// 
/// ## Defining structures
/// In this macro, zero or more structures can be defined.
///
/// Attributes can be attached to the structures, but currently attributes that may remove the structures (like `#[cfg]`) will not work.
/// ```no_run
/// # use html_extractor::html_extractor;
/// # fn main() {}
/// html_extractor! {
///     //private structure
///     Foo {
///         //fields...
///     }
///     //any visibilities and some attributes can be used
///     #[derive(Debug, Clone)]
///     pub(crate) Bar {
///         //fields...
///     }
/// }
/// ```
/// 
/// ## Defining fields in structures
/// There are two types of fields, "single field" and "tuple field".
/// Tuple fields are used to [capture data with regex](#capture-specifier).
/// 
/// Each field definition has a declaration part and an [extractor](#extractor-part-of-field-definitions) part.
/// 
/// Attributes can be attached to the fields, but currently attributes that may remove the fields (like `#[cfg]`) will not work.
/// ```no_run
/// # use html_extractor::html_extractor;
/// # fn main() {}
/// html_extractor! {
///     Foo {
///         //single field
///         pub foo: usize = (text of "#foo"),
///         //^^^^^^^^^^^^   ^^^^^^^^^^^^^^^^
///         // declaration   extractor
/// 
///         //tuple field
///         (pub bar: usize, pub baz: usize) = (text of "#bar-baz", capture with "bar=(.*),baz=(.*)"),
///         //^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
///         //                   declaration   extractor
///     }
/// }
/// ```
/// 
/// ## Extractor part of field definitions
/// The extractor part of field definitions specifies how to extract data from HTML.
/// Extractor consists of [Target](#target-specifier), [Capture](#capture-specifier), [Collector](#collector-specifier) and [Parser](#parser-specifier) specifier.
///
/// The order of specifiers does not matter. If the same specifier is written multiple times, the one given later applies.
/// ### Target specifier
/// Target specifier specifies a selector to select an element (or elements) and what of the selected element is extracted.
///
/// If the specified selector is invalid, it will be a compile error.  
/// If `text of ..` or `attr[..] of ..` is used, the type of field must implement [`FromStr`](std::str::FromStr).  
/// If `elem of ..` is used, the type of field must implement [`HtmlExtractor`].  
/// If `text of ..` is used, leading and trailing whitespace removed from the extracted string.  
/// If `presence of ..` is used, the type must be `bool` and any other specifier cannot be used,
/// ```
/// use html_extractor::{html_extractor, HtmlExtractor};
/// html_extractor! {
///     #[derive(Debug, PartialEq)]
///     Foo {
///         // extracts the first text node in the element that first matched the selector "#foo" 
///         foo: usize = (text of "#foo"),
///         // extracts the third text node in the element that first matched the selector "#bar"
///         bar: usize = (text[2] of "#bar"),
///         // extracts attribute "data-baz" in the element that first matched the selector "#baz"
///         baz: usize = (attr["data-baz"] of "#baz"),
///         // extracts an element that first matched the selector "#qux" and parse it with `HtmlExtractor::extract()`
///         qux: Qux = (elem of "#qux"),
///         // extracts inner HTML of the element that first matched the selector "#grault",
///         grault: String = (inner_html of "#grault"),
///         // stores if the elements that matches the selector "#garply" exist.
///         garply: bool = (presence of "#garply"),
///     }
///     #[derive(Debug, PartialEq)]
///     Qux {
///         corge: usize = (text of "#corge"),
///     }
/// }
///
/// fn main() {
///     let input = r#"
///         <div id="foo">1</div>
///         <div id="bar">ignore first<br>ignore second<br>2</div>
///         <div id="baz" data-baz="3"></div>
///         <div id="qux">
///             <div id="corge">4</div>
///         </div>
///         <div id="grault">
///             inner<br>html
///         </div>
///     "#;
///     let foo = Foo::extract_from_str(input).unwrap();
///     assert_eq!(foo, Foo {
///         foo: 1,
///         bar: 2,
///         baz: 3,
///         qux: Qux { corge: 4 },
///         grault: "inner<br>html".to_owned(),
///         garply: false,
///     });
/// }
/// ```
/// ### Capture specifier
/// Capture specifier specifies an regex that is used to capture desired data from the string that is extracted with target specifier.
///
/// The number of captures and the number of tuple elements must be the same.
/// 
/// If the specified regex is invalid, it will be a compile error.
///
/// It cannot be used with target specifier `elem of ..`.
///
/// If it is used without [collect specifier](#collect-specifier), the field must be a [tuple field](#defining-fields-in-structures).
/// If it is used with [collect specifier](#collect-specifier), the type of the field must be [`FromIterator`](std::iter::FromIterator) of tuple.
/// ```
/// use html_extractor::{html_extractor, HtmlExtractor};
/// html_extractor! {
///     #[derive(Debug, PartialEq)]
///     Foo {
///         // extracts a string from the first text node in the element that matches the selector "#foo-bar",
///         // and captures two data from the string with the regex "foo=(.*), bar=(.*)"
///         (foo: usize, bar: usize) = (text of "#foo-bar", capture with "foo=(.*), bar=(.*)"),
///         
///         // extracts strings from the first text node in all elements that matches the selector ".baz-qux-corge",
///         // captures three data from each string with the regex "baz=(.*), qux=(.*), corge=(.*)" ,
///         // and collects into `Vec<(usize, usize, usize)>`
///         baz_qux_corge: Vec<(usize, usize, usize)> = (text of ".baz-qux-corge", capture with "baz=(.*), qux=(.*), corge=(.*)", collect),
///     }
/// }
///
/// fn main() {
///     let input = r#"
///         <div id="foo-bar">foo=1, bar=2</div>
///
///         <div class="baz-qux-corge">baz=1, qux=2, corge=3</div>
///         <div class="baz-qux-corge">baz=4, qux=5, corge=6</div>
///         <div class="baz-qux-corge">baz=7, qux=8, corge=9</div>
///         <div class="baz-qux-corge">baz=10, qux=11, corge=12</div>
///     "#;
///     let foo = Foo::extract_from_str(input).unwrap();
///     assert_eq!(foo, Foo {
///         foo: 1,
///         bar: 2,
///         baz_qux_corge: vec![(1, 2, 3), (4, 5, 6), (7, 8, 9), (10, 11, 12)],
///     });
/// }
/// ```
///
/// ### Collector specifier
/// Collector specifier specifies how to collect HTML elements.  
/// The default collector is "first", which collects only the first matched element.  
/// The "collect" collector collects all the element into the type that implements [`FromIterator`](std::iter::FromIterator).  
/// The "optional" collector collects the first element if it exists. If not, it emits `None`.
/// ```
/// use html_extractor::{html_extractor, HtmlExtractor};
/// html_extractor! {
///     #[derive(Debug, PartialEq)]
///     Foo {
///         // extracts the first text node from each element that matches the selector ".foo", and collect them into `Vec<usize>`.
///         foo: Vec<usize> = (text of ".foo", collect),
///
///         // extracts all the elements that match that selector "#bar",
///         // parses them with `HtmlExtractor::extract()`,
///         // and collects into `Vec<Bar>`.
///         bar: Vec<Bar> = (elem of "#bar", collect),
///         
///         // extracts strings from the first text node in all elements that matches the selector ".baz-qux-corge",
///         // captures three data from each string with the regex "baz=(.*), qux=(.*), corge=(.*)" ,
///         // and collects into `Vec<(usize, usize, usize)>`
///         baz_qux_corge: Vec<(usize, usize, usize)> = (text of ".baz-qux-corge", capture with "baz=(.*), qux=(.*), corge=(.*)", collect),
/// 
///         // optionally extracts the first text node in the first element that matches the selector ".grault".
///         grault: Option<usize> = (text of ".grault", optional),
///     }
///     #[derive(Debug, PartialEq)]
///     Bar {
///         bar: usize = (text of ".bar-data"),
///     }
/// }
///
/// fn main() {
///     let input = r#"
///         <div class="foo">1</div>
///         <div class="foo">2</div>
///         <div class="foo">3</div>
///         <div class="foo">4</div>
///
///         <div id="bar"><div class="bar-data">1</div></div>
///         <div id="bar"><div class="bar-data">2</div></div>
///         <div id="bar"><div class="bar-data">3</div></div>
///         <div id="bar"><div class="bar-data">4</div></div>
///
///         <div class="baz-qux-corge">baz=1, qux=2, corge=3</div>
///         <div class="baz-qux-corge">baz=4, qux=5, corge=6</div>
///         <div class="baz-qux-corge">baz=7, qux=8, corge=9</div>
///         <div class="baz-qux-corge">baz=10, qux=11, corge=12</div>
///     "#;
///     let foo = Foo::extract_from_str(input).unwrap();
///     assert_eq!(foo, Foo {
///         foo: vec![1, 2, 3, 4],
///         bar: vec![
///             Bar { bar: 1 },
///             Bar { bar: 2 },
///             Bar { bar: 3 },
///             Bar { bar: 4 },
///         ],
///         baz_qux_corge: vec![(1, 2, 3), (4, 5, 6), (7, 8, 9), (10, 11, 12)],
///         grault: None,
///     });
/// }
/// ```
/// ### Parser specifier
/// Parser specifier specifies the parser used to parse the extracted string.  
/// The default parser is [`::std::str::FromStr::from_str`].  
/// The parser must be `Fn(&str) -> Result<_, T> where T: std::fmt::Debug`
/// ```
/// use html_extractor::{html_extractor, HtmlExtractor};
/// html_extractor! {
///     #[derive(Debug, PartialEq)]
///     Foo {
///         // extracts using a custom parser.
///         foo: usize = (text of "#foo", parse with custom_parser),
///     }
/// }
/// fn custom_parser(input: &str) -> Result<usize, std::num::ParseIntError> {
///     input.replace(",", "").parse()
/// }
///
/// fn main() {
///     let input = r#"
///         <div id="foo">1,000,000,000</div>
///     "#;
///     let foo = Foo::extract_from_str(input).unwrap();
///     assert_eq!(foo, Foo {
///         foo: 1000000000,
///     });
/// }
/// ```
/// 
/// # Usage of the generated structures
/// The generated structures implement trait [`HtmlExtractor`].
/// See the document of the trait.
pub use html_extractor_macros::html_extractor;

/// A trait for extracting data from HTML documents.
/// 
/// It is recommended to use [`html_extractor!`](macro.html_extractor.html) to implement `HtmlExtractor`.
pub trait HtmlExtractor
where
    Self: Sized,
{
    /// Extracts data from [`scraper::element_ref::ElementRef`].
    fn extract(elem: &scraper::ElementRef) -> Result<Self, Error>;
    /// Parses HTML string and extracts data from it.
    fn extract_from_str(html_str: &str) -> Result<Self, Error> {
        let html = scraper::Html::parse_document(html_str);
        HtmlExtractor::extract(&html.root_element())
    }
}

#[cfg(test)]
mod test;
