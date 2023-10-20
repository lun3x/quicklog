use crate::Flush;
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::SystemTime,
};

/// Flushes into a rolling log file
pub struct RollingFileFlusher {
    dir: PathBuf,
    prefix: String,
    file: File,
    rollover: SystemTime,
}

impl RollingFileFlusher {
    /// Flushes into file with specified path
    pub fn new<P: AsRef<Path>>(dir: P, prefix: &str, log_time: SystemTime) -> RollingFileFlusher {
        let datetime = time::OffsetDateTime::from(log_time);
        let rollover = datetime
            .date()
            .midnight()
            .checked_add(time::Duration::DAY)
            .expect("overflowed date")
            .assume_utc();

        let format = time::format_description::parse("[year]-[month]-[day]")
            .expect("unable to create a formatter");
        let date = datetime
            .format(&format)
            .expect("unable to format OffsetDateTime");
        let dir = PathBuf::from(dir.as_ref());
        let path = dir.join(format!("{prefix}.log.{date}"));

        match OpenOptions::new().create(true).append(true).open(&path) {
            Ok(file) => Self {
                dir,
                prefix: prefix.to_string(),
                file,
                rollover: rollover.into(),
            },
            Err(_) => panic!("unable to open file: {}", path.to_string_lossy()),
        }
    }
}

impl Flush for RollingFileFlusher {
    fn flush_one(&mut self, display: String, log_time: SystemTime) {
        if log_time >= self.rollover {
            *self = Self::new(&self.dir, &self.prefix, log_time)
        }
        match self.file.write_all(display.as_bytes()) {
            Ok(_) => (),
            Err(_) => panic!("Unable to write to file"),
        };
    }
}
