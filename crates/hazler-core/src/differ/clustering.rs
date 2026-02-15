//! Response clustering using K-means and DBSCAN algorithms
//!
//! This module groups similar responses together to identify patterns
//! and anomalies in web application behavior.

use crate::differ::simhash::SimHash;
use serde::{Deserialize, Serialize};

/// A cluster of similar responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseCluster {
    /// Cluster ID
    pub id: usize,
    /// URLs in this cluster
    pub urls: Vec<String>,
    /// Representative hash for the cluster
    pub centroid: SimHash,
    /// Average similarity within cluster
    pub cohesion: f64,
}

/// Clustering algorithm type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClusteringAlgorithm {
    KMeans,
    DBSCAN,
}

/// K-means clustering for responses
pub struct KMeansClusterer {
    num_clusters: usize,
    max_iterations: usize,
}

impl KMeansClusterer {
    /// Create a new K-means clusterer
    pub fn new(num_clusters: usize) -> Self {
        Self {
            num_clusters,
            max_iterations: 100,
        }
    }

    /// Cluster responses by their SimHash values
    pub fn cluster(&self, responses: &[(String, SimHash)]) -> Vec<ResponseCluster> {
        if responses.is_empty() || self.num_clusters == 0 {
            return Vec::new();
        }

        let k = self.num_clusters.min(responses.len());

        // Initialize centroids using first k responses
        let mut centroids: Vec<SimHash> = responses.iter().take(k).map(|(_, hash)| *hash).collect();

        let mut assignments = vec![0usize; responses.len()];

        // K-means iterations
        for _ in 0..self.max_iterations {
            let mut changed = false;

            // Assignment step: assign each response to nearest centroid
            for (i, (_, hash)) in responses.iter().enumerate() {
                let mut best_cluster = 0;
                let mut best_distance = u32::MAX;

                for (c, centroid) in centroids.iter().enumerate() {
                    let distance = hash.hamming_distance(centroid);
                    if distance < best_distance {
                        best_distance = distance;
                        best_cluster = c;
                    }
                }

                if assignments[i] != best_cluster {
                    assignments[i] = best_cluster;
                    changed = true;
                }
            }

            if !changed {
                break; // Converged
            }

            // Update step: recompute centroids
            #[allow(clippy::needless_range_loop)]
            for c in 0..k {
                let cluster_hashes: Vec<SimHash> = responses
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| assignments[*i] == c)
                    .map(|(_, (_, hash))| *hash)
                    .collect();

                if !cluster_hashes.is_empty() {
                    centroids[c] = Self::compute_centroid(&cluster_hashes);
                }
            }
        }

        // Build cluster results
        let mut clusters = Vec::new();
        #[allow(clippy::needless_range_loop)]
        for c in 0..k {
            let cluster_items: Vec<(String, SimHash)> = responses
                .iter()
                .enumerate()
                .filter(|(i, _)| assignments[*i] == c)
                .map(|(_, item)| item.clone())
                .collect();

            if !cluster_items.is_empty() {
                let urls: Vec<String> = cluster_items.iter().map(|(url, _)| url.clone()).collect();
                let cohesion = Self::compute_cohesion(
                    &cluster_items.iter().map(|(_, h)| *h).collect::<Vec<_>>(),
                );

                clusters.push(ResponseCluster {
                    id: c,
                    urls,
                    centroid: centroids[c],
                    cohesion,
                });
            }
        }

        clusters
    }

    /// Compute centroid of a set of hashes (majority voting per bit)
    fn compute_centroid(hashes: &[SimHash]) -> SimHash {
        if hashes.is_empty() {
            return SimHash::new(0);
        }

        let mut bit_counts = [0i32; 64];

        for hash in hashes {
            #[allow(clippy::needless_range_loop)]
            for i in 0..64 {
                if (hash.0 >> i) & 1 == 1 {
                    bit_counts[i] += 1;
                } else {
                    bit_counts[i] -= 1;
                }
            }
        }

        let mut centroid = 0u64;

        #[allow(clippy::needless_range_loop)]
        for i in 0..64 {
            if bit_counts[i] > 0 {
                centroid |= 1u64 << i;
            }
        }

        SimHash::new(centroid)
    }

    /// Compute average cohesion (similarity) within a cluster
    fn compute_cohesion(hashes: &[SimHash]) -> f64 {
        if hashes.len() <= 1 {
            return 1.0;
        }

        let mut total_similarity = 0.0;
        let mut count = 0;

        for i in 0..hashes.len() {
            for j in (i + 1)..hashes.len() {
                total_similarity += hashes[i].similarity(&hashes[j]);
                count += 1;
            }
        }

        if count > 0 {
            total_similarity / count as f64
        } else {
            1.0
        }
    }
}

/// DBSCAN clustering for responses
pub struct DBSCANClusterer {
    epsilon: f64,
    min_points: usize,
}

impl DBSCANClusterer {
    /// Create a new DBSCAN clusterer
    pub fn new(epsilon: f64, min_points: usize) -> Self {
        Self {
            epsilon,
            min_points,
        }
    }

    /// Cluster responses using DBSCAN
    pub fn cluster(&self, responses: &[(String, SimHash)]) -> Vec<ResponseCluster> {
        if responses.is_empty() {
            return Vec::new();
        }

        let n = responses.len();
        let mut visited = vec![false; n];
        let mut cluster_ids = vec![-1i32; n]; // -1 = noise
        let mut cluster_id = 0;

        for i in 0..n {
            if visited[i] {
                continue;
            }
            visited[i] = true;

            let neighbors = self.find_neighbors(i, responses);

            if neighbors.len() < self.min_points {
                // Mark as noise
                cluster_ids[i] = -1;
            } else {
                // Start a new cluster
                self.expand_cluster(
                    i,
                    &neighbors,
                    cluster_id,
                    responses,
                    &mut visited,
                    &mut cluster_ids,
                );
                cluster_id += 1;
            }
        }

        // Build cluster results
        let mut clusters = Vec::new();
        for c in 0..cluster_id {
            let cluster_items: Vec<(String, SimHash)> = responses
                .iter()
                .enumerate()
                .filter(|(i, _)| cluster_ids[*i] == c)
                .map(|(_, item)| item.clone())
                .collect();

            if !cluster_items.is_empty() {
                let urls: Vec<String> = cluster_items.iter().map(|(url, _)| url.clone()).collect();
                let hashes: Vec<SimHash> = cluster_items.iter().map(|(_, h)| *h).collect();
                let centroid = KMeansClusterer::compute_centroid(&hashes);
                let cohesion = KMeansClusterer::compute_cohesion(&hashes);

                clusters.push(ResponseCluster {
                    id: c as usize,
                    urls,
                    centroid,
                    cohesion,
                });
            }
        }

        clusters
    }

    fn find_neighbors(&self, point: usize, responses: &[(String, SimHash)]) -> Vec<usize> {
        let mut neighbors = Vec::new();
        let point_hash = responses[point].1;

        for (i, (_, hash)) in responses.iter().enumerate() {
            let distance = 1.0 - point_hash.similarity(hash);
            if distance <= self.epsilon {
                neighbors.push(i);
            }
        }

        neighbors
    }

    fn expand_cluster(
        &self,
        point: usize,
        neighbors: &[usize],
        cluster_id: i32,
        responses: &[(String, SimHash)],
        visited: &mut [bool],
        cluster_ids: &mut [i32],
    ) {
        use std::collections::HashSet;

        cluster_ids[point] = cluster_id;

        let mut queue: Vec<usize> = neighbors.to_vec();
        let mut in_queue: HashSet<usize> = queue.iter().copied().collect();
        let mut idx = 0;

        while idx < queue.len() {
            let current = queue[idx];
            idx += 1;

            if !visited[current] {
                visited[current] = true;
                let current_neighbors = self.find_neighbors(current, responses);

                if current_neighbors.len() >= self.min_points {
                    for &neighbor in &current_neighbors {
                        if !in_queue.contains(&neighbor) {
                            queue.push(neighbor);
                            in_queue.insert(neighbor);
                        }
                    }
                }
            }

            if cluster_ids[current] == -1 {
                cluster_ids[current] = cluster_id;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_responses() -> Vec<(String, SimHash)> {
        vec![
            ("url1".to_string(), SimHash::new(0b1111)),
            ("url2".to_string(), SimHash::new(0b1110)),
            ("url3".to_string(), SimHash::new(0b0001)),
            ("url4".to_string(), SimHash::new(0b0010)),
        ]
    }

    #[test]
    fn test_kmeans_clustering() {
        let responses = create_test_responses();
        let clusterer = KMeansClusterer::new(2);
        let clusters = clusterer.cluster(&responses);

        assert!(!clusters.is_empty());
        assert!(clusters.len() <= 2);
    }

    #[test]
    fn test_kmeans_empty() {
        let responses = Vec::new();
        let clusterer = KMeansClusterer::new(2);
        let clusters = clusterer.cluster(&responses);

        assert!(clusters.is_empty());
    }

    #[test]
    fn test_kmeans_single_cluster() {
        let responses = vec![("url1".to_string(), SimHash::new(100))];
        let clusterer = KMeansClusterer::new(1);
        let clusters = clusterer.cluster(&responses);

        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].urls.len(), 1);
    }

    #[test]
    fn test_dbscan_clustering() {
        let responses = create_test_responses();
        let clusterer = DBSCANClusterer::new(0.3, 2);
        let clusters = clusterer.cluster(&responses);

        // DBSCAN might create different number of clusters based on density
        assert!(!clusters.is_empty() || responses.len() < 2);
    }

    #[test]
    fn test_dbscan_empty() {
        let responses = Vec::new();
        let clusterer = DBSCANClusterer::new(0.3, 2);
        let clusters = clusterer.cluster(&responses);

        assert!(clusters.is_empty());
    }

    #[test]
    fn test_compute_centroid() {
        let hashes = vec![
            SimHash::new(0b1111),
            SimHash::new(0b1110),
            SimHash::new(0b1100),
        ];

        let centroid = KMeansClusterer::compute_centroid(&hashes);
        assert!(centroid.0 > 0);
    }

    #[test]
    fn test_compute_cohesion() {
        let hashes = vec![SimHash::new(0b1111), SimHash::new(0b1111)];

        let cohesion = KMeansClusterer::compute_cohesion(&hashes);
        assert_eq!(cohesion, 1.0);
    }

    #[test]
    fn test_compute_cohesion_single() {
        let hashes = vec![SimHash::new(0b1111)];
        let cohesion = KMeansClusterer::compute_cohesion(&hashes);
        assert_eq!(cohesion, 1.0);
    }
}
