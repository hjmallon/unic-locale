mod errors;
mod layout_table;
#[cfg(feature = "likelysubtags")]
pub mod likelysubtags;
#[doc(hidden)]
pub mod parser;
mod subtags;

pub use crate::errors::LanguageIdentifierError;
use layout_table::CHARACTER_DIRECTION_RTL;
use std::iter::Peekable;
use std::str::FromStr;

use tinystr::{TinyStr4, TinyStr8};

/// Enum representing available character direction orientations.
#[derive(Debug, PartialEq)]
pub enum CharacterDirection {
    /// Right To Left
    ///
    /// Used in languages such as Arabic, Hebrew, Fula, Kurdish etc.
    RTL,
    /// Left To Right
    ///
    /// Used in languages such as French, Spanish, English, German etc.
    LTR,
}

/// `LanguageIdentifier` is a core struct representing a Unicode Language Identifier.
///
/// # Examples
///
/// ```
/// use unic_langid_impl::LanguageIdentifier;
///
/// let li: LanguageIdentifier = "en-US".parse()
///     .expect("Failed to parse.");
///
/// assert_eq!(li.get_language(), "en");
/// assert_eq!(li.get_script(), None);
/// assert_eq!(li.get_region(), Some("US"));
/// assert_eq!(li.get_variants().len(), 0);
/// ```
///
/// # Parsing
///
/// Unicode recognizes three levels of standard conformance for any language identifier:
///
///  * *well-formed* - syntactically correct
///  * *valid* - well-formed and only uses registered language subtags, extensions, keywords, types...
///  * *canonical* - valid and no deprecated codes or structure.
///
/// At the moment parsing normalizes a well-formed language identifier converting
/// `_` separators to `-` and adjusting casing to conform to the Unicode standard.
///
/// Any bogus subtags will cause the parsing to fail with an error.
/// No subtag validation is performed.
///
/// # Eamples:
///
/// ```
/// use unic_langid_impl::LanguageIdentifier;
///
/// let li: LanguageIdentifier = "eN_latn_Us-Valencia".parse()
///     .expect("Failed to parse.");
///
/// assert_eq!(li.get_language(), "en");
/// assert_eq!(li.get_script(), Some("Latn"));
/// assert_eq!(li.get_region(), Some("US"));
/// assert_eq!(li.get_variants(), &["valencia"]);
/// ```
#[derive(Default, Debug, PartialEq, Eq, Clone, Hash)]
pub struct LanguageIdentifier {
    language: Option<TinyStr8>,
    script: Option<TinyStr4>,
    region: Option<TinyStr4>,
    // We store it as an Option to allow for const constructor.
    // Once const constructor for Box::new stabilizes, we can remove this.
    variants: Option<Box<[TinyStr8]>>,
}

impl LanguageIdentifier {
    /// A constructor which takes optional subtags as `&str`, parses them and
    /// produces a well-formed `LanguageIdentifier`.
    ///
    /// # Examples
    ///
    /// ```
    /// use unic_langid_impl::LanguageIdentifier;
    ///
    /// let li = LanguageIdentifier::from_parts(Some("fr"), None, Some("CA"), &[])
    ///     .expect("Parsing failed.");
    ///
    /// assert_eq!(li.to_string(), "fr-CA");
    /// ```
    pub fn from_parts<S: AsRef<str>>(
        language: Option<S>,
        script: Option<S>,
        region: Option<S>,
        variants: &[S],
    ) -> Result<Self, LanguageIdentifierError> {
        let language = if let Some(subtag) = language {
            subtags::parse_language_subtag(subtag.as_ref())?
        } else {
            None
        };
        let script = if let Some(subtag) = script {
            Some(subtags::parse_script_subtag(subtag.as_ref())?)
        } else {
            None
        };
        let region = if let Some(subtag) = region {
            Some(subtags::parse_region_subtag(subtag.as_ref())?)
        } else {
            None
        };

        let variants = if !variants.is_empty() {
            let mut vars = variants
                .into_iter()
                .map(|v| subtags::parse_variant_subtag(v.as_ref()))
                .collect::<Result<Vec<TinyStr8>, parser::errors::ParserError>>()?;
            vars.sort();
            vars.dedup();
            Some(vars.into_boxed_slice())
        } else {
            None
        };

        Ok(Self {
            language,
            script,
            region,
            variants,
        })
    }

    #[doc(hidden)]
    /// This method is used by `unic-locale` to handle partial
    /// subtag iterator.
    ///
    /// Not stable.
    pub fn try_from_iter<'a>(
        iter: &mut Peekable<impl Iterator<Item = &'a str>>,
        allow_extension: bool,
    ) -> Result<LanguageIdentifier, LanguageIdentifierError> {
        parser::parse_language_identifier_from_iter(iter, allow_extension)
            .map_err(std::convert::Into::into)
    }

    /// Consumes `LanguageIdentifier` and produces raw internal representations
    /// of all subtags in form of `u64`/`u32`.
    ///
    /// Primarily used for storing internal representation and restoring via
    /// an unsafe `from_raw_parts_unchecked`.
    ///
    /// # Examples
    ///
    /// ```
    /// use unic_langid_impl::LanguageIdentifier;
    /// use tinystr::{TinyStr8, TinyStr4};
    ///
    /// let li: LanguageIdentifier = "en-US".parse()
    ///     .expect("Parsing failed.");
    ///
    /// let (lang, script, region, variants) = li.into_raw_parts();
    ///
    /// let li2 = unsafe { LanguageIdentifier::from_raw_parts_unchecked(
    ///     lang.map(|l| TinyStr8::new_unchecked(l)),
    ///     script.map(|s| TinyStr4::new_unchecked(s)),
    ///     region.map(|r| TinyStr4::new_unchecked(r)),
    ///     variants.map(|v| v.into_iter().map(|v| TinyStr8::new_unchecked(*v)).collect()),
    /// ) };
    ///
    /// assert_eq!(li2.to_string(), "en-US");
    /// ```
    pub fn into_raw_parts(self) -> (Option<u64>, Option<u32>, Option<u32>, Option<Box<[u64]>>) {
        (
            self.language.map(|l| l.into()),
            self.script.map(|s| s.into()),
            self.region.map(|r| r.into()),
            self.variants
                .map(|v| v.iter().map(|v| (*v).into()).collect()),
        )
    }

    /// Consumes raw representation of subtags generating new `LanguageIdentifier`
    /// without any checks.
    ///
    /// Primarily used for restoring internal representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use unic_langid_impl::LanguageIdentifier;
    /// use tinystr::{TinyStr8, TinyStr4};
    ///
    /// let li: LanguageIdentifier = "en-US".parse()
    ///     .expect("Parsing failed.");
    ///
    /// let (lang, script, region, variants) = li.into_raw_parts();
    ///
    /// let li2 = unsafe { LanguageIdentifier::from_raw_parts_unchecked(
    ///     lang.map(|l| TinyStr8::new_unchecked(l)),
    ///     script.map(|s| TinyStr4::new_unchecked(s)),
    ///     region.map(|r| TinyStr4::new_unchecked(r)),
    ///     variants.map(|v| v.into_iter().map(|v| TinyStr8::new_unchecked(*v)).collect()),
    /// ) };
    ///
    /// assert_eq!(li2.to_string(), "en-US");
    /// ```
    #[inline(always)]
    pub const unsafe fn from_raw_parts_unchecked(
        language: Option<TinyStr8>,
        script: Option<TinyStr4>,
        region: Option<TinyStr4>,
        variants: Option<Box<[TinyStr8]>>,
    ) -> Self {
        Self {
            language,
            script,
            region,
            variants,
        }
    }

    /// Compares a `LanguageIdentifier` to another `AsRef<LanguageIdentifier`
    /// allowing for either side to use the missing fields as wildcards.
    ///
    /// This allows for matching between `en` (treated as `en-*-*-*`) and `en-US`.
    ///
    /// # Examples
    ///
    /// ```
    /// use unic_langid_impl::LanguageIdentifier;
    ///
    /// let li1: LanguageIdentifier = "en".parse()
    ///     .expect("Parsing failed.");
    ///
    /// let li2: LanguageIdentifier = "en-US".parse()
    ///     .expect("Parsing failed.");
    ///
    /// assert_ne!(li1, li2); // "en" != "en-US"
    /// assert_ne!(li1.to_string(), li2.to_string()); // "en" != "en-US"
    ///
    /// assert_eq!(li1.matches(&li2, false, false), false); // "en" != "en-US"
    /// assert_eq!(li1.matches(&li2, true, false), true); // "en-*-*-*" == "en-US"
    /// assert_eq!(li1.matches(&li2, false, true), false); // "en" != "en-*-US-*"
    /// assert_eq!(li1.matches(&li2, true, true), true); // "en-*-*-*" == "en-*-US-*"
    /// ```
    pub fn matches<O: AsRef<Self>>(
        &self,
        other: &O,
        self_as_range: bool,
        other_as_range: bool,
    ) -> bool {
        let other = other.as_ref();
        subtag_matches(
            &self.language,
            &other.language,
            self_as_range,
            other_as_range,
        ) && subtag_matches(&self.script, &other.script, self_as_range, other_as_range)
            && subtag_matches(&self.region, &other.region, self_as_range, other_as_range)
            && subtags_match(
                &self.variants,
                &other.variants,
                self_as_range,
                other_as_range,
            )
    }

    /// Returns the language subtag of the `LanguageIdentifier`.
    ///
    /// If the language is empty, `"und"` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use unic_langid_impl::LanguageIdentifier;
    ///
    /// let li1: LanguageIdentifier = "de-AT".parse()
    ///     .expect("Parsing failed.");
    ///
    /// assert_eq!(li1.get_language(), "de");
    ///
    /// let li2: LanguageIdentifier = "und-AT".parse()
    ///     .expect("Parsing failed.");
    ///
    /// assert_eq!(li2.get_language(), "und");
    /// ```
    pub fn get_language(&self) -> &str {
        self.language.as_ref().map(|s| s.as_ref()).unwrap_or("und")
    }

    /// Sets the language subtag of the `LanguageIdentifier`.
    ///
    /// If `None` is passed, the field will be set to `None` and returned as `"und"`.
    ///
    /// # Examples
    ///
    /// ```
    /// use unic_langid_impl::LanguageIdentifier;
    ///
    /// let mut li: LanguageIdentifier = "de-Latn-AT".parse()
    ///     .expect("Parsing failed.");
    ///
    /// li.set_language(Some("fr"));
    ///
    /// assert_eq!(li.to_string(), "fr-Latn-AT");
    /// ```
    pub fn set_language(&mut self, language: Option<&str>) -> Result<(), LanguageIdentifierError> {
        self.language = if let Some(lang) = language {
            subtags::parse_language_subtag(lang)?
        } else {
            None
        };
        Ok(())
    }

    /// Returns the script subtag of the `LanguageIdentifier`, if set.
    ///
    /// # Examples
    ///
    /// ```
    /// use unic_langid_impl::LanguageIdentifier;
    ///
    /// let li1: LanguageIdentifier = "de-Latn-AT".parse()
    ///     .expect("Parsing failed.");
    ///
    /// assert_eq!(li1.get_script(), Some("Latn"));
    ///
    /// let li2: LanguageIdentifier = "de-AT".parse()
    ///     .expect("Parsing failed.");
    ///
    /// assert_eq!(li2.get_script(), None);
    /// ```
    pub fn get_script(&self) -> Option<&str> {
        self.script.as_ref().map(|s| s.as_ref())
    }

    /// Sets the script subtag of the `LanguageIdentifier`.
    ///
    /// # Examples
    ///
    /// ```
    /// use unic_langid_impl::LanguageIdentifier;
    ///
    /// let mut li: LanguageIdentifier = "sr-Latn".parse()
    ///     .expect("Parsing failed.");
    ///
    /// li.set_script(Some("Cyrl"));
    ///
    /// assert_eq!(li.to_string(), "sr-Cyrl");
    /// ```
    pub fn set_script(&mut self, script: Option<&str>) -> Result<(), LanguageIdentifierError> {
        self.script = if let Some(script) = script {
            Some(subtags::parse_script_subtag(script)?)
        } else {
            None
        };
        Ok(())
    }

    /// Returns the region subtag of the `LanguageIdentifier`, if set.
    ///
    /// # Examples
    ///
    /// ```
    /// use unic_langid_impl::LanguageIdentifier;
    ///
    /// let li1: LanguageIdentifier = "de-Latn-AT".parse()
    ///     .expect("Parsing failed.");
    ///
    /// assert_eq!(li1.get_region(), Some("AT"));
    ///
    /// let li2: LanguageIdentifier = "de".parse()
    ///     .expect("Parsing failed.");
    ///
    /// assert_eq!(li2.get_region(), None);
    /// ```
    pub fn get_region(&self) -> Option<&str> {
        self.region.as_ref().map(|s| s.as_ref())
    }

    /// Sets the region subtag of the `LanguageIdentifier`.
    ///
    /// # Examples
    ///
    /// ```
    /// use unic_langid_impl::LanguageIdentifier;
    ///
    /// let mut li: LanguageIdentifier = "fr-FR".parse()
    ///     .expect("Parsing failed.");
    ///
    /// li.set_region(Some("CA"));
    ///
    /// assert_eq!(li.to_string(), "fr-CA");
    /// ```
    pub fn set_region(&mut self, region: Option<&str>) -> Result<(), LanguageIdentifierError> {
        self.region = if let Some(region) = region {
            Some(subtags::parse_region_subtag(region)?)
        } else {
            None
        };
        Ok(())
    }

    /// Returns a vector of variants subtags of the `LanguageIdentifier`.
    ///
    /// # Examples
    ///
    /// ```
    /// use unic_langid_impl::LanguageIdentifier;
    ///
    /// let li1: LanguageIdentifier = "ca-ES-valencia".parse()
    ///     .expect("Parsing failed.");
    ///
    /// assert_eq!(li1.get_variants(), &["valencia"]);
    ///
    /// let li2: LanguageIdentifier = "de".parse()
    ///     .expect("Parsing failed.");
    ///
    /// assert_eq!(li2.get_variants().len(), 0);
    /// ```
    pub fn get_variants(&self) -> Vec<&str> {
        if let Some(variants) = &self.variants {
            variants.iter().map(|s| s.as_ref()).collect()
        } else {
            vec![]
        }
    }

    /// Sets variant subtags of the `LanguageIdentifier`.
    ///
    /// # Examples
    ///
    /// ```
    /// use unic_langid_impl::LanguageIdentifier;
    ///
    /// let mut li: LanguageIdentifier = "ca-ES".parse()
    ///     .expect("Parsing failed.");
    ///
    /// li.set_variants(&["valencia"]);
    ///
    /// assert_eq!(li.to_string(), "ca-ES-valencia");
    /// ```
    pub fn set_variants(&mut self, variants: &[&str]) -> Result<(), LanguageIdentifierError> {
        if variants.is_empty() {
            self.variants = None;
        } else {
            let mut result = variants
                .into_iter()
                .map(|v| subtags::parse_variant_subtag(v.as_ref()))
                .collect::<Result<Vec<TinyStr8>, parser::errors::ParserError>>()?;
            result.sort();
            result.dedup();
            self.variants = Some(result.into_boxed_slice());
        }
        Ok(())
    }

    /// Extends the `LanguageIdentifier` adding likely subtags based
    /// on tables provided by CLDR.
    ///
    /// # Examples
    ///
    /// ```
    /// use unic_langid_impl::LanguageIdentifier;
    ///
    /// let mut li: LanguageIdentifier = "en-US".parse()
    ///     .expect("Parsing failed.");
    ///
    /// assert_eq!(li.add_likely_subtags(), true);
    /// assert_eq!(li.to_string(), "en-Latn-US");
    /// ```
    #[cfg(feature = "likelysubtags")]
    pub fn add_likely_subtags(&mut self) -> bool {
        if let Some(new_li) =
            likelysubtags::add_likely_subtags(self.language, self.script, self.region)
        {
            self.language = new_li.0;
            self.script = new_li.1;
            self.region = new_li.2;
            return true;
        } else {
            return false;
        }
    }

    /// Extends the `LanguageIdentifier` removing likely subtags based
    /// on tables provided by CLDR.
    ///
    /// # Examples
    ///
    /// ```
    /// use unic_langid_impl::LanguageIdentifier;
    ///
    /// let mut li: LanguageIdentifier = "en-Latn-US".parse()
    ///     .expect("Parsing failed.");
    ///
    /// assert_eq!(li.remove_likely_subtags(), true);
    /// assert_eq!(li.to_string(), "en");
    /// ```
    #[cfg(feature = "likelysubtags")]
    pub fn remove_likely_subtags(&mut self) -> bool {
        if let Some(new_li) =
            likelysubtags::remove_likely_subtags(self.language, self.script, self.region)
        {
            self.language = new_li.0;
            self.script = new_li.1;
            self.region = new_li.2;
            return true;
        } else {
            return false;
        }
    }

    /// Returns character direction of the `LanguageIdentifier`.
    ///
    /// # Examples
    ///
    /// ```
    /// use unic_langid_impl::{LanguageIdentifier, CharacterDirection};
    ///
    /// let li1: LanguageIdentifier = "es-AR".parse()
    ///     .expect("Parsing failed.");
    /// let li2: LanguageIdentifier = "fa".parse()
    ///     .expect("Parsing failed.");
    ///
    /// assert_eq!(li1.get_character_direction(), CharacterDirection::LTR);
    /// assert_eq!(li2.get_character_direction(), CharacterDirection::RTL);
    /// ```
    pub fn get_character_direction(&self) -> CharacterDirection {
        match self.language {
            Some(lang) if CHARACTER_DIRECTION_RTL.contains(&(lang.into())) => {
                CharacterDirection::RTL
            }
            _ => CharacterDirection::LTR,
        }
    }
}

impl FromStr for LanguageIdentifier {
    type Err = LanguageIdentifierError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        parser::parse_language_identifier(source).map_err(std::convert::Into::into)
    }
}

impl AsRef<LanguageIdentifier> for LanguageIdentifier {
    #[inline(always)]
    fn as_ref(&self) -> &LanguageIdentifier {
        self
    }
}

impl std::fmt::Display for LanguageIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut subtags = vec![self.get_language()];
        if let Some(script) = self.get_script() {
            subtags.push(script);
        }
        if let Some(region) = self.get_region() {
            subtags.push(region);
        }
        if let Some(variants) = &self.variants {
            for variant in variants.iter() {
                subtags.push(variant);
            }
        }

        f.write_str(&subtags.join("-"))
    }
}

fn subtag_matches<P: PartialEq>(
    subtag1: &Option<P>,
    subtag2: &Option<P>,
    as_range1: bool,
    as_range2: bool,
) -> bool {
    (as_range1 && subtag1.is_none()) || (as_range2 && subtag2.is_none()) || subtag1 == subtag2
}

fn is_option_empty<P: PartialEq>(subtag: &Option<Box<[P]>>) -> bool {
    subtag.as_ref().map(|t| t.is_empty()).unwrap_or(true)
}

fn subtags_match<P: PartialEq>(
    subtag1: &Option<Box<[P]>>,
    subtag2: &Option<Box<[P]>>,
    as_range1: bool,
    as_range2: bool,
) -> bool {
    // or is some and is empty!
    (as_range1 && is_option_empty(subtag1))
        || (as_range2 && is_option_empty(subtag2))
        || subtag1 == subtag2
}

/// This is a best-effort operation that performs all available levels of canonicalization.
///
/// At the moment the operation will normalize casing and the separator, but in the future
/// it may also validate and update from deprecated subtags to canonical ones.
///
/// # Examples
///
/// ```
/// use unic_langid_impl::canonicalize;
///
/// assert_eq!(canonicalize("pL_latn_pl"), Ok("pl-Latn-PL".to_string()));
/// ```
pub fn canonicalize(input: &str) -> Result<String, LanguageIdentifierError> {
    let lang_id: LanguageIdentifier = input.parse()?;
    Ok(lang_id.to_string())
}
