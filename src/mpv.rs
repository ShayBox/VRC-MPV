use std::{process::Command, time::Duration};

use anyhow::Result;
use mpvipc::{Event, Mpv, MpvCommand, PlaylistAddOptions, SeekOptions};

#[cfg(target_os = "windows")]
const SOCKET: &str = "\\\\.\\mpv";

#[cfg(target_os = "linux")]
const SOCKET: &str = "/tmp/mpv";

/// A wrapper around MPV that allows for easier spawning and controlling MPV.
pub struct VideoPlayer(Mpv);

impl VideoPlayer {
    /// # Spawn MPV with an IPC socket server and connect to it
    ///
    /// # Errors
    ///
    /// Will return `Err` if `Command::spawn` fails.
    pub fn new() -> Result<Self> {
        Command::new("mpv")
            .arg(["--input-ipc-server", SOCKET].join("="))
            .arg("--volume=50")
            .arg("--idle")
            .spawn()?;

        std::thread::sleep(Duration::from_secs(1));
        Self::connect(SOCKET)
    }

    /// # Connect to an existing MPV with an IPC socket server
    ///
    /// # Errors
    ///
    /// Will return `Err` if `Mpv::connect` fails.
    pub fn connect(socket: &str) -> Result<Self> {
        Ok(Self(Mpv::connect(socket)?))
    }

    /// # Play a video and wait for it to be loaded
    ///
    /// # Errors
    ///
    /// Will return `Err` if `MpvCommand::LoadFile` fails.
    pub fn play(&mut self, url: &str) -> Result<()> {
        /* Play the video in MPV */
        self.0.run_command(MpvCommand::LoadFile {
            file:   String::from(url),
            option: PlaylistAddOptions::Replace,
        })?;

        /* Wait for the video to be loaded in MPV */
        while let Ok(event) = self.0.event_listen() {
            if matches!(event, Event::FileLoaded) {
                break;
            }
        }

        Ok(())
    }

    /// # Seek the video to a specific absolute time in seconds
    ///
    /// # Errors
    ///
    /// Will return `Err` if `MpvCommand::Seek` fails.
    pub fn seek(&self, seconds: f64) -> Result<()> {
        self.0.seek(seconds, SeekOptions::Absolute)?;

        Ok(())
    }
}
