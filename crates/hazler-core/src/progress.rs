/// Progress tracking and reporting for crawl operations
/// Provides real-time feedback on crawling progress
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::info;

/// Progress statistics for crawl operations
#[derive(Debug, Clone, Default)]
pub struct ProgressStats {
    /// Total URLs discovered
    pub urls_discovered: usize,
    /// Total URLs visited
    pub urls_visited: usize,
    /// Total URLs in queue
    pub urls_queued: usize,
    /// Total errors encountered
    pub errors: usize,
    /// Start time of crawl
    pub start_time: Option<Instant>,
    /// Current crawl rate (pages per second)
    pub crawl_rate: f64,
    /// Estimated time remaining
    pub eta_seconds: Option<u64>,
}

impl ProgressStats {
    /// Calculate elapsed time
    pub fn elapsed(&self) -> Option<Duration> {
        self.start_time.map(|start| start.elapsed())
    }

    /// Calculate progress percentage (if max_pages is known)
    pub fn percentage(&self, max_pages: usize) -> Option<f64> {
        if max_pages > 0 {
            Some((self.urls_visited as f64 / max_pages as f64) * 100.0)
        } else {
            None
        }
    }

    /// Update crawl rate based on elapsed time
    pub fn update_rate(&mut self) {
        if let Some(elapsed) = self.elapsed() {
            let seconds = elapsed.as_secs_f64();
            if seconds > 0.0 {
                self.crawl_rate = self.urls_visited as f64 / seconds;
            }
        }
    }

    /// Estimate time remaining
    pub fn estimate_eta(&mut self, max_pages: usize) {
        if max_pages > 0 && self.crawl_rate > 0.0 {
            let remaining = max_pages.saturating_sub(self.urls_visited);
            let seconds = remaining as f64 / self.crawl_rate;
            self.eta_seconds = Some(seconds as u64);
        }
    }
}

/// Progress tracker for monitoring crawl progress
pub struct ProgressTracker {
    stats: Arc<Mutex<ProgressStats>>,
    max_pages: usize,
    report_interval: Duration,
    last_report: Arc<Mutex<Instant>>,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new(max_pages: usize) -> Self {
        let stats = ProgressStats {
            start_time: Some(Instant::now()),
            ..Default::default()
        };

        Self {
            stats: Arc::new(Mutex::new(stats)),
            max_pages,
            report_interval: Duration::from_secs(5),
            last_report: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Create with custom report interval
    pub fn with_interval(max_pages: usize, interval_secs: u64) -> Self {
        let mut tracker = Self::new(max_pages);
        tracker.report_interval = Duration::from_secs(interval_secs);
        tracker
    }

    /// Record a URL discovery
    pub fn record_discovered(&self) {
        let mut stats = self.stats.lock().unwrap();
        stats.urls_discovered += 1;
    }

    /// Record a URL visit
    pub fn record_visited(&self) {
        let mut stats = self.stats.lock().unwrap();
        stats.urls_visited += 1;
        stats.update_rate();
        stats.estimate_eta(self.max_pages);
    }

    /// Record a URL queued
    pub fn record_queued(&self) {
        let mut stats = self.stats.lock().unwrap();
        stats.urls_queued += 1;
    }

    /// Record URL dequeued
    pub fn record_dequeued(&self) {
        let mut stats = self.stats.lock().unwrap();
        stats.urls_queued = stats.urls_queued.saturating_sub(1);
    }

    /// Record an error
    pub fn record_error(&self) {
        let mut stats = self.stats.lock().unwrap();
        stats.errors += 1;
    }

    /// Get current statistics
    pub fn stats(&self) -> ProgressStats {
        self.stats.lock().unwrap().clone()
    }

    /// Check if it's time to report progress
    pub fn should_report(&self) -> bool {
        let last = self.last_report.lock().unwrap();
        last.elapsed() >= self.report_interval
    }

    /// Report progress (if interval elapsed)
    pub fn try_report(&self) {
        if self.should_report() {
            self.report();
            let mut last = self.last_report.lock().unwrap();
            *last = Instant::now();
        }
    }

    /// Force progress report
    pub fn report(&self) {
        let stats = self.stats();

        let elapsed = stats.elapsed().unwrap_or(Duration::ZERO);
        let elapsed_secs = elapsed.as_secs();

        let percentage = stats
            .percentage(self.max_pages)
            .map(|p| format!(" ({:.1}%)", p))
            .unwrap_or_default();

        let eta = stats
            .eta_seconds
            .map(|s| format!(", ETA: {}s", s))
            .unwrap_or_default();

        info!(
            "Progress: {} visited, {} queued, {} errors | {:.2} pages/sec | Elapsed: {}s{}{}",
            stats.urls_visited,
            stats.urls_queued,
            stats.errors,
            stats.crawl_rate,
            elapsed_secs,
            percentage,
            eta
        );
    }

    /// Get final summary
    pub fn summary(&self) -> String {
        let stats = self.stats();
        let elapsed = stats.elapsed().unwrap_or(Duration::ZERO);

        format!(
            "Crawl completed: {} URLs visited, {} discovered, {} errors in {:.1}s ({:.2} pages/sec)",
            stats.urls_visited,
            stats.urls_discovered,
            stats.errors,
            elapsed.as_secs_f64(),
            stats.crawl_rate
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_stats_creation() {
        let stats = ProgressStats::default();
        assert_eq!(stats.urls_discovered, 0);
        assert_eq!(stats.urls_visited, 0);
        assert_eq!(stats.errors, 0);
    }

    #[test]
    fn test_progress_tracker_creation() {
        let tracker = ProgressTracker::new(100);
        let stats = tracker.stats();
        assert_eq!(stats.urls_visited, 0);
        assert!(stats.start_time.is_some());
    }

    #[test]
    fn test_record_visited() {
        let tracker = ProgressTracker::new(100);
        tracker.record_visited();
        tracker.record_visited();

        let stats = tracker.stats();
        assert_eq!(stats.urls_visited, 2);
    }

    #[test]
    fn test_record_discovered() {
        let tracker = ProgressTracker::new(100);
        tracker.record_discovered();
        tracker.record_discovered();
        tracker.record_discovered();

        let stats = tracker.stats();
        assert_eq!(stats.urls_discovered, 3);
    }

    #[test]
    fn test_record_queued_dequeued() {
        let tracker = ProgressTracker::new(100);
        tracker.record_queued();
        tracker.record_queued();
        tracker.record_queued();
        assert_eq!(tracker.stats().urls_queued, 3);

        tracker.record_dequeued();
        assert_eq!(tracker.stats().urls_queued, 2);
    }

    #[test]
    fn test_record_error() {
        let tracker = ProgressTracker::new(100);
        tracker.record_error();
        tracker.record_error();

        let stats = tracker.stats();
        assert_eq!(stats.errors, 2);
    }

    #[test]
    fn test_percentage_calculation() {
        let mut stats = ProgressStats::default();
        stats.urls_visited = 50;

        assert_eq!(stats.percentage(100), Some(50.0));
        assert_eq!(stats.percentage(200), Some(25.0));
        assert_eq!(stats.percentage(0), None);
    }

    #[test]
    fn test_crawl_rate_calculation() {
        let mut stats = ProgressStats::default();
        stats.start_time = Some(Instant::now() - Duration::from_secs(10));
        stats.urls_visited = 20;

        stats.update_rate();
        assert!(stats.crawl_rate > 1.5 && stats.crawl_rate < 2.5); // ~2.0 pages/sec
    }

    #[test]
    fn test_should_report_interval() {
        let tracker = ProgressTracker::with_interval(100, 1);

        // Should not report immediately
        assert!(!tracker.should_report());

        // Wait and check again
        std::thread::sleep(Duration::from_millis(1100));
        assert!(tracker.should_report());
    }

    #[test]
    fn test_summary() {
        let tracker = ProgressTracker::new(100);
        tracker.record_visited();
        tracker.record_visited();
        tracker.record_discovered();
        tracker.record_discovered();
        tracker.record_discovered();

        let summary = tracker.summary();
        assert!(summary.contains("2 URLs visited"));
        assert!(summary.contains("3 discovered"));
    }

    #[test]
    fn test_eta_estimation() {
        let mut stats = ProgressStats::default();
        stats.start_time = Some(Instant::now() - Duration::from_secs(10));
        stats.urls_visited = 20;
        stats.update_rate();

        stats.estimate_eta(100);
        assert!(stats.eta_seconds.is_some());
        let eta = stats.eta_seconds.unwrap();
        assert!(eta > 30 && eta < 50); // Should be around 40 seconds
    }
}
