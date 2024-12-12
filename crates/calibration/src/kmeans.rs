// TODO: Use SeedRng so we can use fuzzing for testing
use rand::seq::SliceRandom;
use rand::thread_rng;

/// kmeans clustering
///
/// kmeans clustering finds k-amount of groups inside of a slice of values
/// we use this for finding the values for the line and floor using calibration
///
/// returns an array of length values.len()
/// where each element is the index from 0..k showing which group the element
/// belongs to -> This can be used to calculate the average for each group
pub fn kmeans(values: &[f64], k: usize, max_iters: usize) -> Vec<usize> {
    let mut rng = thread_rng();

    let mut centroids: Vec<f64> = values.choose_multiple(&mut rng, k).cloned().collect();
    let mut assignments = vec![0; values.len()];

    for _ in 0..max_iters {
        // Step 2: Assign values to the nearest centroid
        for (i, &value) in values.iter().enumerate() {
            let mut min_dist = f64::MAX;
            let mut best_centroid = 0;
            for (j, &centroid) in centroids.iter().enumerate() {
                let dist = (value - centroid).abs();
                if dist < min_dist {
                    min_dist = dist;
                    best_centroid = j;
                }
            }
            assignments[i] = best_centroid;
        }

        // Step 3: Update centroids based on the assigned values
        let mut clusters = vec![vec![]; k];
        for (i, &cluster) in assignments.iter().enumerate() {
            clusters[cluster].push(values[i]);
        }

        for (i, cluster_values) in clusters.iter().enumerate() {
            if !cluster_values.is_empty() {
                centroids[i] =
                    cluster_values.iter().copied().sum::<f64>() / cluster_values.len() as f64;
            }
        }
    }

    assignments
}

/// Calculate the average of a cluster after kmeans
pub fn average_cluster_sizes(values: &[f64], assignments: &[usize], k: usize) -> Vec<f64> {
    // Group values by their assigned cluster
    let mut groups: Vec<Vec<f64>> = vec![vec![]; k];
    for (i, &cluster) in assignments.iter().enumerate() {
        groups[cluster].push(values[i]);
    }

    // Calculate the average for each group
    groups
        .iter()
        .map(|group| {
            if group.is_empty() {
                0.0
            } else {
                group.iter().sum::<f64>() / group.len() as f64
            }
        })
        .collect()
}
