use std::time::Instant;

use core::{
    Fractal2D, NoiseGenerator, Perlin2D, Simplex2D, ThermalErosion2D,
    domain_warp::DomainWarp2D,
    utils::{flatten2, normalize2, to_terrain_image},
};
use eframe::{App, Frame, NativeOptions, egui, run_native};
use egui::{ColorImage, TextureHandle};
use storage::Storage2D;
use storage::models::{TerrainDoc2D, TerrainParams};

const SPACE_LABEL: f32 = 5.0; // space between label and control
const SPACE_WIDGET: f32 = 8.0; // space between controls
const SPACE_RIGHT: f32 = 16.0; // space from the right edge
const MIN_EXP: u32 = 6;
const MAX_EXP: u32 = 9;

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
    // Last generated grid
    last_grid: Option<core::utils::HeightMap2D>,

    // Save name for terrain in DB
    save_name: String,
    load_list: Vec<String>,
    selected_name: Option<String>,
}

impl Default for TerrainApp {
    fn default() -> Self {
        let mut app = Self {
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
            save_name: String::new(),
            load_list: vec![],
            selected_name: None,
            last_grid: None,
        };
        // On startup, load the DB names
        app.refresh_name_list();
        app
    }
}

impl TerrainApp {
    // Helper to block-on list_names() and update `self.load_list` + status.
    fn refresh_name_list(&mut self) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        match rt.block_on(Storage2D::init(
            "mongodb://localhost:27017",
            "terrain_db",
            "terrain2d",
        )) {
            Ok(storage) => match rt.block_on(storage.list_names()) {
                Ok(names) => {
                    self.load_list = names;
                    self.status_message = "Loaded name list".to_owned();
                }
                Err(e) => {
                    self.status_message = format!("List error: {}", e);
                }
            },
            Err(e) => {
                self.status_message = format!("DB init error: {}", e);
            }
        }
    }
}

impl App for TerrainApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // compute real size
        let size = (1 << self.exp) + 1;
        let total_width = ctx.available_rect().width();
        let panel_width = total_width * 0.4;

        egui::SidePanel::left("controls")
            .resizable(false) // optional: lock panel width
            .min_width(panel_width) // force exact size
            .max_width(panel_width) // prevent stretching
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("Terrain Generator");
                    ui.separator();

                    // Noise Parameters
                    egui::CollapsingHeader::new("Noise Parameters")
                        .default_open(true)
                        .show(ui, |ui| {
                            // Seed
                            ui.label("Seed");
                            ui.add_space(SPACE_LABEL);
                            ui.add(egui::DragValue::new(&mut self.seed).speed(1.0));
                            ui.add_space(SPACE_WIDGET);

                            // Resolution slider
                            let prev_size = (1 << self.exp) + 1;
                            // ui.horizontal(|ui| {
                            ui.label("Resolution (2^n+1):");
                            ui.add_space(SPACE_LABEL);
                            // Stretch slider across entire panel width
                            ui.add_sized(
                                [ui.available_width(), 0.0],
                                egui::Slider::new(&mut self.exp, MIN_EXP..=MAX_EXP)
                                    .text(format!("{}×{}", size, size))
                                    .step_by(1.0),
                            );
                            if prev_size != size {
                                self.terrain_texture = None; // reset texture on size change
                                self.last_flat = None;
                                self.status_message = "Texture reset due to size change".into();
                            }
                            // });
                            ui.add_space(SPACE_WIDGET);

                            // Noise type selector
                            ui.label("Noise Type");
                            ui.add_space(SPACE_LABEL); // Add a 5 px vertical space
                            egui::ComboBox::from_id_salt("noise_type_combo")
                                .selected_text(format!("{:?}", self.noise_type))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.noise_type,
                                        NoiseType::Fractal2D,
                                        "Fractal2D",
                                    );
                                    ui.selectable_value(
                                        &mut self.noise_type,
                                        NoiseType::Perlin2D,
                                        "Perlin2D",
                                    );
                                    ui.selectable_value(
                                        &mut self.noise_type,
                                        NoiseType::Simplex2D,
                                        "Simplex2D",
                                    );
                                });
                            ui.add_space(SPACE_WIDGET);

                            // Parameters based on noise type
                            match self.noise_type {
                                NoiseType::Fractal2D => {
                                    ui.label("Roughness");
                                    ui.add_space(SPACE_LABEL);
                                    ui.add(egui::Slider::new(&mut self.roughness, 1.0..=5.0));
                                }
                                _ => {
                                    ui.label("Frequency");
                                    ui.add_space(SPACE_LABEL);
                                    ui.add(egui::Slider::new(&mut self.frequency, 0.1..=10.0));

                                    ui.label("Persistence");
                                    ui.add_space(SPACE_LABEL);
                                    ui.add(egui::Slider::new(&mut self.persistence, 0.0..=1.0));

                                    ui.label("Octaves");
                                    ui.add_space(SPACE_LABEL);
                                    ui.add(egui::Slider::new(&mut self.octaves, 1..=8));
                                }
                            }
                        });
                    ui.add_space(SPACE_WIDGET);

                    // Domain warping
                    egui::CollapsingHeader::new("Domain warping")
                        .default_open(true)
                        .show(ui, |ui| {
                            if self.noise_type == NoiseType::Fractal2D {
                                self.enable_warping = false; // Disable forcibly
                                ui.add_enabled(
                                    false,
                                    egui::Checkbox::new(
                                        &mut self.enable_warping,
                                        "Enable Domain Warping",
                                    ),
                                );
                                ui.label("Domain warping not supported for Fractal2D");
                            } else {
                                ui.checkbox(&mut self.enable_warping, "Enable Domain Warping");
                                if self.enable_warping {
                                    ui.add(
                                        egui::Slider::new(&mut self.warp_strength, 0.0..=1.0)
                                            .text("Warp Strength"),
                                    );
                                }
                            }
                        });

                    // Erosion
                    egui::CollapsingHeader::new("Erosion")
                        .default_open(true)
                        .show(ui, |ui| {
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
                        });

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
                                    let mut fractal_warp = Fractal2D::new(
                                        size,
                                        self.seed.wrapping_add(42),
                                        self.roughness,
                                    );
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
                            ThermalErosion2D::new(
                                self.erosion_iters as usize,
                                self.talus_angle as f32,
                            )
                            .apply(&mut grid);
                        }

                        // Normalize only after erosion to avoid making erosion useless
                        normalize2(&mut grid); // normalize so heights are in [0,1]
                        // Save the last grid
                        self.last_grid = Some(grid.clone());
                        let flat = flatten2(&grid);
                        let img = to_terrain_image(&flat, size);
                        self.last_flat = Some(img.clone());
                        // Keep size in sync with flat
                        self.last_size = size;
                        let color_image = ColorImage::from_rgb([size, size], &img);
                        self.terrain_texture = Some(ctx.load_texture(
                            "terrain",
                            color_image,
                            egui::TextureOptions::NEAREST,
                        ));
                        self.last_duration = Some(start.elapsed().as_secs_f32() * 1000.0);
                        self.status_message = format!(
                            "Generated in {:.2} ms (seed {})",
                            self.last_duration.unwrap(),
                            self.seed
                        );
                        ctx.request_repaint();
                    }
                    ui.add_space(SPACE_WIDGET);

                    // Terrain name and save options
                    ui.label("Terrain Name:");
                    ui.add_space(SPACE_LABEL);
                    ui.text_edit_singleline(&mut self.save_name);
                    ui.add_space(SPACE_WIDGET);

                    ui.horizontal(|ui| {
                        // Save to PNG
                        if ui.button("Save as PNG").clicked() {
                            if let Some(img) = &self.last_flat {
                                if let Some(path) = rfd::FileDialog::new()
                                    .set_title("Save Terrain as PNG")
                                    .set_directory(".")
                                    .set_file_name(&format!("terrain_{}.png", self.save_name))
                                    .save_file()
                                {
                                    image::save_buffer(
                                        &path,
                                        img,
                                        self.last_size as u32,
                                        self.last_size as u32,
                                        image::ColorType::Rgb8,
                                    )
                                    .unwrap();
                                    self.status_message =
                                        format!("Saved PNG to {}", path.display());
                                }
                            }
                        }
                        ui.add_space(SPACE_WIDGET);

                        // Save to DB
                        // Spacer to push the second button to the right
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(SPACE_RIGHT);
                            if ui.button("Save to Database").clicked() {
                                if self.save_name.trim().is_empty() {
                                    self.status_message =
                                        "Terrain is not stored \nTerrain name is required".into();
                                    return;
                                }
                                if let Some(grid) = &self.last_grid {
                                    // flatten the stored grid
                                    let flat = flatten2(grid);
                                    let params = TerrainParams {
                                        noise_type: format!("{:?}", self.noise_type).to_lowercase(),
                                        frequency: self.frequency,
                                        persistence: self.persistence,
                                        octaves: self.octaves as usize,
                                        roughness: Some(self.roughness),
                                        erosion_iters: Some(self.erosion_iters),
                                        talus_angle: Some(self.talus_angle as f32),
                                        warp_strength: Some(self.warp_strength),
                                    };
                                    let doc = TerrainDoc2D {
                                        id: None,
                                        name: self.save_name.clone(),
                                        seed: self.seed as i64,
                                        params,
                                        height_map: flat,
                                        dimensions: 2,
                                    };

                                    let success = {
                                        let rt = tokio::runtime::Builder::new_current_thread()
                                            .enable_all()
                                            .build()
                                            .unwrap();
                                        rt.block_on(Storage2D::init(
                                            "mongodb://localhost:27017",
                                            "terrain_db",
                                            "terrain2d",
                                        ))
                                        .and_then(|storage| rt.block_on(storage.create(doc)))
                                        .is_ok()
                                    };

                                    if success {
                                        self.status_message = "Saved to MongoDB".into();
                                        // 2) Immediately re‑load the name list:
                                        self.refresh_name_list();
                                    } else {
                                        self.status_message = "Failed to save to MongoDB".into();
                                    }
                                } else {
                                    self.status_message =
                                        "No terrain to save or name is empty".into();
                                }
                            }
                        });
                    });
                    ui.add_space(SPACE_WIDGET);

                    // Load from DB
                    if ui.button("Refresh DB List").clicked() {
                        self.refresh_name_list();
                    }
                    ui.add_space(SPACE_WIDGET);
                    // Draw ComboBox
                    ui.horizontal(|ui| {
                        ui.label("Load terrain:");
                        ui.add_space(SPACE_LABEL);
                        egui::ComboBox::from_label("")
                            .selected_text(
                                self.selected_name
                                    .as_ref()
                                    .map(|s| s.as_str())
                                    .unwrap_or("<none>"),
                            )
                            .show_ui(ui, |ui| {
                                for name in &self.load_list {
                                    ui.selectable_value(
                                        &mut self.selected_name,
                                        Some(name.clone()),
                                        name,
                                    );
                                }
                            });
                    });
                    ui.add_space(SPACE_WIDGET);
                    // Add a “Load Selected” button
                    if ui.button("Load Selected").clicked() {
                        if let Some(name) = &self.selected_name {
                            let rt = tokio::runtime::Builder::new_current_thread()
                                .enable_all()
                                .build()
                                .unwrap();
                            match rt.block_on(Storage2D::init(
                                "mongodb://localhost:27017",
                                "terrain_db",
                                "terrain2d",
                            )) {
                                Ok(storage) => match rt.block_on(storage.read_by_name(name)) {
                                    Ok(Some(doc)) => {
                                        // compute size from flattened length
                                        let len = doc.height_map.len();
                                        let size = (len as f64).sqrt() as usize;
                                        assert!(
                                            size * size == len,
                                            "stored height_map length must be square"
                                        );
                                        // update last_size and last_flat
                                        self.last_size = size;
                                        self.last_flat = Some(
                                            doc.height_map
                                                .clone()
                                                .iter()
                                                .map(|&v| (v * 255.0) as u8)
                                                .collect(),
                                        );

                                        // rebuild texture:
                                        let img = to_terrain_image(&doc.height_map, self.last_size);
                                        let color_image = ColorImage::from_rgb(
                                            [self.last_size, self.last_size],
                                            &img,
                                        );
                                        self.terrain_texture = Some(ctx.load_texture(
                                            "terrain",
                                            color_image,
                                            egui::TextureOptions::NEAREST,
                                        ));
                                        self.status_message = format!("Loaded “{}”", name);

                                        // Sync configuration with loaded terrain parameters
                                        let params = &doc.params;
                                        self.seed = doc.seed as u64;
                                        self.save_name = doc.name.clone();
                                        // Update resolution exponent based on map size
                                        self.exp = match size {
                                            129 => 7,
                                            257 => 8,
                                            513 => 9,
                                            _ => self.exp, // fallback to current if unknown
                                        };
                                        // Update noise type
                                        self.noise_type = match params.noise_type.as_str() {
                                            "fractal2d" => NoiseType::Fractal2D,
                                            "perlin2d" => NoiseType::Perlin2D,
                                            "simplex2d" => NoiseType::Simplex2D,
                                            _ => self.noise_type,
                                        };
                                        // Common parameters
                                        self.frequency = params.frequency;
                                        self.persistence = params.persistence;
                                        self.octaves = params.octaves as u32;
                                        self.roughness = params.roughness.unwrap_or(self.roughness);
                                        // Erosion
                                        self.erosion_iters =
                                            params.erosion_iters.unwrap_or(self.erosion_iters);
                                        self.talus_angle =
                                            params.talus_angle.unwrap_or(self.talus_angle as f32)
                                                as f64;
                                        self.enable_erosion =
                                            self.noise_type == NoiseType::Fractal2D;
                                        // Domain Warping
                                        self.warp_strength =
                                            params.warp_strength.unwrap_or(self.warp_strength);
                                        self.enable_warping =
                                            self.noise_type != NoiseType::Fractal2D;
                                    }
                                    Ok(None) => self.status_message = "Name not found".into(),
                                    Err(e) => self.status_message = format!("Read error: {}", e),
                                },
                                Err(e) => self.status_message = format!("DB init error: {}", e),
                            }
                        } else {
                            self.status_message = "No terrain selected".into();
                        }
                    }
                    ui.add_space(SPACE_WIDGET);

                    ui.separator();
                    ui.label(&self.status_message);
                });
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
                let (_ca, _sa) = (angle.cos(), angle.sin());

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
