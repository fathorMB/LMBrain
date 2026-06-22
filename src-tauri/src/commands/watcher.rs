use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter};

use crate::models::file::{FileEvent, FileEventKind};

/// File watcher service that monitors the workspace for changes
/// and emits events to the frontend with trailing debounce.
pub struct FileWatcherService {
    inner: Arc<Mutex<WatcherInner>>,
}

struct WatcherInner {
    watcher: Option<RecommendedWatcher>,
    active: bool,
}

impl Default for FileWatcherService {
    fn default() -> Self {
        Self::new()
    }
}

impl FileWatcherService {
    pub fn new() -> Self {
        FileWatcherService {
            inner: Arc::new(Mutex::new(WatcherInner {
                watcher: None,
                active: false,
            })),
        }
    }

    /// Start watching the given directory for .md file changes.
    /// Stops any previously active watcher first.
    pub fn start(&self, path: &str, app: AppHandle) -> Result<(), crate::errors::AppError> {
        // Stop any existing watcher first
        self.stop();

        let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default(),
        )
        .map_err(|e| crate::errors::AppError::Watcher(e.to_string()))?;

        let watch_path = PathBuf::from(path).join(".lmbrain");
        if watch_path.exists() {
            watcher
                .watch(&watch_path, RecursiveMode::Recursive)
                .map_err(|e| crate::errors::AppError::Watcher(e.to_string()))?;
        }

        {
            let mut inner = self.inner.lock().unwrap();
            inner.watcher = Some(watcher);
            inner.active = true;
        }

        let inner = self.inner.clone();

        // Spawn a thread to process events with trailing debounce
        thread::spawn(move || {
            let debounce = Duration::from_millis(500);
            let mut pending: Option<(Instant, Vec<FileEvent>)> = None;

            loop {
                // Check if we should stop
                if !inner.lock().unwrap().active {
                    break;
                }

                // Try to receive events
                match rx.try_recv() {
                    Ok(Ok(event)) => {
                        let now = Instant::now();
                        let mut events = pending.take().map(|(_, e)| e).unwrap_or_default();

                        // Filter to .md files only
                        for path in event.paths {
                            if path.extension().and_then(|e| e.to_str()) == Some("md") {
                                let kind = match event.kind {
                                    EventKind::Create(_) => FileEventKind::Created,
                                    EventKind::Modify(_) => FileEventKind::Modified,
                                    EventKind::Remove(_) => FileEventKind::Removed,
                                    _ => continue,
                                };
                                events.push(FileEvent {
                                    kind,
                                    path: path.to_string_lossy().to_string(),
                                });
                            }
                        }

                        if !events.is_empty() {
                            pending = Some((now, events));
                        }
                    }
                    Ok(Err(_)) => {}
                    Err(mpsc::TryRecvError::Empty) => {
                        // No new events — check if we have a pending debounced emit
                        if let Some((last_time, events)) = pending.take() {
                            if last_time.elapsed() >= debounce {
                                // Emit a single coalesced refresh event
                                let _ = app.emit("file-changed", "refresh");
                            } else {
                                // Not enough time yet — put it back
                                pending = Some((last_time, events));
                            }
                        }
                        thread::sleep(Duration::from_millis(50));
                    }
                    Err(mpsc::TryRecvError::Disconnected) => break,
                }
            }
        });

        Ok(())
    }

    pub fn stop(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.active = false;
        inner.watcher = None;
    }

    pub fn is_active(&self) -> bool {
        self.inner.lock().unwrap().active
    }
}
