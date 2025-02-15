use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "assets/icons"]
#[include = "*"]
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(if let Some(file) = Self::get(path) {
            Some(file.data)
        } else {
            println!("Failed to load icon `{path}`");
            None
        })
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter(|p| p.starts_with(path))
            .map(|path| path.into())
            .collect())
    }
}
