use eframe::egui;
use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use egui::epaint::CubicBezierShape;
use std::collections::HashMap;

type NodeId = u64;

#[derive(Debug, Clone)]
enum NodeKind {
    Number(f32),
    Add,
    Print,
}

impl NodeKind {
    fn title(&self) -> &'static str {
        match self {
            NodeKind::Number(_) => "Number",
            NodeKind::Add => "Math (Add)",
            NodeKind::Print => "Print",
        }
    }

    fn input_labels(&self) -> Vec<&'static str> {
        match self {
            NodeKind::Number(_) => vec![],
            NodeKind::Add => vec!["A", "B"],
            NodeKind::Print => vec!["In"],
        }
    }

    fn output_labels(&self) -> Vec<&'static str> {
        match self {
            NodeKind::Number(_) => vec!["Value"],
            NodeKind::Add => vec!["Result"],
            NodeKind::Print => vec![],
        }
    }
}

struct Node {
    id: NodeId,
    kind: NodeKind,
    pos: Pos2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PinKind {
    Input,
    Output,
}

#[derive(Debug, Clone, Copy, Eq, Hash)]
struct PinRef {
    node_id: NodeId,
    kind: PinKind,
    index: usize,
}

#[derive(Debug, Clone, Copy)]
struct Connection {
    from: PinRef,
    to: PinRef,
}

struct DraggingConnection {
    from: PinRef,
    current_pos: Pos2,
}

const NODE_WIDTH: f32 = 170.0;
const TITLE_HEIGHT: f32 = 28.0;
const ROW_HEIGHT: f32 = 24.0;
const BODY_PADDING: f32 = 8.0;
const PIN_RADIUS: f32 = 6.0;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Blocko",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(BlockoApp::new()))
        }),
    )
}

struct BlockoApp {
    nodes: HashMap<NodeId, Node>,
    connections: Vec<Connection>,
    next_id: NodeId,
    pin_positions: HashMap<PinRef, Pos2>,
    dragging_connection: Option<DraggingConnection>,
    status_message: String,
}

impl BlockoApp {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            connections: Vec::new(),
            next_id: 0,
            pin_positions: HashMap::new(),
            dragging_connection: None,
            status_message: "Ready.".to_string(),
        }
    }

    fn add_node(&mut self, kind: NodeKind) {
        let id = self.next_id;
        self.next_id += 1;

        let count = self.nodes.len() as f32;
        let pos = Pos2::new(
            60.0 + (count * 40.0) % 400.0,
            60.0 + (count * 30.0) % 300.0,
        );

        self.status_message = format!("Added node: {}", kind.title());
        self.nodes.insert(id, Node { id, kind, pos });
    }

    fn node_height(kind: &NodeKind) -> f32 {
        let rows = kind.input_labels().len().max(kind.output_labels().len()).max(1);
        TITLE_HEIGHT + rows as f32 * ROW_HEIGHT + BODY_PADDING * 2.0
    }

    fn remove_connections_for(&mut self, node_id: NodeId) {
        self.connections
            .retain(|c| c.from.node_id != node_id && c.to.node_id != node_id);
    }
}

impl eframe::App for BlockoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Clear Canvas").clicked() {
                        self.nodes.clear();
                        self.connections.clear();
                        self.pin_positions.clear();
                        self.status_message = "Canvas cleared.".to_string();
                        ui.close_menu();
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Status: {}", self.status_message));
                ui.separator();
                ui.label(format!("Nodes: {}", self.nodes.len()));
                ui.separator();
                ui.label(format!("Connections: {}", self.connections.len()));
            });
        });

        egui::SidePanel::left("toolbox_panel")
            .resizable(true)
            .default_width(220.0)
            .width_range(150.0..=400.0)
            .show(ctx, |ui| {
                ui.heading("Toolbox");
                ui.separator();

                if ui
                    .add(egui::Button::new("Add Number").min_size(egui::vec2(ui.available_width(), 30.0)))
                    .clicked()
                {
                    self.add_node(NodeKind::Number(0.0));
                }

                ui.add_space(4.0);

                if ui
                    .add(egui::Button::new("Add Math (Add)").min_size(egui::vec2(ui.available_width(), 30.0)))
                    .clicked()
                {
                    self.add_node(NodeKind::Add);
                }

                ui.add_space(4.0);

                if ui
                    .add(egui::Button::new("Add Print").min_size(egui::vec2(ui.available_width(), 30.0)))
                    .clicked()
                {
                    self.add_node(NodeKind::Print);
                }

                ui.add_space(12.0);
                ui.separator();
                ui.label("Coming soon:");
                ui.add_enabled(false, egui::Button::new("If / Else").min_size(egui::vec2(ui.available_width(), 26.0)));
                ui.add_space(4.0);
                ui.add_enabled(false, egui::Button::new("Loop").min_size(egui::vec2(ui.available_width(), 26.0)));
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let origin = ui.min_rect().min;
            let canvas_rect = ui.max_rect();

            let painter = ui.painter();
            painter.rect_filled(canvas_rect, 0.0, Color32::from_rgb(30, 30, 34));

            let grid_spacing = 24.0;
            let dot_color = Color32::from_gray(48);
            let mut x = canvas_rect.left();
            while x < canvas_rect.right() {
                let mut y = canvas_rect.top();
                while y < canvas_rect.bottom() {
                    painter.circle_filled(Pos2::new(x, y), 1.0, dot_color);
                    y += grid_spacing;
                }
                x += grid_spacing;
            }

            ui.interact(canvas_rect, ui.id().with("canvas_bg"), Sense::click());

            for conn in &self.connections {
                if let (Some(&from_pos), Some(&to_pos)) =
                    (self.pin_positions.get(&conn.from), self.pin_positions.get(&conn.to))
                {
                    draw_wire(ui.painter(), from_pos, to_pos, Color32::from_rgb(120, 180, 255));
                }
            }

            let mut node_ids: Vec<NodeId> = self.nodes.keys().copied().collect();
            node_ids.sort_unstable();

            let mut nodes_to_disconnect: Vec<NodeId> = Vec::new();
            let mut new_connection: Option<Connection> = None;
            let mut cancel_drag = false;

            for node_id in node_ids {
                let node = self.nodes.get_mut(&node_id).unwrap();
                let height = BlockoApp::node_height(&node.kind);
                let screen_pos = origin + node.pos.to_vec2();
                let node_rect = Rect::from_min_size(screen_pos, Vec2::new(NODE_WIDTH, height));
                let title_rect = Rect::from_min_size(screen_pos, Vec2::new(NODE_WIDTH, TITLE_HEIGHT));

                let painter = ui.painter();
                painter.rect_filled(node_rect, 6.0, Color32::from_rgb(45, 45, 52));
                painter.rect_filled(title_rect, 6.0, Color32::from_rgb(60, 60, 90));
                painter.text(
                    title_rect.center(),
                    Align2::CENTER_CENTER,
                    node.kind.title(),
                    FontId::proportional(14.0),
                    Color32::WHITE,
                );
                painter.rect_stroke(node_rect, 6.0, Stroke::new(1.0, Color32::from_gray(80)));

                let drag_id = ui.id().with(("node_drag", node_id));
                let drag_response = ui.interact(title_rect, drag_id, Sense::click_and_drag());
                if drag_response.dragged() {
                    node.pos += drag_response.drag_delta();
                }
                if drag_response.double_clicked() {
                    nodes_to_disconnect.push(node_id);
                }

                let input_labels = node.kind.input_labels();
                let output_labels = node.kind.output_labels();
                let rows = input_labels.len().max(output_labels.len()).max(1);

                for row in 0..rows {
                    let row_y = screen_pos.y
                        + TITLE_HEIGHT
                        + BODY_PADDING
                        + ROW_HEIGHT * row as f32
                        + ROW_HEIGHT * 0.5;

                    if row < input_labels.len() {
                        let pin_pos = Pos2::new(node_rect.left(), row_y);
                        let pin_ref = PinRef {
                            node_id,
                            kind: PinKind::Input,
                            index: row,
                        };
                        self.pin_positions.insert(pin_ref, pin_pos);

                        let painter = ui.painter();
                        painter.circle_filled(pin_pos, PIN_RADIUS, Color32::from_rgb(220, 180, 90));
                        painter.text(
                            pin_pos + Vec2::new(10.0, 0.0),
                            Align2::LEFT_CENTER,
                            input_labels[row],
                            FontId::proportional(12.0),
                            Color32::from_gray(200),
                        );

                        let pin_rect = Rect::from_center_size(pin_pos, Vec2::splat(PIN_RADIUS * 3.0));
                        let pin_id = ui.id().with(("pin", node_id, "in", row));
                        let pin_response = ui.interact(pin_rect, pin_id, Sense::click_and_drag());

                        if pin_response.hovered() {
                            if let Some(dragging) = &self.dragging_connection {
                                if dragging.from.kind == PinKind::Output
                                    && ctx.input(|i| i.pointer.any_released())
                                {
                                    new_connection = Some(Connection {
                                        from: dragging.from,
                                        to: pin_ref,
                                    });
                                }
                            }
                        }
                    }

                    if row < output_labels.len() {
                        let pin_pos = Pos2::new(node_rect.right(), row_y);
                        let pin_ref = PinRef {
                            node_id,
                            kind: PinKind::Output,
                            index: row,
                        };
                        self.pin_positions.insert(pin_ref, pin_pos);

                        let painter = ui.painter();
                        painter.circle_filled(pin_pos, PIN_RADIUS, Color32::from_rgb(120, 220, 150));
                        painter.text(
                            pin_pos - Vec2::new(10.0, 0.0),
                            Align2::RIGHT_CENTER,
                            output_labels[row],
                            FontId::proportional(12.0),
                            Color32::from_gray(200),
                        );

                        let pin_rect = Rect::from_center_size(pin_pos, Vec2::splat(PIN_RADIUS * 3.0));
                        let pin_id = ui.id().with(("pin", node_id, "out", row));
                        let pin_response = ui.interact(pin_rect, pin_id, Sense::click_and_drag());

                        if pin_response.drag_started() {
                            self.dragging_connection = Some(DraggingConnection {
                                from: pin_ref,
                                current_pos: pin_pos,
                            });
                        }
                    }
                }

                if let NodeKind::Number(value) = &mut node.kind {
                    let row_y = screen_pos.y + TITLE_HEIGHT + BODY_PADDING + ROW_HEIGHT * 0.5;
                    let value_rect = Rect::from_center_size(
                        Pos2::new(node_rect.left() + NODE_WIDTH * 0.42, row_y),
                        Vec2::new(60.0, 18.0),
                    );
                    ui.put(value_rect, egui::DragValue::new(value).speed(0.1));
                }
            }

            if let Some(new_conn) = new_connection {
                self.connections.retain(|c| c.to != new_conn.to);
                self.connections.push(new_conn);
                self.dragging_connection = None;
            }

            for node_id in nodes_to_disconnect {
                self.remove_connections_for(node_id);
            }

            if let Some(dragging) = &mut self.dragging_connection {
                if let Some(hover_pos) = ctx.input(|i| i.pointer.hover_pos()) {
                    dragging.current_pos = hover_pos;
                }

                if let Some(&from_pos) = self.pin_positions.get(&dragging.from) {
                    draw_wire(ui.painter(), from_pos, dragging.current_pos, Color32::from_rgb(255, 210, 120));
                }

                if !ctx.input(|i| i.pointer.primary_down()) {
                    cancel_drag = true;
                }
            }

            if cancel_drag {
                self.dragging_connection = None;
            }
        });
    }
}

impl PartialEq for PinRef {
    fn eq(&self, other: &Self) -> bool {
        self.node_id == other.node_id && self.kind == other.kind && self.index == other.index
    }
}

fn draw_wire(painter: &egui::Painter, from: Pos2, to: Pos2, color: Color32) {
    let control_offset = ((to.x - from.x).abs().max(40.0)) * 0.5;
    let control1 = Pos2::new(from.x + control_offset, from.y);
    let control2 = Pos2::new(to.x - control_offset, to.y);

    let bezier = CubicBezierShape::from_points_stroke(
        [from, control1, control2, to],
        false,
        Color32::TRANSPARENT,
        Stroke::new(2.5, color),
    );

    painter.add(bezier);
}