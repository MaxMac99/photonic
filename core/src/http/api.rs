use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Error {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<HashMap<String, String>>,
}

const EXTENSION_KEY_CODE: &str = "code";
const CODE_NOT_FOUND: &str = "NOT_FOUND";
const CODE_INTERNAL: &str = "INTERNAL";

impl From<crate::Error> for Error {
    fn from(err: crate::Error) -> Self {
        match err {
            crate::Error::NotFound(err) => {
                let mut extensions = HashMap::new();
                extensions.insert(EXTENSION_KEY_CODE.into(), CODE_NOT_FOUND.into());

                Error {
                    message: err.to_string(),
                    extensions: Some(extensions),
                }
            }
            crate::Error::Internal(_) => {
                let mut extensions = HashMap::new();
                extensions.insert(EXTENSION_KEY_CODE.into(), CODE_INTERNAL.into());

                Error {
                    message: err.to_string(),
                    extensions: Some(extensions),
                }
            }
            _ => Error {
                message: err.to_string(),
                extensions: None,
            },
        }
    }
}