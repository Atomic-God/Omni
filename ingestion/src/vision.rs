use image::{DynamicImage, GenericImageView};
use core_vsa::HyperVector;
use tracing::info;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct VisualRegion {
    pub bounds: (u32, u32, u32, u32), // x, y, w, h
    pub edge_density: f32,
    pub entropy: f32,
    pub brightness: f32,
}

#[derive(Debug, Clone)]
pub struct VisualAtom {
    pub kind: String, // "Line", "Region"
    pub intensity: f32,
    pub spatial_hv: HyperVector,
}

pub struct VisionSemanticExtractor;

impl VisionSemanticExtractor {
    pub fn extract_deep_meaning(img: &DynamicImage, dim: usize) -> (HyperVector, String, HashMap<String, String>) {
        info!("Vision Core: Deep structural analysis of {}x{} image", img.width(), img.height());

        let mut metadata = HashMap::new();

        // 1. Spatial Quadrant Analysis
        let quadrants = Self::analyze_quadrants(img);
        let mut scene_vector = HyperVector::deterministic_dim(0x5CE11E, dim);

        for (i, region) in quadrants.iter().enumerate() {
            let region_hv = HyperVector::deterministic_dim(
                ((region.edge_density * 1000.0) as u64) ^ ((region.brightness * 1000.0) as u64),
                dim
            );
            let pos_hv = HyperVector::deterministic_dim(i as u64, dim);
            scene_vector = scene_vector.bundle(&region_hv.bind(&pos_hv));

            metadata.insert(format!("q{}_density", i), format!("{:.2}", region.edge_density));
            metadata.insert(format!("q{}_complexity", i), format!("{:.2}", region.entropy));
        }

        // 2. Visual Grammar: Extract "Atoms"
        let atoms = Self::extract_visual_grammar(img, dim);
        metadata.insert("visual_atom_count".to_string(), atoms.len().to_string());

        for atom in &atoms {
            let atom_hv = HyperVector::deterministic_dim(
                ((atom.intensity * 100.0) as u64) ^ atom.kind.len() as u64,
                dim
            );
            scene_vector = scene_vector.bundle(&atom_hv.bind(&atom.spatial_hv));
        }

        // 3. Global Symmetry & Balance
        let horiz_symmetry = Self::calculate_symmetry(img, true);
        let vert_symmetry = Self::calculate_symmetry(img, false);
        metadata.insert("horiz_symmetry".to_string(), format!("{:.2}", horiz_symmetry));
        metadata.insert("vert_symmetry".to_string(), format!("{:.2}", vert_symmetry));

        // 4. Meaning Construction
        let mut meaning = String::new();
        if horiz_symmetry > 0.8 && vert_symmetry > 0.8 {
            meaning.push_str("Centrally balanced industrial subject. ");
        }

        let high_detail = atoms.iter().filter(|a| a.kind == "Line").count();
        if high_detail > 20 {
            meaning.push_str("Highly structured technical schematic or architectural form. ");
        } else if atoms.iter().any(|a| a.kind == "Region") && high_detail < 5 {
            meaning.push_str("Organic or uniform industrial material profile. ");
        }

        let conceptual_entropy = Self::calculate_conceptual_entropy(&quadrants, &atoms);
        metadata.insert("conceptual_entropy".to_string(), format!("{:.4}", conceptual_entropy));

        (scene_vector, meaning, metadata)
    }

    fn extract_visual_grammar(img: &DynamicImage, dim: usize) -> Vec<VisualAtom> {
        let mut atoms = Vec::new();
        let gray = img.to_luma8();
        let (w, h) = img.dimensions();

        for y in (4..h-4).step_by((h / 10).max(1) as usize) {
            for x in (4..w-4).step_by((w / 10).max(1) as usize) {
                let mut p = [0u8; 9];
                let mut i = 0;
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        p[i] = gray.get_pixel((x as i32 + dx) as u32, (y as i32 + dy) as u32).0[0];
                        i += 1;
                    }
                }

                if (p[0] as i16 - p[2] as i16).abs() > 50 || (p[0] as i16 - p[6] as i16).abs() > 50 {
                    atoms.push(VisualAtom {
                        kind: "Line".to_string(),
                        intensity: p[4] as f32 / 255.0,
                        spatial_hv: HyperVector::deterministic_dim((x ^ y) as u64, dim),
                    });
                } else if p.iter().all(|&v| (v as i16 - p[4] as i16).abs() < 10) {
                    if p[4] > 10 {
                        atoms.push(VisualAtom {
                            kind: "Region".to_string(),
                            intensity: p[4] as f32 / 255.0,
                            spatial_hv: HyperVector::deterministic_dim((x ^ y) as u64, dim),
                        });
                    }
                }
            }
        }
        atoms
    }

    fn calculate_conceptual_entropy(quads: &[VisualRegion], atoms: &[VisualAtom]) -> f32 {
        let mut detail_dist = quads.iter().map(|q| q.edge_density).collect::<Vec<_>>();
        detail_dist.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let variance = detail_dist.last().unwrap_or(&0.0) - detail_dist.first().unwrap_or(&0.0);
        let atom_complexity = (atoms.len() as f32 / 100.0).min(1.0);

        variance * 0.5 + atom_complexity * 0.5
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
        if total == 0 { return 0.0; }
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
        if total == 0.0 { return 0.0; }
        counts.iter().filter(|&&c| c > 0).map(|&c| {
            let p = c as f32 / total;
            -p * p.log2()
        }).sum::<f32>() / 8.0
    }

    fn calculate_brightness(img: &DynamicImage) -> f32 {
        let gray = img.to_luma8();
        let sum: u64 = gray.pixels().map(|p| p.0[0] as u64).sum();
        let total = (img.width() * img.height()) as f32;
        if total == 0.0 { return 0.0; }
        (sum as f32 / total) / 255.0
    }

    fn calculate_symmetry(img: &DynamicImage, horizontal: bool) -> f32 {
        let gray = img.to_luma8();
        let (w, h) = img.dimensions();
        let mut diff = 0u64;
        let total_comparisons;

        if horizontal {
            total_comparisons = (w / 2) * h;
            if total_comparisons == 0 { return 1.0; }
            for y in 0..h {
                for x in 0..(w / 2) {
                    let p1 = gray.get_pixel(x, y).0[0];
                    let p2 = gray.get_pixel(w - 1 - x, y).0[0];
                    diff += (p1 as i16 - p2 as i16).abs() as u64;
                }
            }
        } else {
            total_comparisons = w * (h / 2);
            if total_comparisons == 0 { return 1.0; }
            for y in 0..(h / 2) {
                for x in 0..w {
                    let p1 = gray.get_pixel(x, y).0[0];
                    let p2 = gray.get_pixel(x, h - 1 - y).0[0];
                    diff += (p1 as i16 - p2 as i16).abs() as u64;
                }
            }
        }
        (1.0 - (diff as f32 / (total_comparisons as f32 * 255.0))).max(0.0)
    }
}
