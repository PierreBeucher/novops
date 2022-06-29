use std::collections::HashMap;
use serde::Deserialize;

use crate::bitwarden;
use crate::aws;

#[derive(Debug, Deserialize)]
pub struct NovopsConfig {
    pub name: String,
    pub environments: HashMap<String, NovopsEnvironment>,
    pub default: Option<NovopsConfigDefault>
}

#[derive(Debug, Deserialize)]
pub struct NovopsConfigDefault {
    pub environment: Option<String>
}

#[derive(Debug, Deserialize)]
pub struct NovopsEnvironment {
    pub variables: HashMap<String, NovopsVariable>,
    pub files: HashMap<String, NovopsFile>,
    pub aws: Option<aws::NovopsAws>
}

#[derive(Debug, Deserialize)]
pub struct NovopsFile {
    pub dest: Option<String>,
    pub variable: Option<String>,
    pub content: NovopsValue
}

type NovopsVariable = NovopsValue;

/**
 * A resolved file, with known destination and content
 */
#[derive(Debug)]
pub struct ResolvedNovopsFile {
    pub dest: String,
    pub variable: ResolvedNovopsVariable,
    pub content: String // TODO buffer? content may be long
}

/**
 * A resolved variable, with known name and value
 */
#[derive(Debug, Clone)]
pub struct ResolvedNovopsVariable {
    pub name: String,
    pub value: String
}

/**
 * A Novops value is it's core: in can be a string, a secret, or something else
 */
#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[enum_dispatch(ResolvableNovopsValue)]
pub enum NovopsValue {
    String(String), 
    StringValue(StringValue),
    BitwardenItem(bitwarden::BitwardenItem)
}

#[enum_dispatch]
pub trait ResolvableNovopsValue {
    fn resolve(&self) -> String;
}

impl ResolvableNovopsValue for String {
    fn resolve(&self) -> String {
        return self.clone()
    }
}

/**
 * A string set with 'value' key such as 
 * 
 * myvar:
 *   value: foo
 */

#[derive(Debug, Deserialize)]
pub struct StringValue{
    value: String
}

impl ResolvableNovopsValue for StringValue {
    fn resolve(&self) -> String {
        return self.value.clone();
    }
}
