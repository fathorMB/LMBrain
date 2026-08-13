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
    generation: u64,
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
                generation: 0,
            })),
        }
    }

    /// Watch workspace Markdown artifacts and survive replacement of `.lmbrain`.
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

        let workspace_path = PathBuf::from(path);
        let watch_path = workspace_path.join(".lmbrain");
        watcher
            .watch(&workspace_path, RecursiveMode::NonRecursive)
            .map_err(|e| crate::errors::AppError::Watcher(e.to_string()))?;
        if watch_path.exists() {
            watcher
                .watch(&watch_path, RecursiveMode::Recursive)
                .map_err(|e| crate::errors::AppError::Watcher(e.to_string()))?;
        }

        let generation = {
            let mut inner = self.inner.lock().unwrap();
            inner.generation = inner.generation.wrapping_add(1);
            inner.watcher = Some(watcher);
            inner.active = true;
            inner.generation
        };

        let inner = self.inner.clone();

        // Spawn a thread to process events with trailing debounce
        thread::spawn(move || {
            let debounce = Duration::from_millis(500);
            let mut pending: Option<(Instant, Vec<FileEvent>)> = None;
            let mut reattach_pending = false;

            loop {
                // Check if we should stop
                let current = inner.lock().unwrap();
                if !current.active || current.generation != generation {
                    break;
                }
                drop(current);

                if reattach_pending && watch_path.is_dir() {
                    if let Some(watcher) = inner.lock().unwrap().watcher.as_mut() {
                        let _ = watcher.unwatch(&watch_path);
                        if watcher.watch(&watch_path, RecursiveMode::Recursive).is_ok() {
                            reattach_pending = false;
                        }
                    }
                }

                // Try to receive events
                match rx.try_recv() {
                    Ok(Ok(event)) => {
                        let now = Instant::now();
                        let mut events = pending.take().map(|(_, e)| e).unwrap_or_default();

                        // A controlled migration atomically replaces the complete
                        // `.lmbrain` directory. Recursive OS watches remain attached
                        // to the removed directory, so keep a non-recursive parent
                        // watch and attach the replacement as soon as it appears.
                        if event.paths.iter().any(|path| path == &watch_path) {
                            reattach_pending = true;
                        }

                        // Badge-bearing artifacts are Markdown. The `.lmbrain`
                        // root itself is also relevant because replacing it must
                        // immediately refresh the snapshot after reattachment.
                        for path in event.paths {
                            if is_relevant_path(&path, &watch_path) {
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
        inner.generation = inner.generation.wrapping_add(1);
        inner.watcher = None;
    }

    pub fn is_active(&self) -> bool {
        self.inner.lock().unwrap().active
    }
}

fn is_relevant_path(path: &std::path::Path, brain_path: &std::path::Path) -> bool {
    path == brain_path
        || (path.starts_with(brain_path)
            && path.extension().and_then(|extension| extension.to_str()) == Some("md"))
}

#[cfg(test)]
mod tests {
    use super::is_relevant_path;
    use std::path::Path;

    #[test]
    fn brain_replacement_and_markdown_artifacts_trigger_refresh() {
        let brain = Path::new("C:/workspace/.lmbrain");
        assert!(is_relevant_path(Path::new("C:/workspace/.lmbrain"), brain));
        assert!(is_relevant_path(
            Path::new("C:/workspace/.lmbrain/specs/backlog/SPEC-001.md"),
            brain
        ));
    }

    #[test]
    fn parent_watch_ignores_unrelated_repository_events() {
        let brain = Path::new("C:/workspace/.lmbrain");
        assert!(!is_relevant_path(
            Path::new("C:/workspace/README.md"),
            brain
        ));
        assert!(!is_relevant_path(
            Path::new("C:/workspace/.lmbrain-stage/specs/SPEC-001.md"),
            brain
        ));
        assert!(!is_relevant_path(
            Path::new("C:/workspace/.lmbrain/VERSION"),
            brain
        ));
    }
}
