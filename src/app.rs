use crate::art::*;
use crate::sortfns::*;
use directories::UserDirs;
use egui::ComboBox;
use egui::{Button, ColorImage, Frame, TextureHandle, Vec2};
use image::{
    imageops::{self, FilterType},
    RgbaImage,
};
use std::path::PathBuf;

const SPACE: f32 = 10.0;

fn dims(width: f32, height: f32) -> (f32, f32) {
    if width.max(height) <= 1200.0 {
        return (width, height);
    }
    let aspect_ratio = height / width;
    if width >= height {
        (1200.0, (1200.0 * aspect_ratio))
    } else {
        ((1200.0 / aspect_ratio), 1200.0)
    }
}

fn to_color_image(img: &RgbaImage, width: u32, height: u32) -> ColorImage {
    let img = imageops::resize(img, width, height, FilterType::Lanczos3);
    ColorImage::from_rgba_unmultiplied(
        [img.width() as usize, img.height() as usize],
        &img.into_vec(),
    )
}
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct PixelUnsortApp {
    #[serde(skip)]
    texture: Option<TextureHandle>,
    #[serde(skip)]
    img: RgbaImage,

    sort_img_path: Option<String>,
    unsort_img_path: Option<String>,
    sort_by: SortBy,
    sort_key: SortKey,
    row_sort_order: SortOrder,
    col_sort_order: SortOrder,
    pre_sort: bool,
}

impl Default for PixelUnsortApp {
    fn default() -> Self {
        Self {
            sort_img_path: None,
            unsort_img_path: None,
            img: RgbaImage::new(1, 1),
            texture: None,
            sort_by: SortBy::Row,
            sort_key: SortKey::Lightness,
            row_sort_order: SortOrder::Ascending,
            col_sort_order: SortOrder::Ascending,
            pre_sort: false,
        }
    }
}

impl PixelUnsortApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for PixelUnsortApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel")
            .exact_width(300.0)
            .resizable(false)
            .frame(Frame::default().inner_margin(10.0))
            .show(ctx, |ui| {
                ui.heading("Controls");
                ui.separator();
                ui.add_space(SPACE);
                if ui
                    .add(Button::new("Sort Image Path").min_size(Vec2::new(125.0, 25.0)))
                    .clicked()
                {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("image", &["png", "jpg", "jpeg"])
                        .pick_file()
                    {
                        self.sort_img_path = Some(path.display().to_string());
                    }
                }
                ui.add_space(SPACE);
                if let Some(picked_path) = &self.sort_img_path {
                    ui.label(picked_path);
                }
                ui.add_space(SPACE);
                ui.separator();
                ui.add_space(SPACE);
                if ui
                    .add(Button::new("Unsort Image Path").min_size(Vec2::new(125.0, 25.0)))
                    .clicked()
                {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("image", &["png", "jpg", "jpeg"])
                        .pick_file()
                    {
                        self.unsort_img_path = Some(path.display().to_string());
                    }
                }
                ui.add_space(SPACE);
                if let Some(picked_path) = &self.unsort_img_path {
                    ui.label(picked_path);
                }
                ui.add_space(SPACE);
                ui.separator();
                ui.add_space(SPACE);
                ui.checkbox(&mut self.pre_sort, "Pre-Sort");
                ui.add_space(SPACE);
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.sort_by, SortBy::Row, "Rows");
                    ui.radio_value(&mut self.sort_by, SortBy::Column, "Columns");
                    ui.radio_value(&mut self.sort_by, SortBy::ColRow, "ColRow");
                    ui.radio_value(&mut self.sort_by, SortBy::RowCol, "RowCol");
                    ui.radio_value(&mut self.sort_by, SortBy::Nothing, "Nothing");
                });
                ui.add_space(SPACE);
                ComboBox::from_label("Sort Key")
                    .width(125.0)
                    .selected_text(format!("{:?}", self.sort_key))
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.set_min_width(60.0);
                        ui.selectable_value(&mut self.sort_key, SortKey::Lightness, "Lightness");
                        ui.selectable_value(&mut self.sort_key, SortKey::Hue, "Hue");
                        ui.selectable_value(&mut self.sort_key, SortKey::Saturation, "Saturation");
                    });
                ui.add_space(SPACE);
                ComboBox::from_label("Row Order")
                    .width(125.0)
                    .selected_text(format!("{:?}", self.row_sort_order))
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.set_min_width(60.0);
                        ui.selectable_value(
                            &mut self.row_sort_order,
                            SortOrder::Ascending,
                            "Ascending",
                        );
                        ui.selectable_value(
                            &mut self.row_sort_order,
                            SortOrder::Descending,
                            "Descending",
                        );
                    });
                ui.add_space(SPACE);
                ComboBox::from_label("Column Order")
                    .width(125.0)
                    .selected_text(format!("{:?}", self.col_sort_order))
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.set_min_width(60.0);
                        ui.selectable_value(
                            &mut self.col_sort_order,
                            SortOrder::Ascending,
                            "Ascending",
                        );
                        ui.selectable_value(
                            &mut self.col_sort_order,
                            SortOrder::Descending,
                            "Descending",
                        );
                    });
                ui.add_space(SPACE);
                if ui
                    .add(Button::new("Swap Images").min_size(Vec2::new(125.0, 25.0)))
                    .clicked()
                {
                    (self.sort_img_path, self.unsort_img_path) =
                        (self.unsort_img_path.clone(), self.sort_img_path.clone());
                }
                ui.add_space(SPACE);
                ui.separator();
                ui.add_space(2.0 * SPACE);
                if let Some(sort_path) = &self.sort_img_path {
                    if let Some(unsort_path) = &self.unsort_img_path {
                        ui.vertical_centered(|ui| {
                            if ui
                                .add(Button::new("Unsort Image").min_size(Vec2::new(125.0, 25.0)))
                                .clicked()
                            {
                                if let Ok(img1) = image::open(sort_path) {
                                    if let Ok(img2) = image::open(unsort_path) {
                                        let size = dims(img1.width() as f32, img1.height() as f32);
                                        self.img = draw(
                                            &img1,
                                            &img2,
                                            self.sort_by,
                                            self.sort_key,
                                            DrawType::Unsort,
                                            self.row_sort_order,
                                            self.col_sort_order,
                                            self.pre_sort,
                                        );
                                        self.texture = Some(ui.ctx().load_texture(
                                            "unsort",
                                            to_color_image(&self.img, size.0 as u32, size.1 as u32),
                                            Default::default(),
                                        ));
                                    }
                                }
                            };
                            ui.add_space(2.0 * SPACE);
                            if ui
                                .add(Button::new("Sort Image").min_size(Vec2::new(125.0, 25.0)))
                                .clicked()
                            {
                                if let Ok(img1) = image::open(sort_path) {
                                    let size = dims(img1.width() as f32, img1.height() as f32);
                                    self.img = draw(
                                        &img1,
                                        &img1,
                                        self.sort_by,
                                        self.sort_key,
                                        DrawType::Sort,
                                        self.row_sort_order,
                                        self.col_sort_order,
                                        self.pre_sort,
                                    );
                                    self.texture = Some(ui.ctx().load_texture(
                                        "unsort",
                                        to_color_image(&self.img, size.0 as u32, size.1 as u32),
                                        Default::default(),
                                    ));
                                }
                            }
                        });
                    }
                };
                ui.add_space(2.0 * SPACE);
                ui.vertical_centered(|ui| {
                    if ui
                        .add(Button::new("Save png").min_size(Vec2::new(125.0, 25.0)))
                        .clicked()
                    {
                        let dirs = UserDirs::new().unwrap();
                        let dir = dirs.download_dir().unwrap();
                        let path = format!(r"{}/{}", dir.to_string_lossy(), "pixel_unsort");
                        let mut num = 0;
                        let mut sketch = PathBuf::from(format!(r"{path}_{num}"));
                        sketch.set_extension("png");
                        while sketch.exists() {
                            num += 1;
                            sketch = PathBuf::from(format!(r"{path}_{num}"));
                            sketch.set_extension("png");
                        }
                        self.img.save(sketch).unwrap();
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| ui.heading("Pixel Unsort"));
            ui.add_space(SPACE);
            egui::warn_if_debug_build(ui);
            if let Some(txt) = &self.texture {
                let img_size = txt.size_vec2();
                let size = dims(img_size[0], img_size[1]);
                ui.horizontal(|ui| {
                    ui.add_space(SPACE);
                    ui.add_sized(egui::vec2(size.0, size.1), egui::Image::new(txt, img_size));
                });
            }
        });
    }
}
