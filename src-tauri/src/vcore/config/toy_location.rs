/*
 * Where a toy's settings are kept.
 *
 * By default each toy model's settings are `ToyConfigs/<toy>.json` in the
 * config folder. An app built on VibeCheck can keep some toys' settings
 * somewhere else, depending on the avatar loaded (see
 * `Extensions::toy_settings_location`): every load and save of a toy's
 * settings asks here first, and the toy's default file is used whenever the
 * app gives none.
 *
 * When the answer may have changed (the avatar changed, or the app moved
 * settings), `call_plane::native_reload_toy_settings` loads every toy's
 * settings again from wherever they are now.
 */

use std::{path::PathBuf, sync::Arc};

use tauri::{AppHandle, Manager};

use crate::{
    osc::avatar_catalog::AvatarCatalog,
    util::fs::{build_path_dir, build_path_file, get_config_dir},
    vcore::errors::backend::VibeCheckToyConfigError,
};

/// What a settings location is asked: which toy, and the avatar VRChat
/// loaded last.
#[derive(Clone, Copy, Debug)]
pub struct ToySettingsQuery<'a> {
    /// The toy's name (its model), as its settings file is named.
    pub toy_name: &'a str,
    /// The avatar's opaque key (see `Control`), once VRChat has loaded one.
    pub avatar_key: Option<&'a str>,
}

/// The file to keep a toy's settings in instead of its default one, or
/// `None` for the default.
pub type ToySettingsLocation = Arc<dyn Fn(ToySettingsQuery) -> Option<PathBuf> + Send + Sync>;

/// The app's settings location, if it gave one (managed state).
#[derive(Clone, Default)]
pub struct ToySettingsLocations(pub(crate) Option<ToySettingsLocation>);

impl ToySettingsLocations {
    /// Whether an app keeps toy settings anywhere but the default files.
    pub fn given(app: &AppHandle) -> bool {
        app.try_state::<Self>().is_some_and(|l| l.0.is_some())
    }
}

/// The default file for `toy_name`'s settings: `ToyConfigs/<toy>.json`.
pub fn default_toy_settings_path(
    app: &AppHandle,
    toy_name: &str,
) -> Result<String, VibeCheckToyConfigError> {
    let config_dir = get_config_dir(app).map_err(|_| VibeCheckToyConfigError::ConfigDirFail)?;
    let toy_config_dir = build_path_dir(&[&config_dir, "ToyConfigs"]);
    Ok(build_path_file(&[
        &toy_config_dir,
        &format!("{}.json", toy_name),
    ]))
}

/// The file `toy_name`'s settings are loaded from and saved to now.
pub fn toy_settings_path(
    app: &AppHandle,
    toy_name: &str,
) -> Result<String, VibeCheckToyConfigError> {
    // Managed state, not VibeCheck's state lock: settings are loaded and
    // saved while that's held
    let location = app
        .try_state::<ToySettingsLocations>()
        .and_then(|l| l.0.clone());
    let avatar_key = || {
        app.try_state::<AvatarCatalog>()
            .and_then(|c| c.last_avatar())
            .map(|id| crate::control::avatar_key(&id))
    };
    resolve(location.as_ref(), toy_name, avatar_key, || {
        default_toy_settings_path(app, toy_name)
    })
}

/// The given location's file for `toy_name`, else the default one.
fn resolve(
    location: Option<&ToySettingsLocation>,
    toy_name: &str,
    avatar_key: impl FnOnce() -> Option<String>,
    default: impl FnOnce() -> Result<String, VibeCheckToyConfigError>,
) -> Result<String, VibeCheckToyConfigError> {
    let given = location.and_then(|location| {
        let avatar_key = avatar_key();
        location(ToySettingsQuery {
            toy_name,
            avatar_key: avatar_key.as_deref(),
        })
    });
    match given {
        Some(path) => Ok(path.to_string_lossy().into_owned()),
        None => default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default() -> Result<String, VibeCheckToyConfigError> {
        Ok("ToyConfigs/Lush.json".into())
    }

    #[test]
    fn without_a_location_the_default_file_is_used() {
        let path = resolve(None, "Lush", || panic!("not asked"), default).unwrap();
        assert_eq!(path, "ToyConfigs/Lush.json");
    }

    #[test]
    fn a_location_gets_the_toy_and_the_avatar() {
        let location: ToySettingsLocation = Arc::new(|q: ToySettingsQuery| {
            Some(PathBuf::from(format!(
                "{}/{}.json",
                q.avatar_key.unwrap_or("none"),
                q.toy_name
            )))
        });
        let path = resolve(Some(&location), "Lush", || Some("abc".into()), default).unwrap();
        assert_eq!(path, "abc/Lush.json");
        let path = resolve(Some(&location), "Lush", || None, default).unwrap();
        assert_eq!(path, "none/Lush.json");
    }

    #[test]
    fn a_location_giving_none_falls_back_to_the_default() {
        let location: ToySettingsLocation =
            Arc::new(|q: ToySettingsQuery| (q.toy_name == "Hush").then(|| "hush.json".into()));
        assert_eq!(
            resolve(Some(&location), "Lush", || None, default).unwrap(),
            "ToyConfigs/Lush.json"
        );
        assert_eq!(
            resolve(Some(&location), "Hush", || None, default).unwrap(),
            "hush.json"
        );
    }
}
