#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("socinfo parsing error: `{0}`")]
    Parse(String),

    #[error("I/O error: `{source}`")]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[error("utf8 conversion error: `{source}`")]
    Utf8Conversion {
        #[from]
        source: std::string::FromUtf8Error,
    },

    #[error("integer parsing error: `{source}`")]
    ParseInt {
        #[from]
        source: std::num::ParseIntError,
    },
}
