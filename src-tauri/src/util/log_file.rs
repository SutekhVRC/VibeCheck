/*
 * VibeCheck's log file, for bug reports.
 *
 * Performance: logging a line only formats it and queues it; a background
 * thread writes the queue and flushes whenever it runs dry. If the queue is
 * full, lines are dropped (and counted in the file) rather than slowing the
 * caller. Levels that aren't logged are skipped before anything is
 * formatted.
 *
 * Privacy: users post logs publicly (Discord, GitHub), so every line is
 * redacted before it's written (see `Redactor`): the home folder, computer
 * and account names, VRChat user, avatar and world ids, Bluetooth addresses
 * and IP addresses other than loopback. Never log credentials or tokens in
 * the first place; redaction is a safety net, not a licence. Times are UTC,
 * so the file doesn't give away the user's time zone.
 *
 * Files: `VibeCheck.log` is this session, `VibeCheck.old.log` the one
 * before (or this session's start, once it passes MAX_FILE_BYTES), so the
 * folder stays under about twice that. `VibeCheck.lock` is held by the
 * running app, so starting VibeCheck again while it runs (which hands over
 * to the running one and quits) leaves its log alone.
 */

use std::{
    borrow::Cow,
    fs::{self, File, OpenOptions, TryLockError},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{sync_channel, Receiver, SyncSender, TrySendError},
        OnceLock,
    },
    thread,
    time::Duration,
};

use log::{LevelFilter, Log, Metadata, Record};
use regex::{Captures, Regex};

const LOG_NAME: &str = "VibeCheck.log";
const OLD_LOG_NAME: &str = "VibeCheck.old.log";
/// Locked by the running VibeCheck, so a second start leaves the logs alone.
const LOCK_NAME: &str = "VibeCheck.lock";
/// Past this, the current file becomes the old one and a new one starts.
const MAX_FILE_BYTES: u64 = 5 * 1024 * 1024;
/// Lines waiting to be written. Past this, new lines are dropped.
const QUEUE_LINES: usize = 4096;
/// How long a flush (e.g. from the panic hook) waits for the writer.
const FLUSH_WAIT: Duration = Duration::from_millis(500);

static LOG_DIR: OnceLock<PathBuf> = OnceLock::new();

/// The folder the log files are in, once [`init`] has run.
pub fn log_dir() -> Option<&'static Path> {
    LOG_DIR.get().map(PathBuf::as_path)
}

/// Starts logging to the log folder for the app `identifier` (the same
/// folder Tauri calls the app log dir). Debug builds also print to stderr
/// and log at debug level. If the folder or file can't be used, logs still
/// go to stderr in debug builds. Call once, before anything logs.
pub fn init(identifier: &str) {
    let debug = cfg!(debug_assertions);
    let dir = directories::BaseDirs::new()
        .map(|dirs| dirs.data_local_dir().join(identifier).join("logs"));

    let tx = dir.as_ref().and_then(|dir| match open_log(dir) {
        Ok(OpenedLog { file, lock }) => {
            let _ = LOG_DIR.set(dir.clone());
            let (tx, rx) = sync_channel(QUEUE_LINES);
            let dir = dir.clone();
            thread::Builder::new()
                .name("log-writer".into())
                .spawn(move || {
                    // Held for as long as the app runs; the OS lets go of it
                    // when the app ends, however it ends
                    let _lock = lock;
                    write_lines(rx, file, &dir)
                })
                .ok()
                .map(|_| tx)
        }
        Err(e) => {
            eprintln!(
                "VibeCheck: couldn't open the log file in {}: {e}",
                dir.display()
            );
            None
        }
    });

    let logger = FileLogger {
        tx,
        dropped: AtomicUsize::new(0),
        redactor: Redactor::for_this_machine(),
        stderr: debug,
        own_level: if debug {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        },
        other_level: if debug {
            LevelFilter::Debug
        } else {
            LevelFilter::Warn
        },
    };
    let max = logger.own_level.max(logger.other_level);
    if log::set_boxed_logger(Box::new(logger)).is_ok() {
        log::set_max_level(max);
    }
}

/// A new log file, and the lock that says this VibeCheck owns the folder.
struct OpenedLog {
    file: BufWriter<File>,
    /// Locked while this VibeCheck runs (see [`open_log`]).
    lock: File,
}

/// Moves last session's log aside and opens a new one, unless another
/// VibeCheck already runs and owns the folder (then it's an error, and its
/// files are left alone). A second start runs this before it finds the
/// first and hands over to it; without the lock it would move the running
/// app's log aside, and a third start would overwrite it.
fn open_log(dir: &Path) -> std::io::Result<OpenedLog> {
    fs::create_dir_all(dir)?;
    let lock = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.join(LOCK_NAME))?;
    match lock.try_lock() {
        Ok(()) => {}
        Err(TryLockError::WouldBlock) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::WouldBlock,
                "another VibeCheck is running and logging there",
            ));
        }
        // Locking isn't possible here (e.g. the file system can't): log
        // anyway, as before
        Err(TryLockError::Error(_)) => {}
    }
    let path = dir.join(LOG_NAME);
    if path.exists() {
        fs::rename(&path, dir.join(OLD_LOG_NAME))?;
    }
    let file = create_private(&path).map(BufWriter::new)?;
    Ok(OpenedLog { file, lock })
}

/// A new file only the user can read (on Linux; Windows keeps it in the
/// user's own AppData).
fn create_private(path: &Path) -> std::io::Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}

enum Message {
    Line(String),
    /// Write everything queued, flush, then reply.
    Flush(SyncSender<()>),
}

/// The writer thread: writes queued lines, flushing whenever the queue runs
/// dry, and starts a new file past MAX_FILE_BYTES.
fn write_lines(rx: Receiver<Message>, mut file: BufWriter<File>, dir: &Path) {
    let mut written: u64 = 0;
    while let Ok(first) = rx.recv() {
        let mut next = Some(first);
        while let Some(message) = next.take() {
            match message {
                Message::Line(line) => {
                    if written > MAX_FILE_BYTES {
                        if let Ok(new) = start_over(&mut file, dir) {
                            file = new;
                            written = 0;
                        }
                    }
                    if file.write_all(line.as_bytes()).is_ok() {
                        written += line.len() as u64;
                    }
                }
                Message::Flush(done) => {
                    let _ = file.flush();
                    let _ = done.send(());
                }
            }
            next = rx.try_recv().ok();
        }
        let _ = file.flush();
    }
}

fn start_over(file: &mut BufWriter<File>, dir: &Path) -> std::io::Result<BufWriter<File>> {
    file.flush()?;
    fs::rename(dir.join(LOG_NAME), dir.join(OLD_LOG_NAME))?;
    let mut new = BufWriter::new(create_private(&dir.join(LOG_NAME))?);
    writeln!(
        new,
        "(continued: the start of this session is in {OLD_LOG_NAME})"
    )?;
    Ok(new)
}

struct FileLogger {
    tx: Option<SyncSender<Message>>,
    /// Lines dropped because the queue was full, not yet noted in the file.
    dropped: AtomicUsize,
    redactor: Redactor,
    stderr: bool,
    /// For VibeCheck's own crates.
    own_level: LevelFilter,
    /// For everything else (Buttplug, mDNS, Tauri...), which is chatty.
    other_level: LevelFilter,
}

impl FileLogger {
    fn level_for(&self, target: &str) -> LevelFilter {
        if target.starts_with("vibecheck") || target.starts_with("vrcoscquery") {
            self.own_level
        } else {
            self.other_level
        }
    }

    fn queue(&self, tx: &SyncSender<Message>, line: String) {
        match tx.try_send(Message::Line(line)) {
            Ok(()) => {
                let dropped = self.dropped.swap(0, Ordering::Relaxed);
                if dropped > 0 {
                    let note = format!("({dropped} log lines dropped: logging faster than the file could keep up)\n");
                    if tx.try_send(Message::Line(note)).is_err() {
                        self.dropped.fetch_add(dropped, Ordering::Relaxed);
                    }
                }
            }
            Err(TrySendError::Full(_)) => {
                self.dropped.fetch_add(1, Ordering::Relaxed);
            }
            Err(TrySendError::Disconnected(_)) => {}
        }
    }
}

impl Log for FileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level_for(metadata.target())
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let message = record.args().to_string();
        let line = format!(
            "{} {:<5} [{}] {}\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3fZ"),
            record.level(),
            record.target(),
            self.redactor.redact(&message),
        );
        if self.stderr {
            eprint!("{line}");
        }
        if let Some(tx) = &self.tx {
            self.queue(tx, line);
        }
    }

    /// Waits (briefly) until everything queued is on disk. Used by the panic
    /// hook, so a crash's last lines aren't lost.
    fn flush(&self) {
        let Some(tx) = &self.tx else { return };
        let (done_tx, done_rx) = sync_channel(1);
        if tx.try_send(Message::Flush(done_tx)).is_ok() {
            let _ = done_rx.recv_timeout(FLUSH_WAIT);
        }
    }
}

/// Removes personal details from log lines. See the module docs.
struct Redactor {
    /// Exact text replaced first, longest first: (text, replacement).
    literals: Vec<(String, &'static str)>,
    /// Computer and account names, as whole words, any case.
    names: Option<Regex>,
    vrchat_ids: Regex,
    bluetooth: Regex,
    ipv4: Regex,
    ipv6: Regex,
}

impl Redactor {
    fn for_this_machine() -> Self {
        let home =
            directories::BaseDirs::new().map(|dirs| dirs.home_dir().to_string_lossy().into_owned());
        let user = std::env::var("USERNAME")
            .or_else(|_| std::env::var("USER"))
            .ok();
        let host = sysinfo::System::host_name();
        Self::new(home.as_deref(), &[user.as_deref(), host.as_deref()])
    }

    /// `names`: computer and account names. Ones shorter than 3 characters
    /// are skipped, as they'd blank out ordinary words.
    fn new(home: Option<&str>, names: &[Option<&str>]) -> Self {
        let mut literals = Vec::new();
        if let Some(home) = home.filter(|h| h.len() > 1) {
            literals.push((home.to_owned(), "~"));
            // The same folder written with the other kind of slash
            let flipped = if home.contains('\\') {
                home.replace('\\', "/")
            } else {
                home.replace('/', "\\")
            };
            if flipped != home {
                literals.push((flipped, "~"));
            }
        }
        literals.sort_by_key(|(text, _)| std::cmp::Reverse(text.len()));

        let names: Vec<String> = names
            .iter()
            .flatten()
            .filter(|n| n.chars().count() >= 3)
            .map(|n| regex::escape(n))
            .collect();
        let names = (!names.is_empty())
            .then(|| Regex::new(&format!(r"(?i)\b(?:{})\b", names.join("|"))).ok())
            .flatten();

        Self {
            literals,
            names,
            vrchat_ids: Regex::new(r"\b(usr|avtr|wrld|grp|file)_[0-9A-Fa-f]{8}-[0-9A-Fa-f-]{4,}")
                .unwrap(),
            bluetooth: Regex::new(r"\b(?:[0-9A-Fa-f]{2}[:-]){5}[0-9A-Fa-f]{2}\b").unwrap(),
            ipv4: Regex::new(r"\b(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})\b").unwrap(),
            // Full forms (eight groups, or several groups with a "::"); short
            // ones like "::1" are loopback and kept
            ipv6: Regex::new(
                r"\b(?:[0-9A-Fa-f]{1,4}:){7}[0-9A-Fa-f]{1,4}\b|\b(?:[0-9A-Fa-f]{1,4}:){1,6}:(?:[0-9A-Fa-f]{1,4}:){0,5}[0-9A-Fa-f]{1,4}\b",
            )
            .unwrap(),
        }
    }

    fn redact<'a>(&self, text: &'a str) -> Cow<'a, str> {
        let mut out = Cow::Borrowed(text);
        for (literal, replacement) in &self.literals {
            if out.contains(literal.as_str()) {
                out = Cow::Owned(out.replace(literal.as_str(), replacement));
            }
        }
        out = replace(out, &self.vrchat_ids, |c| format!("{}_<id>", &c[1]));
        out = replace(out, &self.bluetooth, |_| "<bt-address>".to_owned());
        out = replace(out, &self.ipv6, |_| "<ip>".to_owned());
        out = replace(out, &self.ipv4, |c| {
            // Loopback and "any address" say nothing about the user, and help
            if &c[1] == "127" || &c[0] == "0.0.0.0" {
                c[0].to_owned()
            } else {
                "<ip>".to_owned()
            }
        });
        if let Some(names) = &self.names {
            out = replace(out, names, |_| "<name>".to_owned());
        }
        out
    }
}

/// Runs `regex` over `text`, allocating only if something matched.
fn replace<'a>(
    text: Cow<'a, str>,
    regex: &Regex,
    with: impl FnMut(&Captures) -> String,
) -> Cow<'a, str> {
    if !regex.is_match(&text) {
        return text;
    }
    Cow::Owned(regex.replace_all(&text, with).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A new, empty folder for one test.
    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("vibecheck-log-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn a_second_start_leaves_the_running_apps_logs_alone() {
        let dir = temp_dir("second-start");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(LOG_NAME), "last session\n").unwrap();

        let mut running = open_log(&dir).unwrap();
        writeln!(running.file, "running").unwrap();
        running.file.flush().unwrap();

        // Started again while that one runs: an error, nothing moved
        let again = open_log(&dir);
        assert_eq!(
            again.err().map(|e| e.kind()),
            Some(std::io::ErrorKind::WouldBlock)
        );
        assert_eq!(fs::read_to_string(dir.join(LOG_NAME)).unwrap(), "running\n");
        assert_eq!(
            fs::read_to_string(dir.join(OLD_LOG_NAME)).unwrap(),
            "last session\n"
        );

        // Once it ends, the next start moves its log aside as usual
        drop(running);
        let next = open_log(&dir).unwrap();
        assert_eq!(
            fs::read_to_string(dir.join(OLD_LOG_NAME)).unwrap(),
            "running\n"
        );
        drop(next);
        let _ = fs::remove_dir_all(&dir);
    }

    fn redactor() -> Redactor {
        Redactor::new(
            Some("/home/alice"),
            &[Some("alice"), Some("Gaming-PC"), Some("x")],
        )
    }

    #[test]
    fn hides_the_home_folder_and_names() {
        let r = redactor();
        assert_eq!(
            r.redact(
                "Saving toy config to: /home/alice/.config/com.vibecheck/ToyConfigs/Lush.json"
            ),
            "Saving toy config to: ~/.config/com.vibecheck/ToyConfigs/Lush.json"
        );
        assert_eq!(
            r.redact("host GAMING-PC.local, user Alice"),
            "host <name>.local, user <name>"
        );
        // Too-short names are left alone rather than blanking words
        assert_eq!(r.redact("x marks the spot"), "x marks the spot");
    }

    #[test]
    fn hides_windows_home_folders_with_either_slash() {
        let r = Redactor::new(Some(r"C:\Users\Alice"), &[]);
        assert_eq!(r.redact(r"C:\Users\Alice\AppData\x"), r"~\AppData\x");
        assert_eq!(r.redact("C:/Users/Alice/AppData/x"), "~/AppData/x");
    }

    #[test]
    fn hides_vrchat_ids() {
        let r = redactor();
        assert_eq!(
            r.redact("Clearing ~/OSC/usr_0123abcd-89ab-4cde-8f01-23456789abcd/Avatars avtr_fedcba98-7654-3210-fedc-ba9876543210"),
            "Clearing ~/OSC/usr_<id>/Avatars avtr_<id>"
        );
    }

    #[test]
    fn hides_addresses_but_keeps_loopback() {
        let r = redactor();
        assert_eq!(
            r.redact("VRChat at 192.168.1.20:9000, bound 127.0.0.1:9001 and 0.0.0.0:10042"),
            "VRChat at <ip>:9000, bound 127.0.0.1:9001 and 0.0.0.0:10042"
        );
        assert_eq!(
            r.redact("toy C4:7F:51:0A:1B:2C connected"),
            "toy <bt-address> connected"
        );
        assert_eq!(r.redact("from fe80::1c2d:3e4f:5a6b:7c8d"), "from <ip>");
        assert_eq!(r.redact("listening on ::1"), "listening on ::1");
    }

    #[test]
    fn leaves_ordinary_lines_alone() {
        let r = redactor();
        let line = "Toy Connected: Lovense Lush 3 | 0 at 12:34:56, mode Raw, level 0.50";
        assert!(matches!(r.redact(line), Cow::Borrowed(_)));
    }

    #[test]
    fn writes_lines_and_starts_over_past_the_limit() {
        let dir = std::env::temp_dir().join(format!("vibecheck-log-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(LOG_NAME), "last session\n").unwrap();

        let file = open_log(&dir).unwrap();
        assert_eq!(
            fs::read_to_string(dir.join(OLD_LOG_NAME)).unwrap(),
            "last session\n"
        );

        let (tx, rx) = sync_channel(QUEUE_LINES);
        let writer_dir = dir.clone();
        let writer = thread::spawn(move || write_lines(rx, file.file, &writer_dir));
        let big = "x".repeat(MAX_FILE_BYTES as usize + 1) + "\n";
        tx.send(Message::Line(big)).unwrap();
        tx.send(Message::Line("after\n".into())).unwrap();
        drop(tx);
        writer.join().unwrap();

        let current = fs::read_to_string(dir.join(LOG_NAME)).unwrap();
        assert!(current.starts_with("(continued"));
        assert!(current.ends_with("after\n"));
        assert!(fs::metadata(dir.join(OLD_LOG_NAME)).unwrap().len() > MAX_FILE_BYTES);

        fs::remove_dir_all(&dir).unwrap();
    }
}
