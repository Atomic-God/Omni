use image::{DynamicImage, GenericImageView};
use core_vsa::HyperVector;
use log::info;
use std::collections::HashMap;

pub struct VisionSemanticExtractor;

impl VisionSemanticExtractor {
    pub fn extract_meaning(img: &DynamicImage) -> (HyperVector, String, HashMap<String, String>) {
        info!("Vision Core: Extracting structural meaning from {}x{} image", img.width(), img.height());

        let mut metadata = HashMap::new();
        let (width, height) = img.dimensions();
        metadata.insert("width".to_string(), width.to_string());
        metadata.insert("height".to_string(), height.to_string());

        // 1. Edge Density Analysis (Structural)
        let edge_score = Self::calculate_edge_density(img);
        metadata.insert("edge_density".to_string(), format!("{:.4}", edge_score));

        // 2. Spatial Distribution (Symmetry/Complexity)
        let complexity = Self::calculate_complexity(img);
        metadata.insert("complexity_index".to_string(), format!("{:.4}", complexity));

        // 3. Color Profile (Simulated via Luma variance)
        let luma_variance = Self::calculate_luma_variance(img);
        metadata.insert("luma_variance".to_string(), format!("{:.4}", luma_variance));

        // Semantic Description based on structural features
        let description = if edge_score > 0.15 {
            if complexity > 0.6 {
                "Highly structured/complex scene (potential text or architecture)"
            } else {
                "Regular structured scene (potential geometric objects)"
            }
        } else {
            if luma_variance < 0.05 {
                "Low contrast/plain visual region"
            } else {
                "Natural/Organic visual distribution"
            }
        };

        // Map features to a deterministic HyperVector (Symbolic Mapping)
        // We use bits of the scores to seed the vector
        let seed = ((edge_score * 1000.0) as u64) ^ ((complexity * 1000.0) as u64) ^ ((luma_variance * 1000.0) as u64);
        let vector = HyperVector::deterministic(seed);

        (vector, description.to_string(), metadata)
    }

    fn calculate_edge_density(img: &DynamicImage) -> f32 {
        let gray = img.to_luma8();
        let mut edges = 0;
        let total_pixels = img.width() * img.height();

        // Simple Sobel-like gradient detection in one pass
        for y in 1..(img.height() - 1) {
            for x in 1..(img.width() - 1) {
                let p = gray.get_pixel(x, y).0[0];
                let px = gray.get_pixel(x + 1, y).0[0];
                let py = gray.get_pixel(x, y + 1).0[0];

                if (p as i16 - px as i16).abs() > 30 || (p as i16 - py as i16).abs() > 30 {
                    edges += 1;
                }
            }
        }
        edges as f32 / total_pixels as f32
    }

    fn calculate_complexity(img: &DynamicImage) -> f32 {
        // Spatial entropy approximation
        let gray = img.to_luma8();
        let mut counts = [0u32; 256];
        for p in gray.pixels() {
            counts[p.0[0] as usize] += 1;
        }

        let total = (img.width() * img.height()) as f32;
        let mut entropy = 0.0;
        for &c in counts.iter() {
            if c > 0 {
                let p = c as f32 / total;
                entropy -= p * p.log2();
            }
        }
        entropy / 8.0 // Normalized to 0..1
    }

    fn calculate_luma_variance(img: &DynamicImage) -> f32 {
        let gray = img.to_luma8();
        let mut sum = 0.0;
        let mut sum_sq = 0.0;
        let n = (img.width() * img.height()) as f32;

        for p in gray.pixels() {
            let val = p.0[0] as f32 / 255.0;
            sum += val;
            sum_sq += val * val;
        }

        let mean = sum / n;
        (sum_sq / n - mean * mean).max(0.0)
    }
}
