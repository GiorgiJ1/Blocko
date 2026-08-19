use eframe::egui;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Visual Coding Language",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(VisualLangApp::new()))
        }),
    )
}

#[derive(Debug, Clone)]
struct ToolboxItem {
    name: String,
    description: String,
}

struct VisualLangApp {
    toolbox_items: Vec<ToolboxItem>,
    toolbox_width: f32,
    canvas_zoom: f32,
    status_message: String,
}

impl VisualLangApp {
    fn new() -> Self {
        let toolbox_items = vec![
            ToolboxItem {
                name: "Print".to_string(),
                description: "Outputs a value to the console".to_string(),
            },
            ToolboxItem {
                name: "Variable".to_string(),
                description: "Stores a value".to_string(),
            },
            ToolboxItem {
                name: "If / Else".to_string(),
                description: "Conditional branching".to_string(),
            },
            ToolboxItem {
                name: "Loop".to_string(),
                description: "Repeats a block of logic".to_string(),
            },
            ToolboxItem {
                name: "Math".to_string(),
                description: "Basic arithmetic operations".to_string(),
            },
        ];

        Self {
            toolbox_items,
            toolbox_width: 220.0,
            canvas_zoom: 1.0,
            status_message: "Ready.".to_string(),
        }
    }
}

impl eframe::App for VisualLangApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Project").clicked() {
                        self.status_message = "New project created (placeholder).".to_string();
                        ui.close_menu();
                    }
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });
                ui.menu_button("View", |ui| {
                    if ui.button("Reset Zoom").clicked() {
                        self.canvas_zoom = 1.0;
                        ui.close_menu();
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Status: {}", self.status_message));
                ui.separator();
                ui.label(format!("Zoom: {:.0}%", self.canvas_zoom * 100.0));
            });
        });

        egui::SidePanel::left("toolbox_panel")
            .resizable(true)
            .default_width(self.toolbox_width)
            .width_range(150.0..=400.0)
            .show(ctx, |ui| {
                ui.heading("Toolbox");
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for item in &self.toolbox_items {
                        let response = ui.add(
                            egui::Button::new(&item.name)
                                .min_size(egui::vec2(ui.available_width(), 30.0)),
                        );

                        response.clone().on_hover_text(&item.description);

                        if response.clicked() {
                            self.status_message =
                                format!("Selected block: {}", item.name);
                        }

                        if response.dragged() {
                            self.status_message =
                                format!("Dragging block: {} (drop-to-canvas not yet implemented)", item.name);
                        }

                        ui.add_space(4.0);
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Canvas");
            ui.separator();

            let available_rect = ui.available_rect_before_wrap();
            let painter = ui.painter();

            painter.rect_filled(
                available_rect,
                0.0,
                egui::Color32::from_rgb(30, 30, 34),
            );

            let grid_spacing = 24.0;
            let dot_color = egui::Color32::from_gray(55);
            let mut x = available_rect.left();
            while x < available_rect.right() {
                let mut y = available_rect.top();
                while y < available_rect.bottom() {
                    painter.circle_filled(egui::pos2(x, y), 1.0, dot_color);
                    y += grid_spacing;
                }
                x += grid_spacing;
            }

            ui.allocate_rect(available_rect, egui::Sense::hover());
        });
    }
}