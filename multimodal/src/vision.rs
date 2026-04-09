use image::{DynamicImage, GenericImageView};
use core_vsa::HyperVector;
use crate::{SemanticVector, Modality};

pub struct VisualGrounding;

#[derive(Debug, Clone)]
pub struct VisualFeature {
    pub name: String,
    pub value: f32,
    pub vector: HyperVector,
}

impl VisualGrounding {
    pub fn analyze_image(img: &DynamicImage, dim: usize) -> SemanticVector {
        let (w, h) = img.dimensions();
        let mut features = Vec::new();

        // 1. Color Analysis (Simplified)
        let (r, g, b) = Self::calculate_average_color(img);
        features.push(VisualFeature {
            name: "color_red".to_string(),
            value: r,
            vector: HyperVector::deterministic_dim(0x1001, dim),
        });
        features.push(VisualFeature {
            name: "color_green".to_string(),
            value: g,
            vector: HyperVector::deterministic_dim(0x1002, dim),
        });
        features.push(VisualFeature {
            name: "color_blue".to_string(),
            value: b,
            vector: HyperVector::deterministic_dim(0x1003, dim),
        });

        // 2. Structural Analysis (Brightness, Edge Density)
        let brightness = Self::calculate_brightness(img);
        let edge_density = Self::calculate_edge_density(img);

        features.push(VisualFeature {
            name: "brightness".to_string(),
            value: brightness,
            vector: HyperVector::deterministic_dim(0x2001, dim),
        });
        features.push(VisualFeature {
            name: "edge_density".to_string(),
            value: edge_density,
            vector: HyperVector::deterministic_dim(0x2002, dim),
        });

        // 3. Construct Unified Visual Vector
        let mut visual_vector = HyperVector::deterministic_dim(0xBA5E, dim);
        for feat in &features {
            // Bind feature identity with its quantized value
            let val_hash = seahash::hash(&feat.value.to_be_bytes());
            let val_vec = HyperVector::deterministic_dim(val_hash, dim);
            visual_vector = visual_vector.bundle(&feat.vector.bind(&val_vec));
        }

        let label = format!("Image ({}x{}): avg_color=({:.1},{:.1},{:.1}), brightness={:.2}, edges={:.2}",
            w, h, r*255.0, g*255.0, b*255.0, brightness, edge_density);

        SemanticVector::new(visual_vector, Modality::Vision, label, "VisualGrounding")
    }

    fn calculate_average_color(img: &DynamicImage) -> (f32, f32, f32) {
        let mut r = 0u64;
        let mut g = 0u64;
        let mut b = 0u64;
        let (w, h) = img.dimensions();
        let total = (w * h) as u64;
        if total == 0 { return (0.0, 0.0, 0.0); }

        for y in 0..h {
            for x in 0..w {
                let p = img.get_pixel(x, y);
                r += p.0[0] as u64;
                g += p.0[1] as u64;
                b += p.0[2] as u64;
            }
        }

        (
            (r as f32 / total as f32) / 255.0,
            (g as f32 / total as f32) / 255.0,
            (b as f32 / total as f32) / 255.0,
        )
    }

    fn calculate_brightness(img: &DynamicImage) -> f32 {
        let gray = img.to_luma8();
        let sum: u64 = gray.pixels().map(|p| p.0[0] as u64).sum();
        let total = (img.width() * img.height()) as f32;
        if total == 0.0 { return 0.0; }
        (sum as f32 / total) / 255.0
    }

    fn calculate_edge_density(img: &DynamicImage) -> f32 {
        let gray = img.to_luma8();
        let mut edges = 0;
        let (w, h) = img.dimensions();
        if w < 2 || h < 2 { return 0.0; }
        for y in 1..(h - 1) {
            for x in 1..(w - 1) {
                let p = gray.get_pixel(x, y).0[0];
                let px = gray.get_pixel(x + 1, y).0[0];
                let py = gray.get_pixel(x, y + 1).0[0];
                if (p as i16 - px as i16).abs() > 40 || (p as i16 - py as i16).abs() > 40 {
                    edges += 1;
                }
            }
        }
        edges as f32 / (w * h) as f32
    }

    pub fn detect_objects(img: &DynamicImage) -> Vec<String> {
        let mut objects = Vec::new();
        let (w, h) = img.dimensions();

        // Geometric grouping: Analyze quadrants for distinct high-entropy regions
        let qw = w / 2;
        let qh = h / 2;

        for y_off in 0..2 {
            for x_off in 0..2 {
                let sub = img.crop_imm(x_off * qw, y_off * qh, qw, qh);
                let edge_density = Self::calculate_edge_density(&sub);
                let brightness = Self::calculate_brightness(&sub);

                if edge_density > 0.1 {
                    objects.push(format!("Structured object in quadrant ({},{})", x_off, y_off));
                }
                if brightness > 0.8 {
                    objects.push(format!("Bright region in quadrant ({},{})", x_off, y_off));
                }
            }
        }

        objects
    }
}
