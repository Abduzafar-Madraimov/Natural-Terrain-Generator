use std::time::Instant;

use core::{
    Fractal2D, NoiseGenerator, Perlin2D, Simplex2D, ThermalErosion2D,
    domain_warp::DomainWarp2D,
    utils::{flatten2, normalize2, to_terrain_image},
};
use eframe::{App, Frame, NativeOptions, egui, run_native};
use egui::{ColorImage, TextureHandle, Vec2};
use image::{ImageBuffer, Rgb};
use storage::Storage2D;
use storage::models::{TerrainDoc2D, TerrainParams};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum NoiseType {
    Fractal2D,
    Perlin2D,
    Simplex2D,
}
impl Default for NoiseType {
    fn default() -> Self {
        NoiseType::Fractal2D
    }
}
struct TerrainApp {
    // parameters
    noise_type: NoiseType,
    // slider is for n; size = 2^n + 1
    exp: u32,
    seed: u64,
    roughness: f64,
    erosion_iters: u32,
    frequency: f64,
    persistence: f64,
    octaves: u32,

    // erosion parameters
    enable_erosion: bool,
    talus_angle: f64,

    // domain warping parameters
    enable_warping: bool,
    warp_strength: f64,

    // generated texture
    terrain_texture: Option<TextureHandle>,

    // timing & status
    last_duration: Option<f32>,
    status_message: String,

    // Store the last RGB buffer
    last_flat: Option<Vec<u8>>,
    // Stores last size of the generated terrain
    last_size: usize,
}

impl Default for TerrainApp {
    fn default() -> Self {
        Self {
            exp: 7, // 2^7 + 1 = 129
            last_size: 129,
            seed: 2025,
            roughness: 1.0,
            erosion_iters: 5,
            terrain_texture: None,
            last_duration: None,
            status_message: String::new(),
            last_flat: None,
            noise_type: NoiseType::Fractal2D,
            frequency: 1.0,
            persistence: 0.5,
            octaves: 4,
            enable_erosion: true,
            talus_angle: 1.0,
            enable_warping: false,
            warp_strength: 0.5,
        }
    }
}

impl App for TerrainApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // compute real size
        let size = (1 << self.exp) + 1;

        egui::SidePanel::left("controls").show(ctx, |ui| {
            ui.heading("Terrain Generator");
            ui.separator();

            // Noise type selector
            ui.label("Noise Type");
            egui::ComboBox::from_label("Noise Algorithm")
                .selected_text(format!("{:?}", self.noise_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.noise_type, NoiseType::Fractal2D, "Fractal2D");
                    ui.selectable_value(&mut self.noise_type, NoiseType::Perlin2D, "Perlin2D");
                    ui.selectable_value(&mut self.noise_type, NoiseType::Simplex2D, "Simplex2D");
                });

            // Resolution slider
            let prev_size = (1 << self.exp) + 1;
            ui.horizontal(|ui| {
                ui.label("Resolution 2^n+1:");
                ui.add(
                    egui::Slider::new(&mut self.exp, 6..=9)
                        .text(format!("{}×{}", size, size))
                        .step_by(1.0),
                );
                if prev_size != size {
                    self.terrain_texture = None; // reset texture on size change
                    self.last_flat = None;
                    self.status_message = "Texture reset due to size change".into();
                }
            });

            // Seed
            ui.label("Seed");
            ui.add(egui::DragValue::new(&mut self.seed).speed(1.0));

            match self.noise_type {
                NoiseType::Fractal2D => {
                    ui.label("Roughness");
                    ui.add(egui::Slider::new(&mut self.roughness, 0.1..=5.0));
                }
                _ => {
                    ui.label("Frequency");
                    ui.add(egui::Slider::new(&mut self.frequency, 0.1..=10.0));

                    ui.label("Persistence");
                    ui.add(egui::Slider::new(&mut self.persistence, 0.0..=1.0));

                    ui.label("Octaves");
                    ui.add(egui::Slider::new(&mut self.octaves, 1..=8));
                }
            }

            // Domain warping
            if self.noise_type == NoiseType::Fractal2D {
                self.enable_warping = false; // Disable forcibly
                ui.add_enabled(
                    false,
                    egui::Checkbox::new(&mut self.enable_warping, "Enable Domain Warping"),
                );
                ui.label("Domain warping not supported for Fractal2D");
            } else {
                ui.checkbox(&mut self.enable_warping, "Enable Domain Warping");
                if self.enable_warping {
                    ui.add(
                        egui::Slider::new(&mut self.warp_strength, 0.0..=1.0).text("Warp Strength"),
                    );
                }
            }

            // Erosion
            if self.noise_type != NoiseType::Fractal2D {
                self.enable_erosion = false;
                ui.add_enabled(
                    false,
                    egui::Checkbox::new(&mut self.enable_erosion, "Apply Erosion"),
                );
                ui.label("Erosion only supported for Fractal2D");
            } else {
                ui.checkbox(&mut self.enable_erosion, "Apply Erosion");
                if self.enable_erosion {
                    ui.label("Erosion Iterations");
                    ui.add(egui::Slider::new(&mut self.erosion_iters, 0..=50));
                    ui.label("Talus Angle");
                    ui.add(egui::Slider::new(&mut self.talus_angle, 0.1..=5.0));
                }
            }

            ui.separator();

            // Generate & measure
            if ui.button("Generate Terrain").clicked() {
                let start = Instant::now();

                // Base Generator
                let mut fractal_base = Fractal2D::new(size, self.seed, self.roughness);
                let mut grid = match self.noise_type {
                    NoiseType::Fractal2D => {
                        let base = {
                            let _ = fractal_base.generate(); // fill internal map
                            &fractal_base
                        };

                        if self.enable_warping {
                            let mut fractal_warp =
                                Fractal2D::new(size, self.seed.wrapping_add(42), self.roughness);
                            let _ = fractal_warp.generate();
                            DomainWarp2D {
                                base,
                                warp: &fractal_warp,
                                size,
                                warp_strength: self.warp_strength,
                            }
                            .generate()
                        } else {
                            let mut g = vec![vec![0.0; size]; size];
                            for y in 0..size {
                                for x in 0..size {
                                    let fx = x as f64 / size as f64;
                                    let fy = y as f64 / size as f64;
                                    g[y][x] = base.get2(fx, fy) as f32;
                                }
                            }
                            g
                        }
                    }

                    NoiseType::Perlin2D | NoiseType::Simplex2D => {
                        let base: Box<dyn NoiseGenerator> = match self.noise_type {
                            NoiseType::Perlin2D => Box::new(Perlin2D::new(
                                self.seed,
                                self.frequency,
                                self.persistence,
                                self.octaves as usize,
                            )),
                            NoiseType::Simplex2D => Box::new(Simplex2D::new(
                                self.seed,
                                self.frequency,
                                self.persistence,
                                self.octaves as usize,
                            )),
                            _ => unreachable!(),
                        };

                        if self.enable_warping {
                            let warp: Box<dyn NoiseGenerator> = match self.noise_type {
                                NoiseType::Perlin2D => Box::new(Perlin2D::new(
                                    self.seed.wrapping_add(42),
                                    self.frequency,
                                    self.persistence,
                                    self.octaves as usize,
                                )),
                                NoiseType::Simplex2D => Box::new(Simplex2D::new(
                                    self.seed.wrapping_add(42),
                                    self.frequency,
                                    self.persistence,
                                    self.octaves as usize,
                                )),
                                _ => unreachable!(),
                            };

                            DomainWarp2D {
                                base: base.as_ref(),
                                warp: warp.as_ref(),
                                size,
                                warp_strength: self.warp_strength,
                            }
                            .generate()
                        } else {
                            let mut g = vec![vec![0.0; size]; size];
                            for y in 0..size {
                                for x in 0..size {
                                    let fx = x as f64 / size as f64;
                                    let fy = y as f64 / size as f64;
                                    g[y][x] = base.get2(fx, fy) as f32;
                                }
                            }
                            g
                        }
                    }
                };

                // Apply thermal erosion
                if self.enable_erosion {
                    ThermalErosion2D::new(self.erosion_iters as usize, self.talus_angle as f32)
                        .apply(&mut grid);
                }

                // Normalize only after erosion to avoid making erosion useless
                normalize2(&mut grid); // normalize so heights are in [0,1]
                let flat = flatten2(&grid);
                let img = to_terrain_image(&flat, size);
                self.last_flat = Some(img.clone());
                // Keep size in sync with flat
                self.last_size = size;
                let color_image = ColorImage::from_rgb([size, size], &img);
                self.terrain_texture =
                    Some(ctx.load_texture("terrain", color_image, egui::TextureOptions::NEAREST));
                self.last_duration = Some(start.elapsed().as_secs_f32() * 1000.0);
                self.status_message = format!(
                    "Generated in {:.2} ms (seed {})",
                    self.last_duration.unwrap(),
                    self.seed
                );
                ctx.request_repaint();
            }

            // Save to PNG
            if ui.button("Save PNG…").clicked() {
                if let Some(img) = &self.last_flat {
                    let filename = format!("terrain_{}.png", self.seed);
                    image::save_buffer(
                        &filename,
                        img,
                        size as u32,
                        size as u32,
                        image::ColorType::Rgb8,
                    )
                    .unwrap();
                    self.status_message = format!("Saved {}", filename);
                }
            }

            // Save to DB
            if ui.button("Save to DB…").clicked() {
                if let Some(tex) = &self.terrain_texture {
                    // reuse same flatten pipeline
                    let mut grid = Fractal2D::new(size, self.seed, self.roughness).generate();
                    ThermalErosion2D::new(self.erosion_iters as usize, 1.0).apply(&mut grid);
                    let flat = flatten2(&grid);
                    let params = TerrainParams {
                        noise_type: "fractal2d".into(),
                        frequency: 0.0,
                        persistence: 0.0,
                        octaves: 0,
                        roughness: Some(self.roughness),
                        erosion_iters: Some(self.erosion_iters),
                        talus_angle: Some(1.0),
                    };
                    let doc = TerrainDoc2D {
                        id: None,
                        seed: self.seed as i64,
                        params,
                        height_map: flat.clone(),
                        dimensions: 2,
                    };

                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap();

                    match rt.block_on(Storage2D::init(
                        "mongodb://localhost:27017",
                        "terrain_db",
                        "terrain2d",
                    )) {
                        Ok(mut storage) => {
                            let res = rt.block_on(storage.create(doc));
                            self.status_message = if res.is_ok() {
                                "Saved to MongoDB".into()
                            } else {
                                format!("DB error: {}", res.unwrap_err())
                            };
                        }
                        Err(e) => {
                            self.status_message = format!("DB init error: {}", e);
                        }
                    }
                }
            }

            // Load from DB
            if ui.button("Load from DB…").clicked() {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                match rt.block_on(Storage2D::init(
                    "mongodb://localhost:27017",
                    "terrain_db",
                    "terrain2d",
                )) {
                    Ok(mut storage) => {
                        let found = rt.block_on(storage.read_by_seed(self.seed as i64)).unwrap();
                        if let Some(doc) = found {
                            // reconstruct texture
                            let img = to_terrain_image(&doc.height_map, size);
                            let color_image = ColorImage::from_rgb([size, size], &img);
                            self.terrain_texture = Some(ctx.load_texture(
                                "terrain",
                                color_image,
                                egui::TextureOptions::NEAREST,
                            ));
                            self.status_message = "Loaded from MongoDB".into();
                        } else {
                            self.status_message = "No entry for this seed".into();
                        }
                    }
                    Err(e) => {
                        self.status_message = format!("DB init error: {}", e);
                    }
                }
                ctx.request_repaint();
            }

            ui.separator();
            ui.label(&self.status_message);
        });

        // central display
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(tex) = &self.terrain_texture {
                let available = ui.available_size();
                ui.image((tex.id(), available));
                ui.separator();
                ui.label("3D Preview:");
                // pull back your last‐computed f32 heights:
                let flat = match &self.last_flat {
                    Some(v) => v,
                    None => {
                        ui.label("no data");
                        return;
                    }
                };
                let hscale = 100.0;
                let angle = std::f32::consts::FRAC_PI_4; // 45°
                let (ca, sa) = (angle.cos(), angle.sin());

                // Build mesh:
                let mut verts = Vec::new();
                let mut inds = Vec::new();
                let mesh_size = self.last_size;
                for y in 0..mesh_size - 1 {
                    for x in 0..mesh_size - 1 {
                        let corners = [
                            (x as f32, flat[y * mesh_size + x] as f32 * hscale),
                            (x as f32 + 1.0, flat[y * mesh_size + x + 1] as f32 * hscale),
                            (x as f32, flat[(y + 1) * mesh_size + x] as f32 * hscale),
                            (
                                x as f32 + 1.0,
                                flat[(y + 1) * mesh_size + x + 1] as f32 * hscale,
                            ),
                        ];
                        for &(dx, h) in &corners {
                            // simple side‐view projection:
                            let px = dx;
                            let py = -h;
                            verts.push(egui::epaint::Vertex {
                                pos: egui::pos2(px, py),
                                uv: egui::pos2(0.0, 0.0),
                                color: egui::Color32::WHITE,
                            });
                        }
                        let base = verts.len() as u32 - 4;
                        inds.extend_from_slice(&[
                            base,
                            base + 1,
                            base + 2,
                            base + 1,
                            base + 3,
                            base + 2,
                        ]);
                    }
                }
                let mesh = egui::epaint::Mesh {
                    vertices: verts,
                    indices: inds,
                    texture_id: egui::TextureId::default(),
                };
                // add as a mesh Shape
                ui.painter().add(egui::epaint::Shape::mesh(mesh));
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Click “Generate” to start");
                });
            }
        });
    }
}

fn main() {
    let opts = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };
    run_native(
        "FYP Terrain Generator",
        opts,
        Box::new(|_cc| Ok(Box::new(TerrainApp::default()))),
    )
    .unwrap();
}
