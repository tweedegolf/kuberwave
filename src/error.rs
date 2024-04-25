use failure::{Backtrace, Context, Fail};
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

impl Error {
    pub fn create<D: Display + Debug + Sync + Send + 'static>(message: D, kind: ErrorKind) -> Self {
        ::failure::err_msg(message).context(kind).into()
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> ::std::fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
    /// Could not load deploy token.
    #[fail(display = "Could not load deploy token.")]
    TokenError,

    /// Environment is incomplete.
    #[fail(display = "Environment is incomplete, missing '{}'.", name)]
    EnvError { name: std::string::String },

    /// Failed to process the manifest file.
    #[fail(display = "Failed to process the manifest file.")]
    ManifestError,

    /// Failed to process the inventory file.
    #[fail(display = "Failed to process the inventory file.")]
    InventoryError,

    /// Failed to populate a context.
    #[fail(display = "Failed to populate a context.")]
    ContextError,

    /// Failed to parse file.
    #[fail(display = "Failed parse file {:?}.", name)]
    ParseError { name: std::path::PathBuf },

    /// Failed to process template.
    #[fail(display = "Failed to process template {:?}.", name)]
    TemplateError { name: std::path::PathBuf },

    /// Failed to read file.
    #[fail(display = "Failed to read file {:?}.", name)]
    FileReadError { name: std::path::PathBuf },

    /// Failed to open Ansible Vault.
    #[fail(
        display = "Failed to open Ansible vault: {:?}. Maybe the SECRET is missing?",
        file
    )]
    AnsibleVaultError { file: std::path::PathBuf },

    /// Failed to read SOPS file.
    #[fail(display = "Failed to read SOPS file: {:?}", file)]
    SOPSError { file: std::path::PathBuf },

    /// Kubectl failed to run.
    #[fail(display = "Kubectl failed to run.")]
    KubectlError,

    /// Something unexpected happened.
    #[fail(display = "An unexpected error occured.")]
    Error,
}

pub type Result<T> = std::result::Result<T, Error>;
