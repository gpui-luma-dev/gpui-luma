use thiserror::Error;

#[derive(Debug, Error)]
pub enum FontCatalogError {
    #[error("failed to enumerate system fonts")]
    Enumeration(#[from] font_kit::error::SelectionError),
}
