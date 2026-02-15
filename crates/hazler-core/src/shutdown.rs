/// Graceful shutdown handler for Ctrl+C and signal handling
/// Ensures state is saved and resources are cleaned up
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::signal;
use tracing::{info, warn};

/// Shutdown signal handler
#[derive(Clone)]
pub struct ShutdownHandler {
    shutdown_flag: Arc<AtomicBool>,
}

impl ShutdownHandler {
    /// Create a new shutdown handler
    pub fn new() -> Self {
        Self {
            shutdown_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start listening for shutdown signals
    pub fn listen(&self) -> tokio::task::JoinHandle<()> {
        let flag = self.shutdown_flag.clone();

        tokio::spawn(async move {
            match signal::ctrl_c().await {
                Ok(()) => {
                    info!("Received shutdown signal (Ctrl+C)");
                    flag.store(true, Ordering::SeqCst);
                }
                Err(err) => {
                    warn!("Unable to listen for shutdown signal: {}", err);
                }
            }
        })
    }

    /// Check if shutdown has been requested
    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown_flag.load(Ordering::SeqCst)
    }

    /// Request shutdown programmatically
    pub fn request_shutdown(&self) {
        info!("Shutdown requested");
        self.shutdown_flag.store(true, Ordering::SeqCst);
    }

    /// Reset the shutdown flag
    pub fn reset(&self) {
        self.shutdown_flag.store(false, Ordering::SeqCst);
    }
}

impl Default for ShutdownHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Graceful shutdown coordinator
pub struct GracefulShutdown {
    handler: ShutdownHandler,
    cleanup_callbacks: Arc<std::sync::Mutex<Vec<Box<dyn FnOnce() + Send>>>>,
}

impl GracefulShutdown {
    /// Create a new graceful shutdown coordinator
    pub fn new() -> Self {
        Self {
            handler: ShutdownHandler::new(),
            cleanup_callbacks: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    /// Get the shutdown handler
    pub fn handler(&self) -> ShutdownHandler {
        self.handler.clone()
    }

    /// Register a cleanup callback
    pub fn on_shutdown<F>(&self, callback: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let mut callbacks = self.cleanup_callbacks.lock().unwrap();
        callbacks.push(Box::new(callback));
    }

    /// Start listening for shutdown signals
    pub fn listen(&self) -> tokio::task::JoinHandle<()> {
        self.handler.listen()
    }

    /// Check if shutdown has been requested
    pub fn is_shutdown_requested(&self) -> bool {
        self.handler.is_shutdown_requested()
    }

    /// Execute cleanup callbacks
    pub fn cleanup(&self) {
        info!("Executing shutdown cleanup...");
        let mut callbacks = self.cleanup_callbacks.lock().unwrap();

        for callback in callbacks.drain(..) {
            callback();
        }

        info!("Cleanup completed");
    }
}

impl Default for GracefulShutdown {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shutdown_handler_creation() {
        let handler = ShutdownHandler::new();
        assert!(!handler.is_shutdown_requested());
    }

    #[test]
    fn test_request_shutdown() {
        let handler = ShutdownHandler::new();
        assert!(!handler.is_shutdown_requested());

        handler.request_shutdown();
        assert!(handler.is_shutdown_requested());
    }

    #[test]
    fn test_reset_shutdown() {
        let handler = ShutdownHandler::new();
        handler.request_shutdown();
        assert!(handler.is_shutdown_requested());

        handler.reset();
        assert!(!handler.is_shutdown_requested());
    }

    #[test]
    fn test_graceful_shutdown_creation() {
        let shutdown = GracefulShutdown::new();
        assert!(!shutdown.is_shutdown_requested());
    }

    #[test]
    fn test_cleanup_callbacks() {
        let shutdown = GracefulShutdown::new();
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let counter_clone = counter.clone();
        shutdown.on_shutdown(move || {
            counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });

        let counter_clone = counter.clone();
        shutdown.on_shutdown(move || {
            counter_clone.fetch_add(10, std::sync::atomic::Ordering::SeqCst);
        });

        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 0);

        shutdown.cleanup();
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 11);
    }

    #[test]
    fn test_handler_clone() {
        let handler = ShutdownHandler::new();
        let handler_clone = handler.clone();

        handler.request_shutdown();
        assert!(handler_clone.is_shutdown_requested());
    }
}
