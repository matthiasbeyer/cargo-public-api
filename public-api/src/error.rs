use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
/// Enumerates all errors that can currently occur within this crate.
pub enum Error {
    /// Occurs if the rustdoc JSON you provide can't be parsed. Typically
    /// because the rustdoc JSON format that your version of nightly outputs is
    /// too old. Consult the "Compatibility matrix" in the README.
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
}

/// Shorthand for [`std::result::Result<T, public_api::Error>`].
pub type Result<T> = std::result::Result<T, Error>;
