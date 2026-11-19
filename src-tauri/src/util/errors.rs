use thiserror::Error;

#[derive(Debug, Error)]
pub enum UtilError {
    #[error("Couldn't start the toy connection service (Buttplug).")]
    BPIOInit,
    #[cfg(target_os = "windows")]
    #[error("Couldn't find your home folder.")]
    HomeDirFS,
    #[error("Couldn't find or create VibeCheck's config folder.")]
    ConfigDirFS,
}
