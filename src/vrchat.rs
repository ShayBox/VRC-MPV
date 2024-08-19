use std::{ffi::OsStr, path::PathBuf, sync::LazyLock, time::Duration};

use crossbeam::channel::Receiver;
use notify::{Config, Event, PollWatcher, RecursiveMode, Result, Watcher};

#[cfg(target_os = "windows")]
const VRCHAT: &str = "%AppData%\\..\\LocalLow\\VRChat\\VRChat";

#[cfg(target_os = "linux")]
const VRCHAT: &str = "$HOME/.local/share/Steam/steamapps/compatdata/438100/pfx/drive_c/users/steamuser/AppData/LocalLow/VRChat/VRChat";

/// The path must be parsed and canonicalized during runtime
pub static PATH: LazyLock<PathBuf> = LazyLock::new(|| crate::parse_path_env(VRCHAT).unwrap());

pub struct LogWatcher {
    receiver: Receiver<Result<Event>>,
    watcher: PollWatcher,
}

impl LogWatcher {
    /// # Create a new channel and poll watcher
    ///
    /// # Errors
    /// Will return `Err` if `PollWatcher::new` fails.
    ///
    /// # Panics
    /// Will panic if `Sender::send` fails.
    pub fn new() -> Result<Self> {
        let (tx, rx) = crossbeam::channel::unbounded();
        let watcher = PollWatcher::new(
            move |event| tx.send(event).expect("Failed to send"),
            Config::default().with_poll_interval(Duration::ZERO),
        )?;

        Ok(Self {
            receiver: rx,
            watcher,
        })
    }

    /// # Watch the `VRChat` log file for changes
    ///
    /// # Errors
    /// Will return `Err` if `PollWatcher::watch` fails.
    pub fn watch(&mut self) -> Result<()> {
        let path = PATH.as_path();
        self.watcher.watch(path, RecursiveMode::NonRecursive)
    }

    /// # Receive log files from the channel
    ///
    /// # Errors
    /// Will return `Err` if `Receiver::recv` fails.
    pub fn recv(&self) -> Result<Vec<PathBuf>> {
        let event = self.receiver.recv()??;
        let mut paths = event.paths;
        paths.dedup();

        Ok(paths
            .into_iter()
            .filter(|path| {
                if let (Some(file_name), Some(extension)) = (
                    path.file_name().and_then(OsStr::to_str),
                    path.extension().and_then(OsStr::to_str),
                ) {
                    file_name.starts_with("output_log_") && extension.ends_with("txt")
                } else {
                    false
                }
            })
            .collect())
    }
}
