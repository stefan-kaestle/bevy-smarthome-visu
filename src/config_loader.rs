use crate::config;

use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    reflect::TypePath,
};
use thiserror::Error;

/// Custom asset loader for Smart Home configuration.

#[derive(Asset, TypePath, Debug)]
pub struct SmartHomeConfigAsset {
    // Smart home config as given by Json file
    pub config: config::ItemConfiguration,
}

#[derive(Default)]
pub struct SmartHomeConfigAssetLoader;

/// Possible errors that can be produced by [`SmartHomeConfigAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum SmartHomeConfigAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load file: {0}")]
    Io(#[from] std::io::Error),
    // Conversion from UTF-8 failed
    #[error("Could not read file as UTF-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    // Json parsing error
    #[error("Failed to parse Json: {0}")]
    Json(#[from] serde_json::Error),
}

impl AssetLoader for SmartHomeConfigAssetLoader {
    type Asset = SmartHomeConfigAsset;
    type Settings = ();
    type Error = SmartHomeConfigAssetLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let file_as_str = String::from_utf8(bytes)?;
            let config = serde_json::from_str(&file_as_str)?;

            Ok(SmartHomeConfigAsset { config })
        })
    }
}
