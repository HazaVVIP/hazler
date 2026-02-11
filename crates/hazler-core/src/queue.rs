use std::collections::{HashSet, VecDeque};
use url::Url;

/// URL queue for managing crawl frontier
#[derive(Debug)]
pub struct UrlQueue {
    /// Queue of URLs to crawl with their depth
    queue: VecDeque<(Url, usize)>,
    /// Set of URLs already visited
    visited: HashSet<String>,
    /// Set of URLs already queued
    queued: HashSet<String>,
}

impl UrlQueue {
    /// Create a new empty queue
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            visited: HashSet::new(),
            queued: HashSet::new(),
        }
    }

    /// Add a URL to the queue if not already visited or queued
    pub fn push(&mut self, url: Url, depth: usize) -> bool {
        let url_str = url.as_str();
        
        if self.visited.contains(url_str) || self.queued.contains(url_str) {
            return false;
        }
        
        self.queued.insert(url_str.to_string());
        self.queue.push_back((url, depth));
        true
    }

    /// Get the next URL from the queue
    pub fn pop(&mut self) -> Option<(Url, usize)> {
        if let Some((url, depth)) = self.queue.pop_front() {
            let url_str = url.as_str();
            self.queued.remove(url_str);
            self.visited.insert(url_str.to_string());
            Some((url, depth))
        } else {
            None
        }
    }

    /// Mark a URL as visited without queuing it
    pub fn mark_visited(&mut self, url: &Url) {
        self.visited.insert(url.as_str().to_string());
    }

    /// Check if a URL has been visited
    pub fn is_visited(&self, url: &Url) -> bool {
        self.visited.contains(url.as_str())
    }

    /// Get the number of URLs in the queue
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Get the total number of URLs visited
    pub fn visited_count(&self) -> usize {
        self.visited.len()
    }
}

impl Default for UrlQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_push_pop() {
        let mut queue = UrlQueue::new();
        let url = Url::parse("https://example.com").unwrap();
        
        assert!(queue.push(url.clone(), 0));
        assert_eq!(queue.len(), 1);
        
        let (popped_url, depth) = queue.pop().unwrap();
        assert_eq!(popped_url, url);
        assert_eq!(depth, 0);
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_queue_deduplication() {
        let mut queue = UrlQueue::new();
        let url = Url::parse("https://example.com").unwrap();
        
        assert!(queue.push(url.clone(), 0));
        assert!(!queue.push(url.clone(), 0)); // Should not add duplicate
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_visited_tracking() {
        let mut queue = UrlQueue::new();
        let url = Url::parse("https://example.com").unwrap();
        
        assert!(!queue.is_visited(&url));
        queue.mark_visited(&url);
        assert!(queue.is_visited(&url));
        assert!(!queue.push(url, 0)); // Should not queue visited URL
    }
}
