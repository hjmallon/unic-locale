pub mod errors;
pub mod parser;
pub mod subtags;

#[cfg(feature = "validity")]
pub mod validity;

use crate::errors::LanguageIdentifierError;
use std::str::FromStr;

use tinystr::{TinyStr4, TinyStr8};

#[derive(Default, Debug, PartialEq, Eq, Clone, Hash)]
pub struct LanguageIdentifier {
    language: Option<TinyStr8>,
    script: Option<TinyStr4>,
    region: Option<TinyStr4>,
    variants: Box<[TinyStr8]>,
    is_valid: Option<bool>,
}

impl LanguageIdentifier {
    pub fn from_str(input: &str, allow_extensions: bool) -> Result<Self, LanguageIdentifierError> {
        parser::parse_language_identifier(input, allow_extensions)
            .map_err(std::convert::Into::into)
            .map(|(langid, _)| langid)
    }

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

        let mut vars = Vec::with_capacity(variants.len());
        for variant in variants {
            vars.push(subtags::parse_variant_subtag(variant.as_ref())?);
        }
        vars.sort();
        vars.dedup();

        Ok(Self {
            language,
            script,
            region,
            variants: vars.into_boxed_slice(),
            is_valid: None,
        })
    }

    pub fn to_raw_parts(self) -> (Option<u64>, Option<u32>, Option<u32>, Box<[u64]>, Option<bool>) {
        (
            self.language.map(|l| l.into()),
            self.script.map(|s| s.into()),
            self.region.map(|r| r.into()),
            self.variants.into_iter().map(|v| (*v).into()).collect(),
            self.is_valid,
        )
    }

    #[inline(always)]
    pub const unsafe fn from_raw_parts_unchecked(
        language: Option<TinyStr8>,
        script: Option<TinyStr4>,
        region: Option<TinyStr4>,
        variants: Box<[TinyStr8]>,
        is_valid: Option<bool>,
    ) -> Self {
        Self {
            language,
            script,
            region,
            variants,
            is_valid,
        }
    }

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

    pub fn get_language(&self) -> &str {
        self.language.as_ref().map(|s| s.as_ref()).unwrap_or("und")
    }

    pub fn set_language(&mut self, language: Option<&str>) -> Result<(), LanguageIdentifierError> {
        self.language = if let Some(lang) = language {
            subtags::parse_language_subtag(lang)?
        } else {
            None
        };
        Ok(())
    }

    pub fn get_script(&self) -> Option<&str> {
        self.script.as_ref().map(|s| s.as_ref())
    }

    pub fn set_script(&mut self, script: Option<&str>) -> Result<(), LanguageIdentifierError> {
        self.script = if let Some(script) = script {
            Some(subtags::parse_script_subtag(script)?)
        } else {
            None
        };
        Ok(())
    }

    pub fn get_region(&self) -> Option<&str> {
        self.region.as_ref().map(|s| s.as_ref())
    }

    pub fn set_region(&mut self, region: Option<&str>) -> Result<(), LanguageIdentifierError> {
        self.region = if let Some(region) = region {
            Some(subtags::parse_region_subtag(region)?)
        } else {
            None
        };
        Ok(())
    }

    pub fn get_variants(&self) -> Vec<&str> {
        self.variants.iter().map(|s| s.as_ref()).collect()
    }

    pub fn set_variants(&mut self, variants: &[&str]) -> Result<(), LanguageIdentifierError> {
        let mut result = Vec::with_capacity(variants.len());
        for variant in variants {
            result.push(subtags::parse_variant_subtag(variant)?);
        }
        result.sort();
        result.dedup();

        self.variants = result.into_boxed_slice();
        Ok(())
    }

    pub fn canonicalize_str(input: &str) -> Result<String, LanguageIdentifierError> {
        let lang_id: LanguageIdentifier = input.parse()?;
        Ok(lang_id.to_string())
    }

    pub fn is_str_well_formed(input: &str) -> bool {
        // XXX: We could optimize it by extracting the testing
        //      fns out of the parser.
        input.parse::<LanguageIdentifier>().is_ok()
    }
}

impl FromStr for LanguageIdentifier {
    type Err = LanguageIdentifierError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        parser::parse_language_identifier(source, false)
            .map_err(std::convert::Into::into)
            .map(|(langid, _)| langid)
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
        for variant in self.variants.iter() {
            subtags.push(variant);
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

fn subtags_match<P: PartialEq>(
    subtag1: &[P],
    subtag2: &[P],
    as_range1: bool,
    as_range2: bool,
) -> bool {
    (as_range1 && subtag1.is_empty()) || (as_range2 && subtag2.is_empty()) || subtag1 == subtag2
}
