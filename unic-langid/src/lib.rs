//! `unic-langid` is a core API for parsing, manipulating, and serializing Unicode Language
//! Identifiers.
//!
//! The crate provides algorithms for parsing a string into a well-formed language identifier
//! as defined by [`UTS #35: Unicode LDML 3.1 Unicode Language Identifier`].
//!
//! # Examples
//!
//! ```
//! use unic_langid::LanguageIdentifier;
//!
//! let mut li: LanguageIdentifier = "en-US".parse()
//!     .expect("Failed to parse.");
//!
//! assert_eq!(li.get_language(), "en");
//! assert_eq!(li.get_script(), None);
//! assert_eq!(li.get_region(), Some("US"));
//! assert_eq!(li.get_variants().len(), 0);
//!
//! li.set_region(Some("GB"))
//!     .expect("Region parsing failed.");
//!
//! assert_eq!(li.to_string(), "en-GB");
//! ```
//!
//! For more details, see [`LanguageIdentifier`].
//!
//! # Optional features
//!
//! ## `langid!` and `langids!` macros
//!
//! If `feature = "macros"` is selected, the crate provides a procedural macro
//! which allows to construct build-time validated language identifiers with zero-cost at runtime.
//!
//! ``` ignore
//! use unic_langid::{langid, langids};
//!
//! let es_ar = langid!("es-AR");
//! let en_us = langid!("en-US");
//!
//! assert_eq!(es_ar, "es-AR");
//! assert_eq!(en_us, "en-US");
//!
//! let lang_ids = langids!("es-AR", "en-US", "de");
//!
//! assert_eq!(lang_ids.get(0), "es-AR");
//! assert_eq!(lang_ids.get(1), "en-US");
//! assert_eq!(lang_ids.get(2), "de");
//! ```
//!
//! The macros produce instances of `LanguageIdentifier` the same way as parsing from `&str` does,
//! but since the parsing is performed at build time, it doesn't need a `Result`.
//!
//! At the moment `langid!` can also be used for const variables, but only if no variants are used.
//!
//! The macros are optional to reduce the dependency chain and compilation time of `unic-langid`.
//!
//! ## Likely Subtags
//!
//! If `feature = "likelysubtags"` is selected, the `LanguageIdentifier` gains two more methods:
//!
//!  * add_likely_subtags
//!  * remove_likely_subtags
//!
//! Both of them operate in place updating the existing `LanguageIdentifier` by either extending
//! subtags to most likely values, or removing the subtags that are not needed.
//!
//! Both methods return a `bool` that indicates if the identifier has been modified.
//!
//! ``` ignore
//! use unic_langid::LanuageIdentifier;
//!
//! let mut li: LanguageIdentifier = "fr-FR".parse()
//!     .expect("Parsing failed.");
//!
//! assert_eq!(li.add_likely_subtags(), true);
//! assert_eq!(li, "fr-Latn-FR");
//!
//! assert_eq!(li.remove_likely_subtags(), true);
//! assert_eq!(li, "fr");
//! ```
//!
//! The feature is optional because it increases the binary size of the library by including
//! a data table for CLDR likelySubtags.
//!
//! [`UTS #35: Unicode LDML 3.1 Unicode Language Identifier`]: https://unicode.org/reports/tr35/tr35.html#Unicode_language_identifier
//! [`LanguageIdentifier`]: ./struct.LanguageIdentifier.html

pub use unic_langid_impl::*;

#[cfg(feature = "unic-langid-macros")]
pub use unic_langid_macros::langid;

#[cfg(feature = "unic-langid-macros")]
#[macro_export]
macro_rules! langids {
    ( $($langid:expr),* ) => {
        {
            let mut v = vec![];
            $(
                v.push(langid!($langid));
            )*
            v
        }
    };
}
