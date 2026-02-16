use image::{DynamicImage, GenericImageView};
use core_vsa::HyperVector;
use log::info;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct VisualRegion {
    pub bounds: (u32, u32, u32, u32), // x, y, w, h
    pub edge_density: f32,
    pub entropy: f32,
    pub brightness: f32,
}

pub struct VisionSemanticExtractor;

impl VisionSemanticExtractor {
    pub fn extract_deep_meaning(img: &DynamicImage) -> (HyperVector, String, HashMap<String, String>) {
        info!("Vision Core: Deep structural analysis of {}x{} image", img.width(), img.height());

        let mut metadata = HashMap::new();
        let (_width, _height) = img.dimensions();

        // 1. Spatial Quadrant Analysis
        let quadrants = Self::analyze_quadrants(img);
        let mut scene_vector = HyperVector::random(); // Base scene HV

        for (i, region) in quadrants.iter().enumerate() {
            let region_hv = HyperVector::deterministic_dim(
                ((region.edge_density * 100.0) as u64) ^ ((region.brightness * 100.0) as u64),
                core_vsa::DIMENSION
            );
            // Bind region HV with a position HV
            let pos_hv = HyperVector::deterministic(i as u64);
            scene_vector = scene_vector.bundle(&region_hv.bind(&pos_hv));

            metadata.insert(format!("q{}_density", i), format!("{:.2}", region.edge_density));
            metadata.insert(format!("q{}_brightness", i), format!("{:.2}", region.brightness));
        }

        // 2. Global Symmetry Detection
        let horiz_symmetry = Self::calculate_symmetry(img, true);
        let vert_symmetry = Self::calculate_symmetry(img, false);
        metadata.insert("horiz_symmetry".to_string(), format!("{:.2}", horiz_symmetry));
        metadata.insert("vert_symmetry".to_string(), format!("{:.2}", vert_symmetry));

        // 3. Meaning Construction
        let mut meaning = String::new();
        if horiz_symmetry > 0.8 && vert_symmetry > 0.8 {
            meaning.push_str("Highly symmetric/centered object detected. ");
        } else if horiz_symmetry > 0.8 {
            meaning.push_str("Horizontal symmetry detected (potential landscape or balanced scene). ");
        }

        let avg_density: f32 = quadrants.iter().map(|q| q.edge_density).sum::<f32>() / 4.0;
        if avg_density > 0.2 {
            meaning.push_str("Detailed/High-information industrial visual. ");
        } else {
            meaning.push_str("Low-detail or clean visual region. ");
        }

        (scene_vector, meaning, metadata)
    }

    fn analyze_quadrants(img: &DynamicImage) -> Vec<VisualRegion> {
        let (w, h) = img.dimensions();
        let qw = w / 2;
        let qh = h / 2;
        let mut regions = Vec::new();

        for y_off in 0..2 {
            for x_off in 0..2 {
                let sub = img.crop_imm(x_off * qw, y_off * qh, qw, qh);
                regions.push(VisualRegion {
                    bounds: (x_off * qw, y_off * qh, qw, qh),
                    edge_density: Self::calculate_edge_density(&sub),
                    entropy: Self::calculate_entropy(&sub),
                    brightness: Self::calculate_brightness(&sub),
                });
            }
        }
        regions
    }

    fn calculate_edge_density(img: &DynamicImage) -> f32 {
        let gray = img.to_luma8();
        let mut edges = 0;
        let total = img.width() * img.height();
        for y in 1..(img.height() - 1) {
            for x in 1..(img.width() - 1) {
                let p = gray.get_pixel(x, y).0[0];
                let px = gray.get_pixel(x + 1, y).0[0];
                let py = gray.get_pixel(x, y + 1).0[0];
                if (p as i16 - px as i16).abs() > 40 || (p as i16 - py as i16).abs() > 40 {
                    edges += 1;
                }
            }
        }
        edges as f32 / total as f32
    }

    fn calculate_entropy(img: &DynamicImage) -> f32 {
        let gray = img.to_luma8();
        let mut counts = [0u32; 256];
        for p in gray.pixels() { counts[p.0[0] as usize] += 1; }
        let total = (img.width() * img.height()) as f32;
        counts.iter().filter(|&&c| c > 0).map(|&c| {
            let p = c as f32 / total;
            -p * p.log2()
        }).sum::<f32>() / 8.0
    }

    fn calculate_brightness(img: &DynamicImage) -> f32 {
        let gray = img.to_luma8();
        let sum: u64 = gray.pixels().map(|p| p.0[0] as u64).sum();
        (sum as f32 / (img.width() * img.height()) as f32) / 255.0
    }

    fn calculate_symmetry(img: &DynamicImage, horizontal: bool) -> f32 {
        let gray = img.to_luma8();
        let (w, h) = img.dimensions();
        let mut diff = 0u64;
        let total_comparisons;

        if horizontal {
            total_comparisons = (w / 2) * h;
            for y in 0..h {
                for x in 0..(w / 2) {
                    let p1 = gray.get_pixel(x, y).0[0];
                    let p2 = gray.get_pixel(w - 1 - x, y).0[0];
                    diff += (p1 as i16 - p2 as i16).abs() as u64;
                }
            }
        } else {
            total_comparisons = w * (h / 2);
            for y in 0..(h / 2) {
                for x in 0..w {
                    let p1 = gray.get_pixel(x, y).0[0];
                    let p2 = gray.get_pixel(x, h - 1 - y).0[0];
                    diff += (p1 as i16 - p2 as i16).abs() as u64;
                }
            }
        }
        1.0 - (diff as f32 / (total_comparisons as f32 * 255.0))
    }
}
