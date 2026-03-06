use std::io;

#[derive(Debug, thiserror::Error)]
pub enum XP3WriteError {
    #[error(transparent)]
    Io(#[from] io::Error),
}
