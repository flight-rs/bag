use std::path::PathBuf;

use mime::Mime;
use uri::Uri;
use quote::Tokens;

pub enum ArtifactSource {
    Uri(Uri),

}

pub trait Artifact {
    /// Create an expression that evalutates to `Result<impl std::io::Read, failure::Error>`.
    fn expr_result_read(&self) -> Tokens;

    /// Content type of artifact
    fn content_type(&self) -> Mime;
}
