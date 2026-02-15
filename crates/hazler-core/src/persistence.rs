/// State persistence for save/resume functionality
/// Supports JSON and SQLite backends for crawl state storage
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use tracing::{info, warn};
use url::Url;

/// Crawl session state that can be persisted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlState {
    /// Starting URLs
    pub start_urls: Vec<String>,
    /// URLs that have been visited
    pub visited: HashSet<String>,
    /// URLs in the queue to be visited
    pub queue: Vec<QueuedUrl>,
    /// Total pages crawled
    pub pages_crawled: usize,
    /// Timestamp when state was saved
    pub saved_at: String,
    /// Configuration snapshot
    pub config_snapshot: ConfigSnapshot,
}

/// Queued URL with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedUrl {
    pub url: String,
    pub depth: usize,
    pub referrer: Option<String>,
}

/// Configuration snapshot for validation on resume
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSnapshot {
    pub max_depth: usize,
    pub user_agent: String,
    pub stealth_mode: bool,
}

impl CrawlState {
    /// Create a new crawl state
    pub fn new(start_urls: Vec<Url>, config_snapshot: ConfigSnapshot) -> Self {
        Self {
            start_urls: start_urls.iter().map(|u| u.to_string()).collect(),
            visited: HashSet::new(),
            queue: Vec::new(),
            pages_crawled: 0,
            saved_at: chrono::Utc::now().to_rfc3339(),
            config_snapshot,
        }
    }

    /// Add a visited URL
    pub fn add_visited(&mut self, url: &Url) {
        self.visited.insert(url.to_string());
    }

    /// Add a URL to the queue
    pub fn add_to_queue(&mut self, url: &Url, depth: usize, referrer: Option<&Url>) {
        self.queue.push(QueuedUrl {
            url: url.to_string(),
            depth,
            referrer: referrer.map(|u| u.to_string()),
        });
    }

    /// Check if URL has been visited
    pub fn is_visited(&self, url: &Url) -> bool {
        self.visited.contains(url.as_str())
    }

    /// Get number of visited URLs
    pub fn visited_count(&self) -> usize {
        self.visited.len()
    }

    /// Get number of queued URLs
    pub fn queue_count(&self) -> usize {
        self.queue.len()
    }
}

/// Persistence backend type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PersistenceBackend {
    Json,
    Sqlite,
}

/// State persistence manager
pub struct StatePersistence {
    backend: PersistenceBackend,
    state_file: PathBuf,
}

impl StatePersistence {
    /// Create a new state persistence manager
    pub fn new(backend: PersistenceBackend, state_file: PathBuf) -> Self {
        Self {
            backend,
            state_file,
        }
    }

    /// Create with JSON backend
    pub fn json(state_file: PathBuf) -> Self {
        Self::new(PersistenceBackend::Json, state_file)
    }

    /// Create with default location (hazler-state.json)
    pub fn default_json() -> Self {
        Self::json(PathBuf::from("hazler-state.json"))
    }

    /// Save crawl state
    pub fn save(&self, state: &CrawlState) -> anyhow::Result<()> {
        match self.backend {
            PersistenceBackend::Json => self.save_json(state),
            PersistenceBackend::Sqlite => self.save_sqlite(state),
        }
    }

    /// Load crawl state
    pub fn load(&self) -> anyhow::Result<CrawlState> {
        match self.backend {
            PersistenceBackend::Json => self.load_json(),
            PersistenceBackend::Sqlite => self.load_sqlite(),
        }
    }

    /// Check if state file exists
    pub fn exists(&self) -> bool {
        self.state_file.exists()
    }

    /// Delete state file
    pub fn delete(&self) -> anyhow::Result<()> {
        if self.exists() {
            fs::remove_file(&self.state_file)?;
            info!("Deleted state file: {}", self.state_file.display());
        }
        Ok(())
    }

    /// Save state to JSON file
    fn save_json(&self, state: &CrawlState) -> anyhow::Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = self.state_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = File::create(&self.state_file)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, state)?;

        info!(
            "Saved crawl state to {}: {} visited, {} queued",
            self.state_file.display(),
            state.visited_count(),
            state.queue_count()
        );
        Ok(())
    }

    /// Load state from JSON file
    fn load_json(&self) -> anyhow::Result<CrawlState> {
        let file = File::open(&self.state_file)?;
        let reader = BufReader::new(file);
        let state: CrawlState = serde_json::from_reader(reader)?;

        info!(
            "Loaded crawl state from {}: {} visited, {} queued",
            self.state_file.display(),
            state.visited_count(),
            state.queue_count()
        );
        Ok(state)
    }

    /// Save state to SQLite database
    fn save_sqlite(&self, _state: &CrawlState) -> anyhow::Result<()> {
        // TODO: Implement SQLite backend
        warn!("SQLite backend not yet implemented, falling back to JSON");
        self.save_json(_state)
    }

    /// Load state from SQLite database
    fn load_sqlite(&self) -> anyhow::Result<CrawlState> {
        // TODO: Implement SQLite backend
        warn!("SQLite backend not yet implemented, falling back to JSON");
        self.load_json()
    }
}

/// Auto-save manager for periodic state persistence
pub struct AutoSave {
    persistence: StatePersistence,
    save_interval: std::time::Duration,
    last_save: std::sync::Arc<std::sync::Mutex<std::time::Instant>>,
}

impl AutoSave {
    /// Create a new auto-save manager
    pub fn new(persistence: StatePersistence, save_interval_secs: u64) -> Self {
        Self {
            persistence,
            save_interval: std::time::Duration::from_secs(save_interval_secs),
            last_save: std::sync::Arc::new(std::sync::Mutex::new(std::time::Instant::now())),
        }
    }

    /// Check if it's time to save and save if needed
    pub fn try_save(&self, state: &CrawlState) -> anyhow::Result<bool> {
        let mut last_save = self.last_save.lock().unwrap();
        if last_save.elapsed() >= self.save_interval {
            self.persistence.save(state)?;
            *last_save = std::time::Instant::now();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Force save regardless of interval
    pub fn force_save(&self, state: &CrawlState) -> anyhow::Result<()> {
        self.persistence.save(state)?;
        let mut last_save = self.last_save.lock().unwrap();
        *last_save = std::time::Instant::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_state() -> CrawlState {
        let start_urls = vec![Url::parse("https://example.com").unwrap()];
        let config = ConfigSnapshot {
            max_depth: 3,
            user_agent: "TestAgent".to_string(),
            stealth_mode: true,
        };
        let mut state = CrawlState::new(start_urls, config);

        state.add_visited(&Url::parse("https://example.com").unwrap());
        state.add_visited(&Url::parse("https://example.com/page1").unwrap());
        state.add_to_queue(
            &Url::parse("https://example.com/page2").unwrap(),
            1,
            Some(&Url::parse("https://example.com").unwrap()),
        );
        state.pages_crawled = 2;

        state
    }

    #[test]
    fn test_crawl_state_creation() {
        let state = create_test_state();
        assert_eq!(state.visited_count(), 2);
        assert_eq!(state.queue_count(), 1);
        assert_eq!(state.pages_crawled, 2);
    }

    #[test]
    fn test_is_visited() {
        let state = create_test_state();
        assert!(state.is_visited(&Url::parse("https://example.com").unwrap()));
        assert!(!state.is_visited(&Url::parse("https://example.com/page2").unwrap()));
    }

    #[test]
    fn test_json_persistence_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("test-state.json");
        let persistence = StatePersistence::json(state_file.clone());

        let state = create_test_state();
        persistence.save(&state).unwrap();

        assert!(persistence.exists());

        let loaded_state = persistence.load().unwrap();
        assert_eq!(loaded_state.visited_count(), state.visited_count());
        assert_eq!(loaded_state.queue_count(), state.queue_count());
        assert_eq!(loaded_state.pages_crawled, state.pages_crawled);
    }

    #[test]
    fn test_persistence_delete() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("test-state.json");
        let persistence = StatePersistence::json(state_file.clone());

        let state = create_test_state();
        persistence.save(&state).unwrap();
        assert!(persistence.exists());

        persistence.delete().unwrap();
        assert!(!persistence.exists());
    }

    #[test]
    fn test_auto_save_interval() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("test-state.json");
        let persistence = StatePersistence::json(state_file);
        let auto_save = AutoSave::new(persistence, 1); // 1 second interval

        let state = create_test_state();

        // First try should not save (just created)
        let saved = auto_save.try_save(&state).unwrap();
        assert!(!saved);

        // Wait and try again
        std::thread::sleep(std::time::Duration::from_millis(1100));
        let saved = auto_save.try_save(&state).unwrap();
        assert!(saved);
    }

    #[test]
    fn test_force_save() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("test-state.json");
        let persistence = StatePersistence::json(state_file);
        let auto_save = AutoSave::new(persistence, 60); // 60 second interval

        let state = create_test_state();

        // Force save should work immediately
        auto_save.force_save(&state).unwrap();
        assert!(auto_save.persistence.exists());
    }

    #[test]
    fn test_queued_url_serialization() {
        let queued = QueuedUrl {
            url: "https://example.com/test".to_string(),
            depth: 2,
            referrer: Some("https://example.com".to_string()),
        };

        let json = serde_json::to_string(&queued).unwrap();
        let deserialized: QueuedUrl = serde_json::from_str(&json).unwrap();

        assert_eq!(queued.url, deserialized.url);
        assert_eq!(queued.depth, deserialized.depth);
        assert_eq!(queued.referrer, deserialized.referrer);
    }
}
