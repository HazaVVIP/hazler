//! BOLA/IDOR detection through response comparison

use serde::{Deserialize, Serialize};

/// Response comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseComparison {
    /// First response status code
    pub status_code_1: u16,
    /// Second response status code
    pub status_code_2: u16,
    /// First response size
    pub size_1: usize,
    /// Second response size
    pub size_2: usize,
    /// Similarity score (0.0 to 1.0)
    pub similarity: f64,
    /// Whether responses are suspiciously similar
    pub is_suspicious: bool,
    /// Reason for suspicion
    pub reason: Option<String>,
}

/// BOLA/IDOR detector
pub struct BolaDetector {
    similarity_threshold: f64,
}

impl Default for BolaDetector {
    fn default() -> Self {
        Self::new(0.9)
    }
}

impl BolaDetector {
    /// Create a new BOLA detector with similarity threshold
    pub fn new(similarity_threshold: f64) -> Self {
        Self {
            similarity_threshold,
        }
    }

    /// Compare two responses for BOLA/IDOR patterns
    pub fn compare_responses(
        &self,
        response1: &Response,
        response2: &Response,
    ) -> ResponseComparison {
        let status_same = response1.status_code == response2.status_code;
        let size_similarity = self.calculate_size_similarity(response1.size, response2.size);
        let content_similarity =
            self.calculate_content_similarity(&response1.body, &response2.body);

        let similarity = (size_similarity + content_similarity) / 2.0;

        let is_suspicious =
            status_same && response1.status_code == 200 && similarity > self.similarity_threshold;

        let reason = if is_suspicious {
            Some(format!(
                "Both responses returned 200 with {:.1}% similarity - potential BOLA/IDOR",
                similarity * 100.0
            ))
        } else {
            None
        };

        ResponseComparison {
            status_code_1: response1.status_code,
            status_code_2: response2.status_code,
            size_1: response1.size,
            size_2: response2.size,
            similarity,
            is_suspicious,
            reason,
        }
    }

    /// Calculate size similarity between two responses
    fn calculate_size_similarity(&self, size1: usize, size2: usize) -> f64 {
        if size1 == 0 && size2 == 0 {
            return 1.0;
        }

        let min = size1.min(size2) as f64;
        let max = size1.max(size2) as f64;

        if max == 0.0 {
            return 0.0;
        }

        min / max
    }

    /// Calculate content similarity using simple string comparison
    fn calculate_content_similarity(&self, content1: &str, content2: &str) -> f64 {
        if content1.is_empty() && content2.is_empty() {
            return 1.0;
        }

        if content1.is_empty() || content2.is_empty() {
            return 0.0;
        }

        // Use Levenshtein-like comparison (simplified)
        let len1 = content1.len();
        let len2 = content2.len();

        // Simple character-based similarity
        let min_len = len1.min(len2);
        let max_len = len1.max(len2);

        let mut matches = 0;
        let chars1: Vec<char> = content1.chars().collect();
        let chars2: Vec<char> = content2.chars().collect();

        for i in 0..min_len {
            if chars1[i] == chars2[i] {
                matches += 1;
            }
        }

        matches as f64 / max_len as f64
    }

    /// Detect potential BOLA by comparing multiple responses
    pub fn detect_bola(&self, responses: &[Response]) -> Vec<BolaFinding> {
        let mut findings = Vec::new();

        for i in 0..responses.len() {
            for j in (i + 1)..responses.len() {
                let comparison = self.compare_responses(&responses[i], &responses[j]);

                if comparison.is_suspicious {
                    findings.push(BolaFinding {
                        url1: responses[i].url.clone(),
                        url2: responses[j].url.clone(),
                        comparison,
                    });
                }
            }
        }

        findings
    }
}

/// Response data for comparison
#[derive(Debug, Clone)]
pub struct Response {
    /// URL of the response
    pub url: String,
    /// HTTP status code
    pub status_code: u16,
    /// Response body size
    pub size: usize,
    /// Response body content
    pub body: String,
}

impl Response {
    /// Create a new response
    pub fn new(url: String, status_code: u16, body: String) -> Self {
        let size = body.len();
        Self {
            url,
            status_code,
            size,
            body,
        }
    }
}

/// BOLA finding
#[derive(Debug, Clone)]
pub struct BolaFinding {
    /// First URL
    pub url1: String,
    /// Second URL
    pub url2: String,
    /// Comparison result
    pub comparison: ResponseComparison,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_similarity_identical() {
        let detector = BolaDetector::default();
        let similarity = detector.calculate_size_similarity(100, 100);
        assert_eq!(similarity, 1.0);
    }

    #[test]
    fn test_size_similarity_different() {
        let detector = BolaDetector::default();
        let similarity = detector.calculate_size_similarity(100, 200);
        assert_eq!(similarity, 0.5);
    }

    #[test]
    fn test_content_similarity_identical() {
        let detector = BolaDetector::default();
        let content = "Hello World";
        let similarity = detector.calculate_content_similarity(content, content);
        assert_eq!(similarity, 1.0);
    }

    #[test]
    fn test_content_similarity_different() {
        let detector = BolaDetector::default();
        let similarity = detector.calculate_content_similarity("Hello", "World");
        assert!(similarity < 0.5);
    }

    #[test]
    fn test_compare_responses_suspicious() {
        let detector = BolaDetector::new(0.8);

        let response1 = Response::new(
            "https://api.example.com/user/1".to_string(),
            200,
            "User data".to_string(),
        );

        let response2 = Response::new(
            "https://api.example.com/user/2".to_string(),
            200,
            "User data".to_string(),
        );

        let comparison = detector.compare_responses(&response1, &response2);

        assert!(
            comparison.is_suspicious,
            "Should detect suspicious similarity"
        );
        assert!(comparison.reason.is_some(), "Should provide reason");
    }

    #[test]
    fn test_compare_responses_not_suspicious() {
        let detector = BolaDetector::new(0.9);

        let response1 = Response::new(
            "https://api.example.com/user/1".to_string(),
            200,
            "User 1 data".to_string(),
        );

        let response2 = Response::new(
            "https://api.example.com/user/2".to_string(),
            404,
            "Not found".to_string(),
        );

        let comparison = detector.compare_responses(&response1, &response2);

        assert!(
            !comparison.is_suspicious,
            "Should not flag different statuses"
        );
    }

    #[test]
    fn test_detect_bola() {
        let detector = BolaDetector::new(0.8);

        let responses = vec![
            Response::new(
                "https://api.example.com/user/1".to_string(),
                200,
                "User data".to_string(),
            ),
            Response::new(
                "https://api.example.com/user/2".to_string(),
                200,
                "User data".to_string(),
            ),
            Response::new(
                "https://api.example.com/user/3".to_string(),
                200,
                "User data".to_string(),
            ),
        ];

        let findings = detector.detect_bola(&responses);

        assert!(!findings.is_empty(), "Should detect BOLA patterns");
    }
}
