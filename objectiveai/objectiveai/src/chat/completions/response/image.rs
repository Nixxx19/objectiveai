use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Image {
    ImageUrl { image_url: ImageUrl },
}

impl Default for Image {
    fn default() -> Self {
        Image::ImageUrl {
            image_url: ImageUrl { url: String::new() },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageUrl {
    pub url: String,
}
