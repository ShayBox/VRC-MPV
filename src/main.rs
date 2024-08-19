use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
    path::PathBuf,
    sync::Mutex,
};

use anyhow::{bail, Result};
use chrono::{offset::LocalResult, Local, NaiveDateTime, TimeZone};
use lazy_regex::regex_captures;
use vrc_mpv::{mpv::VideoPlayer, vrchat::LogWatcher};

fn main() -> Result<()> {
    /* Keep track of the positions of each file in the `VRChat` folder */
    let positions = Mutex::new(HashMap::new());

    loop {
        let mut player = VideoPlayer::new()?;

        /* Recover the inner HashMap when the Mutex becomes poisoned */
        let mut positions = match positions.lock() {
            Err(poisoned) => poisoned.into_inner(),
            Ok(positions) => positions,
        };

        /* Catch the broken pipe to re-start MPV */
        if let Err(panic) = std::panic::catch_unwind(move || watch(&mut positions, &mut player)) {
            eprintln!("Panic: {panic:?}");
        }
    }
}

fn watch(positions: &mut HashMap<PathBuf, u64>, player: &mut VideoPlayer) -> Result<()> {
    /* Watch the `VRChat` folder for changes */
    let mut watcher = LogWatcher::new()?;
    watcher.watch()?;

    while let Ok(paths) = watcher.recv() {
        for path in paths {
            let file = File::open(&path)?;
            let mut reader = BufReader::new(file);

            /* Seek the video back to the previous position */
            if let Some(pos) = positions.get(&path) {
                reader.seek(SeekFrom::Start(*pos))?;
            } else {
                reader.seek(SeekFrom::End(0))?;
            }

            for line in reader.by_ref().lines().map_while(Result::ok) {
                /* Capture the date time and log message */
                let Some((_, s, log)) = regex_captures!(r"^([\d.: ]+) Log +- +(.+)$", &line) else {
                    continue;
                };

                /* Parse the date time without a timezone */
                let Ok(local) = NaiveDateTime::parse_from_str(s, "%Y.%m.%d %H:%M:%S") else {
                    continue;
                };

                /* Associate the date time to the local timezone */
                let LocalResult::Single(datetime) = Local.from_local_datetime(&local) else {
                    continue;
                };

                /* Capture the video player URL and play it */
                if let Some((_, url)) = regex_captures!(r"^\[Video Playback] .+ to '(.+)'$", log) {
                    player.play(url)?;
                }

                /* Capture the video offset position */
                if let Some((_, pos)) = regex_captures!(
                    r"\[ATA? (?:INFO|DEBUG|\|)[ \t]+TVManager(?:V2)? \((?:.*)\)\] (?:(?:(?:Sync enforcement(?: requested)?|Paused drift threshold exceeded). Updating to)|Jumping \[.*\] to timestamp:) ([\d.]+)$",
                    log // https://github.com/hypevhs/vrc-avpro-sucks/blob/8c652b4716ef731853d0172554848117060eabfd/src/vrc_log_reader.rs#L169-L176
                ) {
                    /* Seek the video in MPV */
                    #[allow(clippy::cast_precision_loss)]
                    let delta = (Local::now() - datetime).num_seconds() as f64;
                    let seconds = delta + pos.parse::<f64>()?;
                    player.seek(seconds)?;
                }
            }

            positions.insert(path, reader.stream_position()?);
        }
    }

    bail!("...")
}
