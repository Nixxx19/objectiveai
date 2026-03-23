use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FunctionsConfig {
    #[serde(skip_serializing_if = "FunctionsInventionsConfig::is_none")]
    pub inventions: Option<FunctionsInventionsConfig>,
    #[serde(skip_serializing_if = "FunctionsProfilesConfig::is_none")]
    pub profiles: Option<FunctionsProfilesConfig>,
    #[serde(skip_serializing_if = "crate::util::vec_is_none_or_empty")]
    pub favorites: Option<Vec<super::Favorite>>,
}

impl FunctionsConfig {
    pub fn is_empty(&self) -> bool {
        self.inventions.as_ref().is_none_or(|cfg| cfg.is_empty())
    }

    pub fn is_none(this: &Option<Self>) -> bool {
        this.as_ref().is_none_or(|cfg| cfg.is_empty())
    }

    pub fn inventions(&mut self) -> &mut FunctionsInventionsConfig {
        self.inventions.get_or_insert_with(FunctionsInventionsConfig::default)
    }

    pub fn profiles(&mut self) -> &mut FunctionsProfilesConfig {
        self.profiles.get_or_insert_with(FunctionsProfilesConfig::default)
    }

    pub fn get_favorites(&self) -> &[super::Favorite] {
        self.favorites.as_deref().unwrap_or(&[])
    }

    pub fn add_favorite(&mut self, favorite: super::Favorite) {
        self.favorites.get_or_insert_with(Vec::new).push(favorite);
    }

    pub fn del_favorite(&mut self, index: usize) -> Result<(), super::ConfigError> {
        let favorites = self.favorites.as_mut().ok_or(super::ConfigError::IndexOutOfBounds(index, 0))?;
        if index >= favorites.len() {
            return Err(super::ConfigError::IndexOutOfBounds(index, favorites.len()));
        }
        favorites.remove(index);
        Ok(())
    }

    pub fn edit_favorite(&mut self, index: usize, note: String) -> Result<(), super::ConfigError> {
        let favorites = self.favorites.as_mut().ok_or(super::ConfigError::IndexOutOfBounds(index, 0))?;
        let len = favorites.len();
        let favorite = favorites.get_mut(index).ok_or(super::ConfigError::IndexOutOfBounds(index, len))?;
        favorite.note = note;
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FunctionsInventionsConfig {
    pub remote: Option<crate::Remote>,
}

impl FunctionsInventionsConfig {
    pub fn is_empty(&self) -> bool {
        self.remote.is_none()
    }

    pub fn is_none(this: &Option<Self>) -> bool {
        this.as_ref().is_none_or(|cfg| cfg.is_empty())
    }

    pub fn get_remote(&self) -> Option<&crate::Remote> {
        self.remote.as_ref()
    }

    pub fn set_remote(&mut self, remote: crate::Remote) -> Result<(), super::ConfigError> {
        if matches!(remote, crate::Remote::Mock) {
            return Err(super::ConfigError::InvalidRemote(remote));
        }
        self.remote = Some(remote);
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FunctionsProfilesConfig {
    #[serde(skip_serializing_if = "crate::util::vec_is_none_or_empty")]
    pub favorites: Option<Vec<super::Favorite>>,
}

impl FunctionsProfilesConfig {
    pub fn is_empty(&self) -> bool {
        crate::util::vec_is_none_or_empty(&self.favorites)
    }

    pub fn is_none(this: &Option<Self>) -> bool {
        this.as_ref().is_none_or(|cfg| cfg.is_empty())
    }

    pub fn get_favorites(&self) -> &[super::Favorite] {
        self.favorites.as_deref().unwrap_or(&[])
    }

    pub fn add_favorite(&mut self, favorite: super::Favorite) {
        self.favorites.get_or_insert_with(Vec::new).push(favorite);
    }

    pub fn del_favorite(&mut self, index: usize) -> Result<(), super::ConfigError> {
        let favorites = self.favorites.as_mut().ok_or(super::ConfigError::IndexOutOfBounds(index, 0))?;
        if index >= favorites.len() {
            return Err(super::ConfigError::IndexOutOfBounds(index, favorites.len()));
        }
        favorites.remove(index);
        Ok(())
    }

    pub fn edit_favorite(&mut self, index: usize, note: String) -> Result<(), super::ConfigError> {
        let favorites = self.favorites.as_mut().ok_or(super::ConfigError::IndexOutOfBounds(index, 0))?;
        let len = favorites.len();
        let favorite = favorites.get_mut(index).ok_or(super::ConfigError::IndexOutOfBounds(index, len))?;
        favorite.note = note;
        Ok(())
    }
}
