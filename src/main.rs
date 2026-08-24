use eframe::egui;
use egui::epaint::CubicBezierShape;
use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

type NodeId = u64;
type Counters = (usize, usize, usize, usize, usize, usize);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
enum CompareOp {
    GreaterThan,
    LessThan,
    EqualTo,
}

impl CompareOp {
    fn label(&self) -> &'static str {
        match self {
            CompareOp::GreaterThan => "Greater Than (>)",
            CompareOp::LessThan => "Less Than (<)",
            CompareOp::EqualTo => "Equal To (==)",
        }
    }

    fn symbol(&self) -> &'static str {
        match self {
            CompareOp::GreaterThan => ">",
            CompareOp::LessThan => "<",
            CompareOp::EqualTo => "==",
        }
    }

    fn apply(&self, a: f32, b: f32) -> bool {
        match self {
            CompareOp::GreaterThan => a > b,
            CompareOp::LessThan => a < b,
            CompareOp::EqualTo => a == b,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum NodeKind {
    Number(f32),
    Add,
    Sub,
    Mul,
    Div,
    Print,
    Compare(CompareOp),
    Branch,
    And,
    Or,
    Not,
    Start,
    SetVariable(String),
    GetVariable(String),
    WhileLoop,
    FunctionDef { name: String, params: String },
    FunctionCall { name: String },
    Return,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PinDataType {
    Number,
    Bool,
    Any,
    Exec,
}

impl NodeKind {
    fn title(&self) -> &'static str {
        match self {
            NodeKind::Number(_) => "Number",
            NodeKind::Add => "Math (Add)",
            NodeKind::Sub => "Math (Subtract)",
            NodeKind::Mul => "Math (Multiply)",
            NodeKind::Div => "Math (Divide)",
            NodeKind::Print => "Print",
            NodeKind::Compare(_) => "Compare",
            NodeKind::Branch => "If / Else",
            NodeKind::And => "Logic (AND)",
            NodeKind::Or => "Logic (OR)",
            NodeKind::Not => "Logic (NOT)",
            NodeKind::Start => "Start",
            NodeKind::SetVariable(_) => "Set Variable",
            NodeKind::GetVariable(_) => "Get Variable",
            NodeKind::WhileLoop => "While Loop",
            NodeKind::FunctionDef { .. } => "Function Def",
            NodeKind::FunctionCall { .. } => "Call Function",
            NodeKind::Return => "Return",
        }
    }

    fn input_labels(&self) -> Vec<&'static str> {
        match self {
            NodeKind::Number(_) => vec![],
            NodeKind::Add | NodeKind::Sub | NodeKind::Mul | NodeKind::Div => vec!["A", "B"],
            NodeKind::Print => vec!["In"],
            NodeKind::Compare(_) => vec!["A", "B"],
            NodeKind::Branch => vec!["Cond", "Then", "Else"],
            NodeKind::And | NodeKind::Or => vec!["A", "B"],
            NodeKind::Not => vec!["A"],
            NodeKind::Start => vec![],
            NodeKind::SetVariable(_) => vec!["Value"],
            NodeKind::GetVariable(_) => vec![],
            NodeKind::WhileLoop => vec!["Cond"],
            NodeKind::FunctionDef { .. } => vec![],
            NodeKind::FunctionCall { .. } => vec!["Arg1", "Arg2"],
            NodeKind::Return => vec!["Value"],
        }
    }

    fn output_labels(&self) -> Vec<&'static str> {
        match self {
            NodeKind::Number(_) => vec!["Value"],
            NodeKind::Add | NodeKind::Sub | NodeKind::Mul | NodeKind::Div => vec!["Result"],
            NodeKind::Print => vec![],
            NodeKind::Compare(_) => vec!["Result"],
            NodeKind::Branch => vec!["Result"],
            NodeKind::And | NodeKind::Or | NodeKind::Not => vec!["Result"],
            NodeKind::Start => vec![],
            NodeKind::SetVariable(_) => vec![],
            NodeKind::GetVariable(_) => vec!["Value"],
            NodeKind::WhileLoop => vec![],
            NodeKind::FunctionDef { .. } => vec![],
            NodeKind::FunctionCall { .. } => vec!["Result"],
            NodeKind::Return => vec![],
        }
    }

    fn input_types(&self) -> Vec<PinDataType> {
        match self {
            NodeKind::Number(_) => vec![],
            NodeKind::Add | NodeKind::Sub | NodeKind::Mul | NodeKind::Div => {
                vec![PinDataType::Number, PinDataType::Number]
            }
            NodeKind::Print => vec![PinDataType::Any],
            NodeKind::Compare(_) => vec![PinDataType::Number, PinDataType::Number],
            NodeKind::Branch => vec![PinDataType::Bool, PinDataType::Number, PinDataType::Number],
            NodeKind::And | NodeKind::Or => vec![PinDataType::Bool, PinDataType::Bool],
            NodeKind::Not => vec![PinDataType::Bool],
            NodeKind::Start => vec![],
            NodeKind::SetVariable(_) => vec![PinDataType::Number],
            NodeKind::GetVariable(_) => vec![],
            NodeKind::WhileLoop => vec![PinDataType::Bool],
            NodeKind::FunctionDef { .. } => vec![],
            NodeKind::FunctionCall { .. } => vec![PinDataType::Number, PinDataType::Number],
            NodeKind::Return => vec![PinDataType::Number],
        }
    }

    fn output_types(&self) -> Vec<PinDataType> {
        match self {
            NodeKind::Number(_) => vec![PinDataType::Number],
            NodeKind::Add | NodeKind::Sub | NodeKind::Mul | NodeKind::Div => {
                vec![PinDataType::Number]
            }
            NodeKind::Print => vec![],
            NodeKind::Compare(_) => vec![PinDataType::Bool],
            NodeKind::Branch => vec![PinDataType::Number],
            NodeKind::And | NodeKind::Or | NodeKind::Not => vec![PinDataType::Bool],
            NodeKind::Start => vec![],
            NodeKind::SetVariable(_) => vec![],
            NodeKind::GetVariable(_) => vec![PinDataType::Number],
            NodeKind::WhileLoop => vec![],
            NodeKind::FunctionDef { .. } => vec![],
            NodeKind::FunctionCall { .. } => vec![PinDataType::Number],
            NodeKind::Return => vec![],
        }
    }

    fn exec_input_labels(&self) -> Vec<&'static str> {
        match self {
            NodeKind::SetVariable(_) => vec!["In"],
            NodeKind::Print => vec!["In"],
            NodeKind::WhileLoop => vec!["In"],
            NodeKind::FunctionCall { .. } => vec!["In"],
            NodeKind::Return => vec!["In"],
            _ => vec![],
        }
    }

    fn exec_output_labels(&self) -> Vec<&'static str> {
        match self {
            NodeKind::Start => vec!["Out"],
            NodeKind::SetVariable(_) => vec!["Out"],
            NodeKind::Print => vec!["Out"],
            NodeKind::WhileLoop => vec!["Body", "After"],
            NodeKind::FunctionDef { .. } => vec!["Body"],
            NodeKind::FunctionCall { .. } => vec!["Out"],
            _ => vec![],
        }
    }

    fn widget_extra_height(&self) -> f32 {
        match self {
            NodeKind::Compare(_) => ROW_HEIGHT,
            NodeKind::SetVariable(_) | NodeKind::GetVariable(_) => ROW_HEIGHT,
            NodeKind::FunctionDef { .. } => ROW_HEIGHT * 2.0,
            NodeKind::FunctionCall { .. } => ROW_HEIGHT,
            _ => 0.0,
        }
    }
}

struct Node {
    id: NodeId,
    kind: NodeKind,
    pos: Pos2,
}

#[derive(Debug, Clone, Copy, Eq, Hash, Serialize, Deserialize)]
struct PinRef {
    node_id: NodeId,
    kind: PinKind,
    index: usize,
    #[serde(default)]
    is_exec: bool,
}

impl PartialEq for PinRef {
    fn eq(&self, other: &Self) -> bool {
        self.node_id == other.node_id
            && self.kind == other.kind
            && self.index == other.index
            && self.is_exec == other.is_exec
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum PinKind {
    Input,
    Output,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Connection {
    from: PinRef,
    to: PinRef,
}

struct DraggingConnection {
    from: PinRef,
    current_pos: Pos2,
}

// ---------------------------------------------------------------------
// Camera: pan/zoom state shared by the canvas and the mini-map.
// World space = node.pos coordinates (what gets saved to disk).
// Screen space = actual egui pixels inside the CentralPanel.
// ---------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
struct Camera {
    /// World-space point that appears at the canvas's top-left corner.
    pan: Vec2,
    /// Uniform zoom multiplier. 1.0 = 100%.
    zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self { pan: Vec2::ZERO, zoom: 1.0 }
    }
}

impl Camera {
    const MIN_ZOOM: f32 = 0.15;
    const MAX_ZOOM: f32 = 2.5;

    fn world_to_screen(&self, canvas_origin: Pos2, world: Pos2) -> Pos2 {
        canvas_origin + (world.to_vec2() - self.pan) * self.zoom
    }

    fn screen_to_world(&self, canvas_origin: Pos2, screen: Pos2) -> Pos2 {
        ((screen - canvas_origin) / self.zoom).to_pos2() + self.pan
    }

    /// The world-space rectangle currently visible inside `canvas_rect`.
    fn visible_world_rect(&self, canvas_origin: Pos2, canvas_rect: Rect) -> Rect {
        Rect::from_min_max(
            self.screen_to_world(canvas_origin, canvas_rect.min),
            self.screen_to_world(canvas_origin, canvas_rect.max),
        )
    }

    /// Re-centers the camera on a given world-space point, keeping zoom fixed.
    fn center_on(&mut self, canvas_rect: Rect, world_point: Pos2) {
        let half_extent = canvas_rect.size() / (2.0 * self.zoom);
        self.pan = world_point.to_vec2() - half_extent;
    }

    fn zoom_at(&mut self, canvas_origin: Pos2, screen_anchor: Pos2, factor: f32) {
        let world_anchor_before = self.screen_to_world(canvas_origin, screen_anchor);
        self.zoom = (self.zoom * factor).clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);
        let world_anchor_after = self.screen_to_world(canvas_origin, screen_anchor);
        // Keep the point under the cursor stationary while zooming.
        self.pan += world_anchor_before.to_vec2() - world_anchor_after.to_vec2();
    }
}

// ---------------------------------------------------------------------
// Stage 14: Universal Quick Search (Space / Ctrl+K)
// ---------------------------------------------------------------------

/// One spawnable entry in the quick-search catalog. `factory` builds the
/// NodeKind fresh each time so text fields (names, params) get sane defaults.
struct SearchEntry {
    label: &'static str,
    category: &'static str,
    keywords: &'static str,
    factory: fn() -> NodeKind,
}

fn search_catalog() -> Vec<SearchEntry> {
    vec![
        SearchEntry { label: "Number", category: "Numbers", keywords: "number literal value const", factory: || NodeKind::Number(0.0) },
        SearchEntry { label: "Math (Add)", category: "Numbers", keywords: "math add plus sum +", factory: || NodeKind::Add },
        SearchEntry { label: "Math (Subtract)", category: "Numbers", keywords: "math sub subtract minus -", factory: || NodeKind::Sub },
        SearchEntry { label: "Math (Multiply)", category: "Numbers", keywords: "math mul multiply times *", factory: || NodeKind::Mul },
        SearchEntry { label: "Math (Divide)", category: "Numbers", keywords: "math div divide /", factory: || NodeKind::Div },
        SearchEntry { label: "Compare", category: "Logic", keywords: "compare greater less equal condition", factory: || NodeKind::Compare(CompareOp::GreaterThan) },
        SearchEntry { label: "If / Else", category: "Logic", keywords: "branch if else condition", factory: || NodeKind::Branch },
        SearchEntry { label: "Logic (AND)", category: "Logic", keywords: "and logic boolean", factory: || NodeKind::And },
        SearchEntry { label: "Logic (OR)", category: "Logic", keywords: "or logic boolean", factory: || NodeKind::Or },
        SearchEntry { label: "Logic (NOT)", category: "Logic", keywords: "not logic invert boolean", factory: || NodeKind::Not },
        SearchEntry { label: "Start", category: "Flow", keywords: "start entry begin main", factory: || NodeKind::Start },
        SearchEntry { label: "Set Variable", category: "Flow", keywords: "set variable assign store", factory: || NodeKind::SetVariable("x".to_string()) },
        SearchEntry { label: "Get Variable", category: "Flow", keywords: "get variable read fetch", factory: || NodeKind::GetVariable("x".to_string()) },
        SearchEntry { label: "While Loop", category: "Flow", keywords: "while loop repeat iterate", factory: || NodeKind::WhileLoop },
        SearchEntry { label: "Print", category: "Flow", keywords: "print log output console", factory: || NodeKind::Print },
        SearchEntry { label: "Function Def", category: "Functions", keywords: "function def define declare", factory: || NodeKind::FunctionDef { name: "my_func".to_string(), params: "a, b".to_string() } },
        SearchEntry { label: "Call Function", category: "Functions", keywords: "call function invoke run", factory: || NodeKind::FunctionCall { name: "my_func".to_string() } },
        SearchEntry { label: "Return", category: "Functions", keywords: "return exit result", factory: || NodeKind::Return },
    ]
}

struct QuickSearchState {
    open: bool,
    query: String,
    selected: usize,
    /// World-space point where a spawned node should land (canvas center at
    /// the moment the search was opened, so nodes appear where you're looking).
    spawn_at: Pos2,
}

impl Default for QuickSearchState {
    fn default() -> Self {
        Self {
            open: false,
            query: String::new(),
            selected: 0,
            spawn_at: Pos2::ZERO,
        }
    }
}

impl QuickSearchState {
    fn open_at_cursor(&mut self) {
        self.open = true;
        self.query.clear();
        self.selected = 0;
    }

    fn close(&mut self) {
        self.open = false;
        self.query.clear();
    }
}

#[derive(Debug, Clone, Copy)]
enum Value {
    Number(f32),
    Bool(bool),
}

impl Value {
    fn as_number(&self) -> Option<f32> {
        match self {
            Value::Number(v) => Some(*v),
            Value::Bool(_) => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(v) => Some(*v),
            Value::Number(_) => None,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(v) => write!(f, "{}", v),
            Value::Bool(v) => write!(f, "{}", v),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum IRType {
    Number,
    Bool,
}

#[derive(Debug, Clone)]
enum IROp {
    Literal(f32),
    Add(String, String),
    Sub(String, String),
    Mul(String, String),
    Div(String, String),
    Compare(CompareOp, String, String),
    And(String, String),
    Or(String, String),
    Not(String),
    Branch {
        cond: String,
        then_val: String,
        else_val: String,
    },
}

#[derive(Debug, Clone)]
struct IRStatement {
    var_name: String,
    #[allow(dead_code)]
    ir_type: IRType,
    op: IROp,
}

#[derive(Debug, Clone)]
enum IRStmt {
    Compute(IRStatement),
    SetVar {
        name: String,
        value_var: String,
    },
    Print {
        value_var: String,
    },
    Comment(String),
    While {
        cond_lines: Vec<IRStmt>,
        cond_var: String,
        body: Vec<IRStmt>,
    },
    CallFunction {
        var_name: String,
        func_name: String,
        args: Vec<String>,
    },
    Return(String),
}

#[derive(Debug, Clone)]
struct IRFunction {
    name: String,
    params: Vec<String>,
    body: Vec<IRStmt>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TargetLanguage {
    Python,
    Rust,
    JavaScript,
    Cpp,
}

impl TargetLanguage {
    fn label(&self) -> &'static str {
        match self {
            TargetLanguage::Python => "Python",
            TargetLanguage::Rust => "Rust",
            TargetLanguage::JavaScript => "JavaScript",
            TargetLanguage::Cpp => "C++",
        }
    }

    fn file_extension(&self) -> &'static str {
        match self {
            TargetLanguage::Python => "py",
            TargetLanguage::Rust => "rs",
            TargetLanguage::JavaScript => "js",
            TargetLanguage::Cpp => "cpp",
        }
    }
}

fn format_literal(value: f32) -> String {
    format!("{:?}", value)
}

fn emit_python_stmts(stmts: &[IRStmt], indent: usize, lines: &mut Vec<String>) {
    let pad = "    ".repeat(indent);
    for stmt in stmts {
        match stmt {
            IRStmt::Compute(s) => match &s.op {
                IROp::Literal(v) => {
                    lines.push(format!("{}{} = {}", pad, s.var_name, format_literal(*v)))
                }
                IROp::Add(a, b) => lines.push(format!("{}{} = {} + {}", pad, s.var_name, a, b)),
                IROp::Sub(a, b) => lines.push(format!("{}{} = {} - {}", pad, s.var_name, a, b)),
                IROp::Mul(a, b) => lines.push(format!("{}{} = {} * {}", pad, s.var_name, a, b)),
                IROp::Div(a, b) => lines.push(format!("{}{} = {} / {}", pad, s.var_name, a, b)),
                IROp::Compare(op, a, b) => lines.push(format!(
                    "{}{} = {} {} {}",
                    pad,
                    s.var_name,
                    a,
                    op.symbol(),
                    b
                )),
                IROp::And(a, b) => lines.push(format!("{}{} = {} and {}", pad, s.var_name, a, b)),
                IROp::Or(a, b) => lines.push(format!("{}{} = {} or {}", pad, s.var_name, a, b)),
                IROp::Not(a) => lines.push(format!("{}{} = not {}", pad, s.var_name, a)),
                IROp::Branch {
                    cond,
                    then_val,
                    else_val,
                } => {
                    lines.push(format!("{}if {}:", pad, cond));
                    lines.push(format!("{}    {} = {}", pad, s.var_name, then_val));
                    lines.push(format!("{}else:", pad));
                    lines.push(format!("{}    {} = {}", pad, s.var_name, else_val));
                }
            },
            IRStmt::SetVar { name, value_var } => {
                lines.push(format!("{}{} = {}", pad, name, value_var))
            }
            IRStmt::Print { value_var } => lines.push(format!("{}print({})", pad, value_var)),
            IRStmt::Comment(msg) => lines.push(format!("{}# {}", pad, msg)),
            IRStmt::CallFunction {
                var_name,
                func_name,
                args,
            } => lines.push(format!(
                "{}{} = {}({})",
                pad,
                var_name,
                func_name,
                args.join(", ")
            )),
            IRStmt::Return(v) => lines.push(format!("{}return {}", pad, v)),
            IRStmt::While {
                cond_lines,
                cond_var,
                body,
            } => {
                lines.push(format!("{}while True:", pad));
                emit_python_stmts(cond_lines, indent + 1, lines);
                lines.push(format!("{}    if not ({}):", pad, cond_var));
                lines.push(format!("{}        break", pad));
                emit_python_stmts(body, indent + 1, lines);
            }
        }
    }
}

fn emit_python(functions: &[IRFunction], main_stmts: &[IRStmt]) -> String {
    let mut lines: Vec<String> = Vec::new();

    for func in functions {
        lines.push(format!("def {}({}):", func.name, func.params.join(", ")));
        if func.body.is_empty() {
            lines.push("    pass".to_string());
        } else {
            emit_python_stmts(&func.body, 1, &mut lines);
        }
        lines.push(String::new());
    }

    if main_stmts.is_empty() && functions.is_empty() {
        lines.push("# Your generated code will appear here.".to_string());
        lines.push(
            "# Add Start, Set Variable, Print, and While Loop nodes to see Python code."
                .to_string(),
        );
    } else {
        emit_python_stmts(main_stmts, 0, &mut lines);
    }

    lines.join("\n")
}

fn emit_rust_stmts(
    stmts: &[IRStmt],
    indent: usize,
    lines: &mut Vec<String>,
    declared: &mut HashSet<String>,
) {
    let pad = "    ".repeat(indent);
    for stmt in stmts {
        match stmt {
            IRStmt::Compute(s) => match &s.op {
                IROp::Literal(v) => lines.push(format!(
                    "{}let {} = {};",
                    pad,
                    s.var_name,
                    format_literal(*v)
                )),
                IROp::Add(a, b) => {
                    lines.push(format!("{}let {} = {} + {};", pad, s.var_name, a, b))
                }
                IROp::Sub(a, b) => {
                    lines.push(format!("{}let {} = {} - {};", pad, s.var_name, a, b))
                }
                IROp::Mul(a, b) => {
                    lines.push(format!("{}let {} = {} * {};", pad, s.var_name, a, b))
                }
                IROp::Div(a, b) => {
                    lines.push(format!("{}let {} = {} / {};", pad, s.var_name, a, b))
                }
                IROp::Compare(op, a, b) => lines.push(format!(
                    "{}let {} = {} {} {};",
                    pad,
                    s.var_name,
                    a,
                    op.symbol(),
                    b
                )),
                IROp::And(a, b) => {
                    lines.push(format!("{}let {} = {} && {};", pad, s.var_name, a, b))
                }
                IROp::Or(a, b) => {
                    lines.push(format!("{}let {} = {} || {};", pad, s.var_name, a, b))
                }
                IROp::Not(a) => lines.push(format!("{}let {} = !{};", pad, s.var_name, a)),
                IROp::Branch {
                    cond,
                    then_val,
                    else_val,
                } => {
                    lines.push(format!("{}let {} = if {} {{", pad, s.var_name, cond));
                    lines.push(format!("{}    {}", pad, then_val));
                    lines.push(format!("{}}} else {{", pad));
                    lines.push(format!("{}    {}", pad, else_val));
                    lines.push(format!("{}}};", pad));
                }
            },
            IRStmt::SetVar { name, value_var } => {
                if declared.contains(name) {
                    lines.push(format!("{}{} = {};", pad, name, value_var));
                } else {
                    declared.insert(name.clone());
                    lines.push(format!("{}let mut {} = {};", pad, name, value_var));
                }
            }
            IRStmt::Print { value_var } => {
                lines.push(format!("{}println!(\"{{}}\", {});", pad, value_var))
            }
            IRStmt::Comment(msg) => lines.push(format!("{}// {}", pad, msg)),
            IRStmt::CallFunction {
                var_name,
                func_name,
                args,
            } => lines.push(format!(
                "{}let {} = {}({});",
                pad,
                var_name,
                func_name,
                args.join(", ")
            )),
            IRStmt::Return(v) => lines.push(format!("{}return {};", pad, v)),
            IRStmt::While {
                cond_lines,
                cond_var,
                body,
            } => {
                lines.push(format!("{}loop {{", pad));
                emit_rust_stmts(cond_lines, indent + 1, lines, declared);
                lines.push(format!("{}    if !({}) {{ break; }}", pad, cond_var));
                emit_rust_stmts(body, indent + 1, lines, declared);
                lines.push(format!("{}}}", pad));
            }
        }
    }
}

fn emit_rust(functions: &[IRFunction], main_stmts: &[IRStmt]) -> String {
    let mut out = String::new();

    for func in functions {
        let params_sig: Vec<String> = func.params.iter().map(|p| format!("{}: f64", p)).collect();
        out.push_str(&format!(
            "fn {}({}) -> f64 {{\n",
            func.name,
            params_sig.join(", ")
        ));
        let mut body_lines = Vec::new();
        let mut declared = HashSet::new();
        emit_rust_stmts(&func.body, 1, &mut body_lines, &mut declared);
        for l in &body_lines {
            out.push_str(l);
            out.push('\n');
        }
        out.push_str("    return 0.0;\n");
        out.push_str("}\n\n");
    }

    if main_stmts.is_empty() && functions.is_empty() {
        out.push_str("fn main() {\n    // Your generated code will appear here.\n    // Add Start, Set Variable, Print, and While Loop nodes.\n}");
        return out;
    }

    out.push_str("fn main() {\n");
    let mut body = Vec::new();
    let mut declared = HashSet::new();
    emit_rust_stmts(main_stmts, 1, &mut body, &mut declared);
    for l in &body {
        out.push_str(l);
        out.push('\n');
    }
    out.push('}');
    out
}

fn emit_js_stmts(
    stmts: &[IRStmt],
    indent: usize,
    lines: &mut Vec<String>,
    declared: &mut HashSet<String>,
) {
    let pad = "  ".repeat(indent);
    for stmt in stmts {
        match stmt {
            IRStmt::Compute(s) => match &s.op {
                IROp::Literal(v) => lines.push(format!(
                    "{}let {} = {};",
                    pad,
                    s.var_name,
                    format_literal(*v)
                )),
                IROp::Add(a, b) => {
                    lines.push(format!("{}let {} = {} + {};", pad, s.var_name, a, b))
                }
                IROp::Sub(a, b) => {
                    lines.push(format!("{}let {} = {} - {};", pad, s.var_name, a, b))
                }
                IROp::Mul(a, b) => {
                    lines.push(format!("{}let {} = {} * {};", pad, s.var_name, a, b))
                }
                IROp::Div(a, b) => {
                    lines.push(format!("{}let {} = {} / {};", pad, s.var_name, a, b))
                }
                IROp::Compare(op, a, b) => lines.push(format!(
                    "{}let {} = {} {} {};",
                    pad,
                    s.var_name,
                    a,
                    op.symbol(),
                    b
                )),
                IROp::And(a, b) => {
                    lines.push(format!("{}let {} = {} && {};", pad, s.var_name, a, b))
                }
                IROp::Or(a, b) => {
                    lines.push(format!("{}let {} = {} || {};", pad, s.var_name, a, b))
                }
                IROp::Not(a) => lines.push(format!("{}let {} = !{};", pad, s.var_name, a)),
                IROp::Branch {
                    cond,
                    then_val,
                    else_val,
                } => {
                    lines.push(format!("{}let {};", pad, s.var_name));
                    lines.push(format!("{}if ({}) {{", pad, cond));
                    lines.push(format!("{}  {} = {};", pad, s.var_name, then_val));
                    lines.push(format!("{}}} else {{", pad));
                    lines.push(format!("{}  {} = {};", pad, s.var_name, else_val));
                    lines.push(format!("{}}}", pad));
                }
            },
            IRStmt::SetVar { name, value_var } => {
                if declared.contains(name) {
                    lines.push(format!("{}{} = {};", pad, name, value_var));
                } else {
                    declared.insert(name.clone());
                    lines.push(format!("{}let {} = {};", pad, name, value_var));
                }
            }
            IRStmt::Print { value_var } => {
                lines.push(format!("{}console.log({});", pad, value_var))
            }
            IRStmt::Comment(msg) => lines.push(format!("{}// {}", pad, msg)),
            IRStmt::CallFunction {
                var_name,
                func_name,
                args,
            } => lines.push(format!(
                "{}let {} = {}({});",
                pad,
                var_name,
                func_name,
                args.join(", ")
            )),
            IRStmt::Return(v) => lines.push(format!("{}return {};", pad, v)),
            IRStmt::While {
                cond_lines,
                cond_var,
                body,
            } => {
                lines.push(format!("{}while (true) {{", pad));
                emit_js_stmts(cond_lines, indent + 1, lines, declared);
                lines.push(format!("{}  if (!({})) break;", pad, cond_var));
                emit_js_stmts(body, indent + 1, lines, declared);
                lines.push(format!("{}}}", pad));
            }
        }
    }
}

fn emit_javascript(functions: &[IRFunction], main_stmts: &[IRStmt]) -> String {
    let mut lines: Vec<String> = Vec::new();

    for func in functions {
        lines.push(format!(
            "function {}({}) {{",
            func.name,
            func.params.join(", ")
        ));
        let mut declared = HashSet::new();
        emit_js_stmts(&func.body, 1, &mut lines, &mut declared);
        lines.push("}".to_string());
        lines.push(String::new());
    }

    if main_stmts.is_empty() && functions.is_empty() {
        lines.push("// Your generated code will appear here.".to_string());
        lines.push(
            "// Add Start, Set Variable, Print, and While Loop nodes to see JavaScript code."
                .to_string(),
        );
    } else {
        let mut declared = HashSet::new();
        emit_js_stmts(main_stmts, 0, &mut lines, &mut declared);
    }

    lines.join("\n")
}

fn emit_cpp_stmts(
    stmts: &[IRStmt],
    indent: usize,
    lines: &mut Vec<String>,
    declared: &mut HashSet<String>,
) {
    let pad = "    ".repeat(indent);
    for stmt in stmts {
        match stmt {
            IRStmt::Compute(s) => match &s.op {
                IROp::Literal(v) => lines.push(format!(
                    "{}auto {} = {};",
                    pad,
                    s.var_name,
                    format_literal(*v)
                )),
                IROp::Add(a, b) => {
                    lines.push(format!("{}auto {} = {} + {};", pad, s.var_name, a, b))
                }
                IROp::Sub(a, b) => {
                    lines.push(format!("{}auto {} = {} - {};", pad, s.var_name, a, b))
                }
                IROp::Mul(a, b) => {
                    lines.push(format!("{}auto {} = {} * {};", pad, s.var_name, a, b))
                }
                IROp::Div(a, b) => {
                    lines.push(format!("{}auto {} = {} / {};", pad, s.var_name, a, b))
                }
                IROp::Compare(op, a, b) => lines.push(format!(
                    "{}auto {} = {} {} {};",
                    pad,
                    s.var_name,
                    a,
                    op.symbol(),
                    b
                )),
                IROp::And(a, b) => {
                    lines.push(format!("{}auto {} = {} && {};", pad, s.var_name, a, b))
                }
                IROp::Or(a, b) => {
                    lines.push(format!("{}auto {} = {} || {};", pad, s.var_name, a, b))
                }
                IROp::Not(a) => lines.push(format!("{}auto {} = !{};", pad, s.var_name, a)),
                IROp::Branch {
                    cond,
                    then_val,
                    else_val,
                } => {
                    lines.push(format!(
                        "{}auto {} = ({}) ? ({}) : ({});",
                        pad, s.var_name, cond, then_val, else_val
                    ));
                }
            },
            IRStmt::SetVar { name, value_var } => {
                if declared.contains(name) {
                    lines.push(format!("{}{} = {};", pad, name, value_var));
                } else {
                    declared.insert(name.clone());
                    lines.push(format!("{}auto {} = {};", pad, name, value_var));
                }
            }
            IRStmt::Print { value_var } => {
                lines.push(format!("{}std::cout << {} << std::endl;", pad, value_var))
            }
            IRStmt::Comment(msg) => lines.push(format!("{}// {}", pad, msg)),
            IRStmt::CallFunction {
                var_name,
                func_name,
                args,
            } => lines.push(format!(
                "{}auto {} = {}({});",
                pad,
                var_name,
                func_name,
                args.join(", ")
            )),
            IRStmt::Return(v) => lines.push(format!("{}return {};", pad, v)),
            IRStmt::While {
                cond_lines,
                cond_var,
                body,
            } => {
                lines.push(format!("{}while (true) {{", pad));
                emit_cpp_stmts(cond_lines, indent + 1, lines, declared);
                lines.push(format!("{}    if (!({})) break;", pad, cond_var));
                emit_cpp_stmts(body, indent + 1, lines, declared);
                lines.push(format!("{}}}", pad));
            }
        }
    }
}

fn emit_cpp(functions: &[IRFunction], main_stmts: &[IRStmt]) -> String {
    let mut out = String::new();
    out.push_str("#include <iostream>\n\n");

    for func in functions {
        let params_sig: Vec<String> = func
            .params
            .iter()
            .map(|p| format!("double {}", p))
            .collect();
        out.push_str(&format!(
            "double {}({}) {{\n",
            func.name,
            params_sig.join(", ")
        ));
        let mut body_lines = Vec::new();
        let mut declared = HashSet::new();
        emit_cpp_stmts(&func.body, 1, &mut body_lines, &mut declared);
        for l in &body_lines {
            out.push_str(l);
            out.push('\n');
        }
        out.push_str("    return 0.0;\n");
        out.push_str("}\n\n");
    }

    if main_stmts.is_empty() && functions.is_empty() {
        out.push_str("int main() {\n    // Your generated code will appear here.\n    // Add Start, Set Variable, Print, and While Loop nodes.\n    return 0;\n}");
        return out;
    }

    out.push_str("int main() {\n");
    let mut body = Vec::new();
    let mut declared = HashSet::new();
    emit_cpp_stmts(main_stmts, 1, &mut body, &mut declared);
    for l in &body {
        out.push_str(l);
        out.push('\n');
    }
    out.push_str("    return 0;\n}");
    out
}

const NODE_WIDTH: f32 = 190.0;
const TITLE_HEIGHT: f32 = 28.0;
const ROW_HEIGHT: f32 = 24.0;
const BODY_PADDING: f32 = 8.0;
const PIN_RADIUS: f32 = 6.0;
const PROJECT_FILE: &str = "blocko_project.json";
const MAX_EXEC_STEPS: u32 = 20_000;

mod theme {
    use egui::Color32;

    pub const BG_APP: Color32 = Color32::from_rgb(15, 15, 17);
    pub const BG_PANEL: Color32 = Color32::from_rgb(20, 20, 23);
    pub const BG_NODE: Color32 = Color32::from_rgb(31, 31, 36);
    pub const BG_NODE_HEADER: Color32 = Color32::from_rgb(41, 42, 50);
    pub const BG_INACTIVE_WIDGET: Color32 = Color32::from_rgb(34, 34, 40);
    pub const BG_HOVER_WIDGET: Color32 = Color32::from_rgb(44, 44, 52);
    pub const BG_ACTIVE_WIDGET: Color32 = Color32::from_rgb(50, 50, 60);
    pub const BORDER: Color32 = Color32::from_rgb(58, 58, 66);
    pub const BORDER_SOFT: Color32 = Color32::from_rgb(42, 42, 48);

    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(230, 230, 236);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(140, 140, 150);

    pub const ACCENT_NUMBERS: Color32 = Color32::from_rgb(240, 190, 90);
    pub const ACCENT_LOGIC: Color32 = Color32::from_rgb(190, 130, 240);
    pub const ACCENT_FLOW: Color32 = Color32::from_rgb(225, 225, 232);
    pub const ACCENT_FUNCTIONS: Color32 = Color32::from_rgb(110, 220, 160);
    pub const ACCENT_PROJECT: Color32 = Color32::from_rgb(240, 150, 90);

    pub const PIN_NUMBER: Color32 = Color32::from_rgb(100, 170, 240);
    pub const PIN_BOOL: Color32 = Color32::from_rgb(190, 130, 240);
    pub const PIN_ANY: Color32 = Color32::from_rgb(175, 175, 182);
    pub const PIN_EXEC: Color32 = Color32::from_rgb(235, 235, 240);

    pub const CONSOLE_TIME: Color32 = Color32::from_rgb(120, 120, 130);
    pub const CONSOLE_OK: Color32 = Color32::from_rgb(110, 220, 140);
    pub const CONSOLE_ERR: Color32 = Color32::from_rgb(235, 100, 100);

    pub const CODE_KEYWORD: Color32 = Color32::from_rgb(200, 140, 230);
    pub const CODE_STRING: Color32 = Color32::from_rgb(150, 210, 140);
    pub const CODE_NUMBER: Color32 = Color32::from_rgb(220, 180, 110);
    pub const CODE_COMMENT: Color32 = Color32::from_rgb(110, 110, 120);
    pub const CODE_DEFAULT: Color32 = Color32::from_rgb(215, 215, 222);
    pub const LINE_NUMBER: Color32 = Color32::from_rgb(90, 90, 100);
}

fn is_input_connected(connections: &[Connection], pin: PinRef) -> bool {
    connections.iter().any(|c| c.to == pin)
}

fn is_output_connected(connections: &[Connection], pin: PinRef) -> bool {
    connections.iter().any(|c| c.from == pin)
}

fn draw_pin(painter: &egui::Painter, pos: Pos2, color: Color32, connected: bool) {
    if connected {
        painter.circle_filled(pos, PIN_RADIUS, color);
        painter.circle_stroke(
            pos,
            PIN_RADIUS,
            Stroke::new(1.0, Color32::from_black_alpha(90)),
        );
    } else {
        painter.circle_filled(pos, PIN_RADIUS, theme::BG_NODE);
        painter.circle_stroke(pos, PIN_RADIUS, Stroke::new(1.6, color));
    }
}

fn toolbox_button(ui: &mut egui::Ui, icon_color: Color32, label: &str) -> bool {
    let desired_size = egui::vec2(ui.available_width(), 30.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let bg = if response.hovered() {
            theme::BG_HOVER_WIDGET
        } else {
            Color32::TRANSPARENT
        };
        painter.rect_filled(rect, 6.0, bg);
        let icon_center = Pos2::new(rect.left() + 16.0, rect.center().y);
        painter.circle_filled(icon_center, 4.0, icon_color);
        painter.text(
            Pos2::new(rect.left() + 30.0, rect.center().y),
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(13.5),
            theme::TEXT_PRIMARY,
        );
    }
    response.clicked()
}

fn section_header(ui: &mut egui::Ui, title: &str) {
    ui.add_space(12.0);
    ui.label(
        egui::RichText::new(title.to_uppercase())
            .size(11.0)
            .color(theme::TEXT_MUTED)
            .strong(),
    );
    ui.add_space(4.0);
}

fn code_language_keywords(lang: TargetLanguage) -> &'static [&'static str] {
    match lang {
        TargetLanguage::Python => &[
            "def", "if", "else", "elif", "while", "for", "return", "print", "True", "False",
            "None", "and", "or", "not", "in", "break", "continue", "pass",
        ],
        TargetLanguage::Rust => &[
            "fn", "let", "mut", "if", "else", "while", "loop", "return", "break", "true", "false",
            "struct", "enum", "match", "for", "in",
        ],
        TargetLanguage::JavaScript => &[
            "function", "let", "const", "var", "if", "else", "while", "for", "return", "true",
            "false", "break", "continue", "console",
        ],
        TargetLanguage::Cpp => &[
            "int", "double", "auto", "if", "else", "while", "for", "return", "true", "false",
            "std", "cout", "endl", "include",
        ],
    }
}

fn classify_token(text: &str, keywords: &[&str]) -> Color32 {
    if text
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        theme::CODE_NUMBER
    } else if keywords.contains(&text) {
        theme::CODE_KEYWORD
    } else {
        theme::CODE_DEFAULT
    }
}

fn highlight_code_line(line: &str, keywords: &[&str]) -> egui::text::LayoutJob {
    use egui::text::{LayoutJob, TextFormat};

    let font_id = egui::FontId::monospace(13.0);
    let mut job = LayoutJob::default();

    let trimmed = line.trim_start();
    if trimmed.starts_with("//") || trimmed.starts_with('#') {
        job.append(
            line,
            0.0,
            TextFormat {
                font_id,
                color: theme::CODE_COMMENT,
                ..Default::default()
            },
        );
        return job;
    }

    let mut buf = String::new();
    let mut in_string = false;

    for ch in line.chars() {
        if in_string {
            buf.push(ch);
            if ch == '"' {
                job.append(
                    &buf,
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: theme::CODE_STRING,
                        ..Default::default()
                    },
                );
                buf.clear();
                in_string = false;
            }
            continue;
        }
        if ch == '"' {
            if !buf.is_empty() {
                let color = classify_token(&buf, keywords);
                job.append(
                    &buf,
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color,
                        ..Default::default()
                    },
                );
                buf.clear();
            }
            buf.push(ch);
            in_string = true;
            continue;
        }
        if ch.is_alphanumeric() || ch == '_' || ch == '!' {
            buf.push(ch);
        } else {
            if !buf.is_empty() {
                let color = classify_token(&buf, keywords);
                job.append(
                    &buf,
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color,
                        ..Default::default()
                    },
                );
                buf.clear();
            }
            job.append(
                &ch.to_string(),
                0.0,
                TextFormat {
                    font_id: font_id.clone(),
                    color: theme::CODE_DEFAULT,
                    ..Default::default()
                },
            );
        }
    }
    if !buf.is_empty() {
        let color = classify_token(&buf, keywords);
        job.append(
            &buf,
            0.0,
            TextFormat {
                font_id,
                color,
                ..Default::default()
            },
        );
    }

    job
}

fn code_preview_view(ui: &mut egui::Ui, code: &str, lang: TargetLanguage) {
    let keywords = code_language_keywords(lang);
    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Frame::none()
                .fill(Color32::from_rgb(13, 13, 15))
                .inner_margin(egui::Margin::symmetric(10.0, 10.0))
                .rounding(egui::Rounding::same(6.0))
                .show(ui, |ui| {
                    for (i, line) in code.lines().enumerate() {
                        ui.horizontal(|ui| {
                            ui.add_sized(
                                [26.0, 16.0],
                                egui::Label::new(
                                    egui::RichText::new(format!("{}", i + 1))
                                        .monospace()
                                        .size(12.0)
                                        .color(theme::LINE_NUMBER),
                                ),
                            );
                            let job = highlight_code_line(line, keywords);
                            ui.label(job);
                        });
                    }
                });
        });
}

#[derive(Debug, Serialize, Deserialize)]
struct SerializableNode {
    id: NodeId,
    kind: NodeKind,
    x: f32,
    y: f32,
}

#[derive(Serialize, Deserialize)]
struct ProjectFile {
    version: u32,
    next_id: NodeId,
    nodes: Vec<SerializableNode>,
    connections: Vec<Connection>,
}

// =======================================================================
// Stage 15a: Project Package System
//
// Moves Blocko away from a single monolithic JSON file into a directory:
//
//   MyBlockoProject/
//     project.blocko        <- manifest (JSON): metadata + pointers to graphs
//     nodes/
//       main.graph.json      <- the root graph (nodes + connections)
//       <function>.graph.json <- (future) one file per sub-graph / function
//     assets/
//       (images, audio, data files referenced by nodes — plain std::fs)
//
// This mirrors how most real engines separate "project settings" from
// "content" so that content files are diff-friendly and can be version
// controlled / partially loaded.
// =======================================================================
mod project {
    use super::{Connection, NodeId, SerializableNode};
    use serde::{Deserialize, Serialize};
    use std::path::{Path, PathBuf};

    pub const MANIFEST_FILE: &str = "project.blocko";
    pub const NODES_DIR: &str = "nodes";
    pub const ASSETS_DIR: &str = "assets";
    pub const MAIN_GRAPH_FILE: &str = "main.graph.json";

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ProjectManifest {
        pub format_version: u32,
        pub project_name: String,
        /// Path (relative to the project root) of the graph to load on open.
        pub entry_graph: String,
        pub next_id: NodeId,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct GraphFile {
        pub nodes: Vec<SerializableNode>,
        pub connections: Vec<Connection>,
    }

    #[derive(Debug)]
    pub enum PackageError {
        Io(String),
        Serde(String),
    }

    impl std::fmt::Display for PackageError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                PackageError::Io(e) => write!(f, "IO error: {}", e),
                PackageError::Serde(e) => write!(f, "Serialization error: {}", e),
            }
        }
    }

    impl From<std::io::Error> for PackageError {
        fn from(e: std::io::Error) -> Self {
            PackageError::Io(e.to_string())
        }
    }
    impl From<serde_json::Error> for PackageError {
        fn from(e: serde_json::Error) -> Self {
            PackageError::Serde(e.to_string())
        }
    }

    /// Creates `project_dir/{project.blocko, nodes/, assets/}` and writes the
    /// current graph into `nodes/main.graph.json`. Safe to call repeatedly
    /// (idempotent directory creation via `create_dir_all`).
    pub fn save_package(
        project_dir: &Path,
        project_name: &str,
        next_id: NodeId,
        nodes: Vec<SerializableNode>,
        connections: Vec<Connection>,
    ) -> Result<(), PackageError> {
        let nodes_dir = project_dir.join(NODES_DIR);
        let assets_dir = project_dir.join(ASSETS_DIR);
        std::fs::create_dir_all(&nodes_dir)?;
        std::fs::create_dir_all(&assets_dir)?;

        let graph = GraphFile { nodes, connections };
        let graph_path = nodes_dir.join(MAIN_GRAPH_FILE);
        std::fs::write(&graph_path, serde_json::to_string_pretty(&graph)?)?;

        let manifest = ProjectManifest {
            format_version: 1,
            project_name: project_name.to_string(),
            entry_graph: format!("{}/{}", NODES_DIR, MAIN_GRAPH_FILE),
            next_id,
        };
        let manifest_path = project_dir.join(MANIFEST_FILE);
        std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;

        Ok(())
    }

    /// Reads `project.blocko`, follows `entry_graph`, and returns the parsed
    /// manifest + graph contents.
    pub fn load_package(
        project_dir: &Path,
    ) -> Result<(ProjectManifest, GraphFile), PackageError> {
        let manifest_path = project_dir.join(MANIFEST_FILE);
        let manifest_text = std::fs::read_to_string(&manifest_path)?;
        let manifest: ProjectManifest = serde_json::from_str(&manifest_text)?;

        let graph_path = project_dir.join(&manifest.entry_graph);
        let graph_text = std::fs::read_to_string(&graph_path)?;
        let graph: GraphFile = serde_json::from_str(&graph_text)?;

        Ok((manifest, graph))
    }

    /// Lists asset filenames under `project_dir/assets/` (non-recursive).
    /// Nodes that reference textures/audio/data files can use this to
    /// populate a picker without hardcoding paths.
    pub fn list_assets(project_dir: &Path) -> Result<Vec<PathBuf>, PackageError> {
        let assets_dir = project_dir.join(ASSETS_DIR);
        if !assets_dir.exists() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        for entry in std::fs::read_dir(&assets_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                out.push(entry.path());
            }
        }
        out.sort();
        Ok(out)
    }

    /// Copies an external file into `project_dir/assets/`, returning the
    /// path stored nodes should reference (relative to the project root).
    pub fn import_asset(project_dir: &Path, source: &Path) -> Result<String, PackageError> {
        let assets_dir = project_dir.join(ASSETS_DIR);
        std::fs::create_dir_all(&assets_dir)?;
        let file_name = source
            .file_name()
            .ok_or_else(|| PackageError::Io("source path has no file name".to_string()))?;
        let dest = assets_dir.join(file_name);
        std::fs::copy(source, &dest)?;
        Ok(format!("{}/{}", ASSETS_DIR, file_name.to_string_lossy()))
    }
}

// =======================================================================
// Stage 15b: AI Graph Schema
//
// A deliberately simple, string-keyed serializable representation of a node
// graph that an LLM (or any external tool) can emit from a text prompt,
// without needing to know Blocko's internal enums/PinRef machinery. Blocko
// converts this into real `Node`/`Connection` values via `import_ai_graph`,
// which runs BEFORE type-checking/compiling so malformed graphs are caught
// early instead of surfacing as confusing IR/codegen errors.
// =======================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AiNodeSpec {
    /// Caller-assigned id, unique within this schema. Remapped internally on
    /// import so it never collides with existing project node ids.
    id: u64,
    /// String tag for the node kind: "Number", "Add", "Sub", "Mul", "Div",
    /// "Print", "Compare", "Branch", "And", "Or", "Not", "Start",
    /// "SetVariable", "GetVariable", "WhileLoop", "FunctionDef",
    /// "FunctionCall", "Return".
    kind: String,
    #[serde(default)]
    number_value: Option<f32>,
    #[serde(default)]
    compare_op: Option<String>, // "GreaterThan" | "LessThan" | "EqualTo"
    #[serde(default)]
    variable_name: Option<String>, // SetVariable / GetVariable
    #[serde(default)]
    function_name: Option<String>, // FunctionDef / FunctionCall
    #[serde(default)]
    function_params: Option<String>, // FunctionDef
    #[serde(default)]
    x: f32,
    #[serde(default)]
    y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AiConnectionSpec {
    from_node: u64,
    from_index: usize,
    from_exec: bool,
    to_node: u64,
    to_index: usize,
    to_exec: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AiGraphSchema {
    schema_version: u32,
    #[serde(default)]
    source_prompt: Option<String>,
    nodes: Vec<AiNodeSpec>,
    connections: Vec<AiConnectionSpec>,
}

/// Errors caught while ingesting an AI-generated graph, before it ever
/// reaches IR construction or code emission.
#[derive(Debug)]
enum AiImportError {
    UnknownKind { node_id: u64, kind: String },
    DuplicateNodeId(u64),
    DanglingConnection { node_id: u64 },
}

impl std::fmt::Display for AiImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiImportError::UnknownKind { node_id, kind } => {
                write!(f, "node {} has unknown kind \"{}\"", node_id, kind)
            }
            AiImportError::DuplicateNodeId(id) => {
                write!(f, "duplicate node id {} in AI graph", id)
            }
            AiImportError::DanglingConnection { node_id } => {
                write!(f, "connection references missing node {}", node_id)
            }
        }
    }
}

impl AiNodeSpec {
    fn to_node_kind(&self) -> Result<NodeKind, AiImportError> {
        let kind = match self.kind.as_str() {
            "Number" => NodeKind::Number(self.number_value.unwrap_or(0.0)),
            "Add" => NodeKind::Add,
            "Sub" => NodeKind::Sub,
            "Mul" => NodeKind::Mul,
            "Div" => NodeKind::Div,
            "Print" => NodeKind::Print,
            "Compare" => {
                let op = match self.compare_op.as_deref() {
                    Some("LessThan") => CompareOp::LessThan,
                    Some("EqualTo") => CompareOp::EqualTo,
                    _ => CompareOp::GreaterThan,
                };
                NodeKind::Compare(op)
            }
            "Branch" => NodeKind::Branch,
            "And" => NodeKind::And,
            "Or" => NodeKind::Or,
            "Not" => NodeKind::Not,
            "Start" => NodeKind::Start,
            "SetVariable" => {
                NodeKind::SetVariable(self.variable_name.clone().unwrap_or_else(|| "x".into()))
            }
            "GetVariable" => {
                NodeKind::GetVariable(self.variable_name.clone().unwrap_or_else(|| "x".into()))
            }
            "WhileLoop" => NodeKind::WhileLoop,
            "FunctionDef" => NodeKind::FunctionDef {
                name: self.function_name.clone().unwrap_or_else(|| "func".into()),
                params: self.function_params.clone().unwrap_or_default(),
            },
            "FunctionCall" => NodeKind::FunctionCall {
                name: self.function_name.clone().unwrap_or_else(|| "func".into()),
            },
            "Return" => NodeKind::Return,
            other => {
                return Err(AiImportError::UnknownKind {
                    node_id: self.id,
                    kind: other.to_string(),
                })
            }
        };
        Ok(kind)
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1620.0, 920.0])
            .with_min_inner_size([980.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Blocko",
        native_options,
        Box::new(|cc| {
            let mut visuals = egui::Visuals::dark();
            visuals.window_fill = theme::BG_APP;
            visuals.panel_fill = theme::BG_PANEL;
            visuals.widgets.noninteractive.bg_fill = theme::BG_PANEL;
            visuals.widgets.inactive.bg_fill = theme::BG_INACTIVE_WIDGET;
            visuals.widgets.hovered.bg_fill = theme::BG_HOVER_WIDGET;
            visuals.widgets.active.bg_fill = theme::BG_ACTIVE_WIDGET;
            visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, theme::BORDER_SOFT);
            visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, theme::BORDER);
            visuals.selection.bg_fill = Color32::from_rgb(70, 100, 190);
            visuals.window_rounding = egui::Rounding::same(8.0);
            visuals.widgets.noninteractive.rounding = egui::Rounding::same(6.0);
            visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);
            visuals.widgets.hovered.rounding = egui::Rounding::same(6.0);
            visuals.widgets.active.rounding = egui::Rounding::same(6.0);
            cc.egui_ctx.set_visuals(visuals);
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
    console_lines: Vec<String>,
    current_language: TargetLanguage,

    // --- Mini-map / camera ---
    camera: Camera,
    minimap_dragging: bool,

    // --- Stage 14: Universal Quick Search ---
    quick_search: QuickSearchState,

    // --- Stage 15: Project package system ---
    project_dir: std::path::PathBuf,
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
            console_lines: Vec::new(),
            current_language: TargetLanguage::Python,
            camera: Camera::default(),
            minimap_dragging: false,
            quick_search: QuickSearchState::default(),
            project_dir: std::path::PathBuf::from("./MyBlockoProject"),
        }
    }

    fn add_node(&mut self, kind: NodeKind) {
        let id = self.next_id;
        self.next_id += 1;

        let count = self.nodes.len() as f32;
        let pos = Pos2::new(60.0 + (count * 40.0) % 400.0, 60.0 + (count * 30.0) % 300.0);

        self.status_message = format!("Added node: {}", kind.title());
        self.nodes.insert(id, Node { id, kind, pos });
    }

    /// Spawns a node at an explicit world-space position (used by quick search).
    fn add_node_at(&mut self, kind: NodeKind, pos: Pos2) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        self.status_message = format!("Added node: {}", kind.title());
        self.nodes.insert(id, Node { id, kind, pos });
        id
    }

    /// Stage 14: Auto-Layout. Arranges nodes into left-to-right layers based on
    /// their dependency depth (how many hops from a "root" node with no
    /// incoming connections), then orders nodes within each layer using a
    /// barycenter heuristic (average position of parents in the previous
    /// layer) to reduce wire crossings. This is a simplified Sugiyama-style
    /// layered layout — good enough for blueprint-style graphs without
    /// pulling in an external graph-layout crate.
    fn auto_arrange(&mut self) {
        let node_ids: Vec<NodeId> = self.nodes.keys().copied().collect();
        if node_ids.is_empty() {
            self.status_message = "Nothing to arrange.".to_string();
            return;
        }

        let mut children: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        let mut parents: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        for &id in &node_ids {
            children.insert(id, Vec::new());
            parents.insert(id, Vec::new());
        }
        for conn in &self.connections {
            let (a, b) = (conn.from.node_id, conn.to.node_id);
            if a == b {
                continue;
            }
            if let Some(v) = children.get_mut(&a) {
                if !v.contains(&b) {
                    v.push(b);
                }
            }
            if let Some(v) = parents.get_mut(&b) {
                if !v.contains(&a) {
                    v.push(a);
                }
            }
        }

        fn compute_depth(
            id: NodeId,
            parents: &HashMap<NodeId, Vec<NodeId>>,
            depth: &mut HashMap<NodeId, i32>,
            visiting: &mut HashSet<NodeId>,
        ) -> i32 {
            if let Some(&d) = depth.get(&id) {
                return d;
            }
            if !visiting.insert(id) {
                // Back-edge (e.g. a While Loop feeding into an earlier node):
                // stop recursing here instead of looping forever.
                return 0;
            }
            let mut best = 0;
            for &p in parents.get(&id).map(|v| v.as_slice()).unwrap_or(&[]) {
                best = best.max(compute_depth(p, parents, depth, visiting) + 1);
            }
            visiting.remove(&id);
            depth.insert(id, best);
            best
        }

        let mut depth: HashMap<NodeId, i32> = HashMap::new();
        for &id in &node_ids {
            let mut visiting = HashSet::new();
            compute_depth(id, &parents, &mut depth, &mut visiting);
        }

        let max_depth = depth.values().copied().max().unwrap_or(0);
        let mut layers: Vec<Vec<NodeId>> = vec![Vec::new(); (max_depth + 1) as usize];
        for &id in &node_ids {
            let d = depth.get(&id).copied().unwrap_or(0).max(0) as usize;
            layers[d].push(id);
        }
        for layer in &mut layers {
            layer.sort_unstable();
        }

        // Barycenter crossing-reduction passes.
        let mut order_index: HashMap<NodeId, f32> = HashMap::new();
        for layer in &layers {
            for (i, &id) in layer.iter().enumerate() {
                order_index.insert(id, i as f32);
            }
        }
        for _pass in 0..4 {
            for li in 1..layers.len() {
                let prev_index: HashMap<NodeId, f32> = layers[li - 1]
                    .iter()
                    .enumerate()
                    .map(|(i, &id)| (id, i as f32))
                    .collect();
                let mut scored: Vec<(NodeId, f32)> = layers[li]
                    .iter()
                    .map(|&id| {
                        let ps = parents.get(&id).map(|v| v.as_slice()).unwrap_or(&[]);
                        let linked: Vec<f32> =
                            ps.iter().filter_map(|p| prev_index.get(p)).copied().collect();
                        let score = if linked.is_empty() {
                            order_index.get(&id).copied().unwrap_or(0.0)
                        } else {
                            linked.iter().sum::<f32>() / linked.len() as f32
                        };
                        (id, score)
                    })
                    .collect();
                scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                layers[li] = scored.iter().map(|(id, _)| *id).collect();
                for (i, &id) in layers[li].iter().enumerate() {
                    order_index.insert(id, i as f32);
                }
            }
        }

        const COL_GAP: f32 = 260.0;
        const ROW_GAP: f32 = 40.0;

        let mut cursor_x = 0.0;
        for layer in &layers {
            let mut cursor_y = 0.0;
            for &id in layer {
                let h = BlockoApp::node_height(&self.nodes[&id].kind);
                if let Some(node) = self.nodes.get_mut(&id) {
                    node.pos = Pos2::new(cursor_x, cursor_y);
                }
                cursor_y += h + ROW_GAP;
            }
            cursor_x += NODE_WIDTH + COL_GAP;
        }

        self.pin_positions.clear();
        self.status_message = format!(
            "Arranged {} nodes across {} layer(s).",
            node_ids.len(),
            layers.len()
        );
    }

    /// Renders the Space/Ctrl+K quick-search modal and handles keyboard
    /// navigation + spawning the selected node.
    fn show_quick_search_overlay(&mut self, ctx: &egui::Context) {
        if !self.quick_search.open {
            return;
        }

        let catalog = search_catalog();
        let query_lower = self.quick_search.query.to_lowercase();
        let mut matches: Vec<usize> = (0..catalog.len())
            .filter(|&i| {
                if query_lower.is_empty() {
                    return true;
                }
                let entry = &catalog[i];
                entry.label.to_lowercase().contains(&query_lower)
                    || entry.category.to_lowercase().contains(&query_lower)
                    || entry.keywords.contains(&query_lower)
            })
            .collect();
        // Stable, readable ordering: category then label.
        matches.sort_by_key(|&i| (catalog[i].category, catalog[i].label));

        if self.quick_search.selected >= matches.len() {
            self.quick_search.selected = matches.len().saturating_sub(1);
        }

        let mut close_after = false;
        let mut spawn_choice: Option<usize> = None;

        egui::Window::new("quick_search")
            .title_bar(false)
            .anchor(Align2::CENTER_TOP, Vec2::new(0.0, 90.0))
            .fixed_size(Vec2::new(420.0, 360.0))
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🔍").size(16.0));
                    let text_response = ui.add(
                        egui::TextEdit::singleline(&mut self.quick_search.query)
                            .hint_text("Search nodes… (e.g. \"math\", \"pri\")")
                            .desired_width(360.0),
                    );
                    text_response.request_focus();
                });
                ui.add_space(4.0);
                ui.separator();

                egui::ScrollArea::vertical()
                    .max_height(280.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if matches.is_empty() {
                            ui.label(
                                egui::RichText::new("No matching nodes.")
                                    .color(theme::TEXT_MUTED),
                            );
                        }
                        for (row, &catalog_idx) in matches.iter().enumerate() {
                            let entry = &catalog[catalog_idx];
                            let selected = row == self.quick_search.selected;
                            let text = egui::RichText::new(format!(
                                "{}   ·   {}",
                                entry.label, entry.category
                            ))
                            .color(if selected {
                                theme::TEXT_PRIMARY
                            } else {
                                theme::TEXT_MUTED
                            });
                            let resp = ui.selectable_label(selected, text);
                            if resp.clicked() {
                                spawn_choice = Some(catalog_idx);
                            }
                            if resp.hovered() {
                                self.quick_search.selected = row;
                            }
                        }
                    });

                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("↑↓ navigate · Enter spawn · Esc close")
                        .size(10.5)
                        .color(theme::TEXT_MUTED),
                );
            });

        ctx.input(|i| {
            if i.key_pressed(egui::Key::Escape) {
                close_after = true;
            }
            if i.key_pressed(egui::Key::ArrowDown) && !matches.is_empty() {
                self.quick_search.selected = (self.quick_search.selected + 1) % matches.len();
            }
            if i.key_pressed(egui::Key::ArrowUp) && !matches.is_empty() {
                self.quick_search.selected =
                    (self.quick_search.selected + matches.len() - 1) % matches.len();
            }
            if i.key_pressed(egui::Key::Enter) && !matches.is_empty() {
                spawn_choice = Some(matches[self.quick_search.selected]);
            }
        });

        if let Some(catalog_idx) = spawn_choice {
            let kind = (catalog[catalog_idx].factory)();
            let spawn_at = self.quick_search.spawn_at;
            self.add_node_at(kind, spawn_at);
            close_after = true;
        }

        if close_after {
            self.quick_search.close();
        }
    }

    fn node_height(kind: &NodeKind) -> f32 {
        let data_rows = kind.input_labels().len().max(kind.output_labels().len());
        let exec_rows = kind
            .exec_input_labels()
            .len()
            .max(kind.exec_output_labels().len());
        let total_rows = (data_rows + exec_rows).max(1);
        TITLE_HEIGHT
            + kind.widget_extra_height()
            + total_rows as f32 * ROW_HEIGHT
            + BODY_PADDING * 2.0
    }

    fn remove_connections_for(&mut self, node_id: NodeId) {
        self.connections
            .retain(|c| c.from.node_id != node_id && c.to.node_id != node_id);
    }

    fn find_source_for_input(&self, node_id: NodeId, input_index: usize) -> Option<PinRef> {
        let target = PinRef {
            node_id,
            kind: PinKind::Input,
            index: input_index,
            is_exec: false,
        };
        self.connections
            .iter()
            .find(|c| c.to == target)
            .map(|c| c.from)
    }

    fn find_exec_target(&self, node_id: NodeId, output_index: usize) -> Option<NodeId> {
        let target = PinRef {
            node_id,
            kind: PinKind::Output,
            index: output_index,
            is_exec: true,
        };
        self.connections
            .iter()
            .find(|c| c.from == target)
            .map(|c| c.to.node_id)
    }

    fn find_start_node(&self) -> Option<NodeId> {
        let mut ids: Vec<NodeId> = self
            .nodes
            .iter()
            .filter(|(_, n)| matches!(n.kind, NodeKind::Start))
            .map(|(id, _)| *id)
            .collect();
        ids.sort_unstable();
        ids.first().copied()
    }

    fn evaluate_output(
        &self,
        node_id: NodeId,
        visiting: &mut Vec<NodeId>,
        variables: &HashMap<String, f32>,
        call_cache: &HashMap<NodeId, f32>,
    ) -> Option<Value> {
        if visiting.contains(&node_id) {
            return None;
        }
        visiting.push(node_id);

        let node = self.nodes.get(&node_id)?;
        let result = match &node.kind {
            NodeKind::Number(v) => Some(Value::Number(*v)),
            NodeKind::GetVariable(name) => variables.get(name).copied().map(Value::Number),
            NodeKind::FunctionCall { .. } => call_cache.get(&node_id).copied().map(Value::Number),
            NodeKind::Add | NodeKind::Sub | NodeKind::Mul | NodeKind::Div => {
                let a_source = self.find_source_for_input(node_id, 0)?;
                let b_source = self.find_source_for_input(node_id, 1)?;
                let a_val = self
                    .evaluate_output(a_source.node_id, visiting, variables, call_cache)?
                    .as_number()?;
                let b_val = self
                    .evaluate_output(b_source.node_id, visiting, variables, call_cache)?
                    .as_number()?;
                let result = match &node.kind {
                    NodeKind::Add => a_val + b_val,
                    NodeKind::Sub => a_val - b_val,
                    NodeKind::Mul => a_val * b_val,
                    NodeKind::Div => {
                        if b_val == 0.0 {
                            0.0
                        } else {
                            a_val / b_val
                        }
                    }
                    _ => unreachable!(),
                };
                Some(Value::Number(result))
            }
            NodeKind::Compare(op) => {
                let a_source = self.find_source_for_input(node_id, 0)?;
                let b_source = self.find_source_for_input(node_id, 1)?;
                let a_val = self
                    .evaluate_output(a_source.node_id, visiting, variables, call_cache)?
                    .as_number()?;
                let b_val = self
                    .evaluate_output(b_source.node_id, visiting, variables, call_cache)?
                    .as_number()?;
                Some(Value::Bool(op.apply(a_val, b_val)))
            }
            NodeKind::And => {
                let a_source = self.find_source_for_input(node_id, 0)?;
                let a_val = self
                    .evaluate_output(a_source.node_id, visiting, variables, call_cache)?
                    .as_bool()?;
                if !a_val {
                    // short-circuit: A is false, B is not evaluated
                    Some(Value::Bool(false))
                } else {
                    let b_source = self.find_source_for_input(node_id, 1)?;
                    let b_val = self
                        .evaluate_output(b_source.node_id, visiting, variables, call_cache)?
                        .as_bool()?;
                    Some(Value::Bool(b_val))
                }
            }
            NodeKind::Or => {
                let a_source = self.find_source_for_input(node_id, 0)?;
                let a_val = self
                    .evaluate_output(a_source.node_id, visiting, variables, call_cache)?
                    .as_bool()?;
                if a_val {
                    // short-circuit: A is true, B is not evaluated
                    Some(Value::Bool(true))
                } else {
                    let b_source = self.find_source_for_input(node_id, 1)?;
                    let b_val = self
                        .evaluate_output(b_source.node_id, visiting, variables, call_cache)?
                        .as_bool()?;
                    Some(Value::Bool(b_val))
                }
            }
            NodeKind::Not => {
                let a_source = self.find_source_for_input(node_id, 0)?;
                let a_val = self
                    .evaluate_output(a_source.node_id, visiting, variables, call_cache)?
                    .as_bool()?;
                Some(Value::Bool(!a_val))
            }
            NodeKind::Branch => {
                let cond_source = self.find_source_for_input(node_id, 0)?;
                let then_source = self.find_source_for_input(node_id, 1)?;
                let else_source = self.find_source_for_input(node_id, 2)?;
                let cond_val = self
                    .evaluate_output(cond_source.node_id, visiting, variables, call_cache)?
                    .as_bool()?;
                if cond_val {
                    let then_val = self
                        .evaluate_output(then_source.node_id, visiting, variables, call_cache)?
                        .as_number()?;
                    Some(Value::Number(then_val))
                } else {
                    let else_val = self
                        .evaluate_output(else_source.node_id, visiting, variables, call_cache)?
                        .as_number()?;
                    Some(Value::Number(else_val))
                }
            }
            _ => None,
        };

        visiting.pop();
        result
    }

    #[allow(clippy::too_many_arguments)]
    fn exec_statement(
        &self,
        node_id: NodeId,
        variables: &mut HashMap<String, f32>,
        console: &mut Vec<String>,
        steps: &mut u32,
        functions: &HashMap<String, (Vec<String>, Option<NodeId>)>,
        call_cache: &mut HashMap<NodeId, f32>,
        return_slot: &mut Option<f32>,
    ) {
        let mut current = Some(node_id);
        while let Some(id) = current {
            if return_slot.is_some() {
                return;
            }
            if *steps >= MAX_EXEC_STEPS {
                console.push(
                    "... execution stopped: step limit reached (possible infinite loop) ..."
                        .to_string(),
                );
                return;
            }
            *steps += 1;

            let node = match self.nodes.get(&id) {
                Some(n) => n,
                None => return,
            };

            match &node.kind {
                NodeKind::SetVariable(name) => {
                    let mut visiting = Vec::new();
                    let value = self
                        .find_source_for_input(id, 0)
                        .and_then(|src| {
                            self.evaluate_output(src.node_id, &mut visiting, variables, call_cache)
                        })
                        .and_then(|v| v.as_number())
                        .unwrap_or(0.0);
                    variables.insert(name.clone(), value);
                    current = self.find_exec_target(id, 0);
                }
                NodeKind::Print => {
                    let mut visiting = Vec::new();
                    match self.find_source_for_input(id, 0) {
                        Some(src) => match self.evaluate_output(
                            src.node_id,
                            &mut visiting,
                            variables,
                            call_cache,
                        ) {
                            Some(value) => console.push(format!("Print[{}] -> {}", id, value)),
                            None => {
                                console.push(format!("Print[{}] -> <could not evaluate input>", id))
                            }
                        },
                        None => console.push(format!("Print[{}] -> <no input connected>", id)),
                    }
                    current = self.find_exec_target(id, 0);
                }
                NodeKind::WhileLoop => {
                    loop {
                        if *steps >= MAX_EXEC_STEPS {
                            console.push("... execution stopped: step limit reached (possible infinite loop) ...".to_string());
                            break;
                        }
                        let mut visiting = Vec::new();
                        let cond = self
                            .find_source_for_input(id, 0)
                            .and_then(|src| {
                                self.evaluate_output(
                                    src.node_id,
                                    &mut visiting,
                                    variables,
                                    call_cache,
                                )
                            })
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        if !cond {
                            break;
                        }
                        if let Some(body_start) = self.find_exec_target(id, 0) {
                            self.exec_statement(
                                body_start,
                                variables,
                                console,
                                steps,
                                functions,
                                call_cache,
                                return_slot,
                            );
                            if return_slot.is_some() {
                                return;
                            }
                        }
                        *steps += 1;
                    }
                    current = self.find_exec_target(id, 1);
                }
                NodeKind::FunctionCall { name } => {
                    let mut visiting_a = Vec::new();
                    let a0 = self
                        .find_source_for_input(id, 0)
                        .and_then(|src| {
                            self.evaluate_output(
                                src.node_id,
                                &mut visiting_a,
                                variables,
                                call_cache,
                            )
                        })
                        .and_then(|v| v.as_number())
                        .unwrap_or(0.0);
                    let mut visiting_b = Vec::new();
                    let a1 = self
                        .find_source_for_input(id, 1)
                        .and_then(|src| {
                            self.evaluate_output(
                                src.node_id,
                                &mut visiting_b,
                                variables,
                                call_cache,
                            )
                        })
                        .and_then(|v| v.as_number())
                        .unwrap_or(0.0);

                    if let Some((params, body_start_opt)) = functions.get(name) {
                        if let Some(p) = params.get(0) {
                            variables.insert(p.clone(), a0);
                        }
                        if let Some(p) = params.get(1) {
                            variables.insert(p.clone(), a1);
                        }
                        let mut inner_return: Option<f32> = None;
                        if let Some(body_start) = body_start_opt {
                            self.exec_statement(
                                *body_start,
                                variables,
                                console,
                                steps,
                                functions,
                                call_cache,
                                &mut inner_return,
                            );
                        }
                        call_cache.insert(id, inner_return.unwrap_or(0.0));
                    } else {
                        console.push(format!("Call to undefined function '{}'", name));
                        call_cache.insert(id, 0.0);
                    }
                    current = self.find_exec_target(id, 0);
                }
                NodeKind::Return => {
                    let mut visiting = Vec::new();
                    let value = self
                        .find_source_for_input(id, 0)
                        .and_then(|src| {
                            self.evaluate_output(src.node_id, &mut visiting, variables, call_cache)
                        })
                        .and_then(|v| v.as_number())
                        .unwrap_or(0.0);
                    *return_slot = Some(value);
                    return;
                }
                _ => {
                    current = None;
                }
            }
        }
    }

    fn run_legacy_prints(&self, console: &mut Vec<String>) {
        let mut print_ids: Vec<NodeId> = self
            .nodes
            .iter()
            .filter(|(_, n)| matches!(n.kind, NodeKind::Print))
            .map(|(id, _)| *id)
            .collect();
        print_ids.sort_unstable();

        if print_ids.is_empty() {
            console.push("No Print nodes on canvas.".to_string());
            return;
        }

        let variables: HashMap<String, f32> = HashMap::new();
        let call_cache: HashMap<NodeId, f32> = HashMap::new();
        for print_id in print_ids {
            let mut visiting = Vec::new();
            match self.find_source_for_input(print_id, 0) {
                Some(source) => match self.evaluate_output(
                    source.node_id,
                    &mut visiting,
                    &variables,
                    &call_cache,
                ) {
                    Some(value) => console.push(format!("Print[{}] -> {}", print_id, value)),
                    None => {
                        console.push(format!("Print[{}] -> <could not evaluate input>", print_id))
                    }
                },
                None => console.push(format!("Print[{}] -> <no input connected>", print_id)),
            }
        }
    }

    fn collect_functions(&self) -> HashMap<String, (Vec<String>, Option<NodeId>)> {
        let mut functions: HashMap<String, (Vec<String>, Option<NodeId>)> = HashMap::new();
        for (id, n) in &self.nodes {
            if let NodeKind::FunctionDef { name, params } = &n.kind {
                let plist: Vec<String> = params
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                let body_start = self.find_exec_target(*id, 0);
                functions.insert(name.clone(), (plist, body_start));
            }
        }
        functions
    }

    fn run_program(&mut self) {
        let mut console: Vec<String> = Vec::new();
        console.push("--- Program started ---".to_string());

        let functions = self.collect_functions();

        if let Some(start_id) = self.find_start_node() {
            let mut variables: HashMap<String, f32> = HashMap::new();
            let mut steps: u32 = 0;
            let mut call_cache: HashMap<NodeId, f32> = HashMap::new();
            let mut return_slot: Option<f32> = None;
            if let Some(first) = self.find_exec_target(start_id, 0) {
                self.exec_statement(
                    first,
                    &mut variables,
                    &mut console,
                    &mut steps,
                    &functions,
                    &mut call_cache,
                    &mut return_slot,
                );
            } else {
                console.push("Start node has no connected statements.".to_string());
            }
        } else {
            self.run_legacy_prints(&mut console);
        }

        console.push("--- Program finished ---".to_string());
        self.console_lines = console;
        self.status_message = "Program executed.".to_string();
    }

    fn build_expr_ir(
        &self,
        node_id: NodeId,
        var_names: &mut HashMap<NodeId, (String, IRType)>,
        out: &mut Vec<IRStmt>,
        counters: &mut Counters,
        visiting: &mut Vec<NodeId>,
    ) -> Option<(String, IRType)> {
        if visiting.contains(&node_id) {
            return None;
        }
        if let Some(existing) = var_names.get(&node_id) {
            return Some(existing.clone());
        }

        visiting.push(node_id);
        let node = self.nodes.get(&node_id)?;

        let result = match &node.kind {
            NodeKind::Number(v) => {
                let name = format!("num_{}", counters.0);
                counters.0 += 1;
                out.push(IRStmt::Compute(IRStatement {
                    var_name: name.clone(),
                    ir_type: IRType::Number,
                    op: IROp::Literal(*v),
                }));
                var_names.insert(node_id, (name.clone(), IRType::Number));
                Some((name, IRType::Number))
            }
            NodeKind::GetVariable(var_name) => {
                var_names.insert(node_id, (var_name.clone(), IRType::Number));
                Some((var_name.clone(), IRType::Number))
            }
            NodeKind::Add | NodeKind::Sub | NodeKind::Mul | NodeKind::Div => {
                let a_source = self.find_source_for_input(node_id, 0)?;
                let b_source = self.find_source_for_input(node_id, 1)?;
                let (a_name, _) =
                    self.build_expr_ir(a_source.node_id, var_names, out, counters, visiting)?;
                let (b_name, _) =
                    self.build_expr_ir(b_source.node_id, var_names, out, counters, visiting)?;

                let (prefix, op, count_ref) = match node.kind {
                    NodeKind::Add => ("add", IROp::Add(a_name, b_name), &mut counters.1),
                    NodeKind::Sub => ("sub", IROp::Sub(a_name, b_name), &mut counters.2),
                    NodeKind::Mul => ("mul", IROp::Mul(a_name, b_name), &mut counters.3),
                    NodeKind::Div => ("div", IROp::Div(a_name, b_name), &mut counters.4),
                    _ => unreachable!(),
                };

                let name = format!("{}_{}", prefix, *count_ref);
                *count_ref += 1;

                out.push(IRStmt::Compute(IRStatement {
                    var_name: name.clone(),
                    ir_type: IRType::Number,
                    op,
                }));
                var_names.insert(node_id, (name.clone(), IRType::Number));
                Some((name, IRType::Number))
            }
            NodeKind::Compare(op) => {
                let a_source = self.find_source_for_input(node_id, 0)?;
                let b_source = self.find_source_for_input(node_id, 1)?;
                let (a_name, _) =
                    self.build_expr_ir(a_source.node_id, var_names, out, counters, visiting)?;
                let (b_name, _) =
                    self.build_expr_ir(b_source.node_id, var_names, out, counters, visiting)?;
                let name = format!("cmp_{}", counters.2);
                counters.2 += 1;
                out.push(IRStmt::Compute(IRStatement {
                    var_name: name.clone(),
                    ir_type: IRType::Bool,
                    op: IROp::Compare(*op, a_name, b_name),
                }));
                var_names.insert(node_id, (name.clone(), IRType::Bool));
                Some((name, IRType::Bool))
            }
            NodeKind::And | NodeKind::Or => {
                let a_source = self.find_source_for_input(node_id, 0)?;
                let b_source = self.find_source_for_input(node_id, 1)?;
                let (a_name, _) =
                    self.build_expr_ir(a_source.node_id, var_names, out, counters, visiting)?;
                let (b_name, _) =
                    self.build_expr_ir(b_source.node_id, var_names, out, counters, visiting)?;

                let (prefix, op) = match node.kind {
                    NodeKind::And => ("and", IROp::And(a_name, b_name)),
                    NodeKind::Or => ("or", IROp::Or(a_name, b_name)),
                    _ => unreachable!(),
                };

                let name = format!("{}_{}", prefix, counters.5);
                counters.5 += 1;
                out.push(IRStmt::Compute(IRStatement {
                    var_name: name.clone(),
                    ir_type: IRType::Bool,
                    op,
                }));
                var_names.insert(node_id, (name.clone(), IRType::Bool));
                Some((name, IRType::Bool))
            }
            NodeKind::Not => {
                let a_source = self.find_source_for_input(node_id, 0)?;
                let (a_name, _) =
                    self.build_expr_ir(a_source.node_id, var_names, out, counters, visiting)?;
                let name = format!("not_{}", counters.5);
                counters.5 += 1;
                out.push(IRStmt::Compute(IRStatement {
                    var_name: name.clone(),
                    ir_type: IRType::Bool,
                    op: IROp::Not(a_name),
                }));
                var_names.insert(node_id, (name.clone(), IRType::Bool));
                Some((name, IRType::Bool))
            }
            NodeKind::Branch => {
                let cond_source = self.find_source_for_input(node_id, 0)?;
                let then_source = self.find_source_for_input(node_id, 1)?;
                let else_source = self.find_source_for_input(node_id, 2)?;
                let (cond_name, _) =
                    self.build_expr_ir(cond_source.node_id, var_names, out, counters, visiting)?;
                let (then_name, _) =
                    self.build_expr_ir(then_source.node_id, var_names, out, counters, visiting)?;
                let (else_name, _) =
                    self.build_expr_ir(else_source.node_id, var_names, out, counters, visiting)?;
                let name = format!("branch_{}", counters.3);
                counters.3 += 1;
                out.push(IRStmt::Compute(IRStatement {
                    var_name: name.clone(),
                    ir_type: IRType::Number,
                    op: IROp::Branch {
                        cond: cond_name,
                        then_val: then_name,
                        else_val: else_name,
                    },
                }));
                var_names.insert(node_id, (name.clone(), IRType::Number));
                Some((name, IRType::Number))
            }
            _ => None,
        };

        visiting.pop();
        result
    }

    fn build_stmt_chain(&self, start: Option<NodeId>, counters: &mut Counters) -> Vec<IRStmt> {
        let mut out = Vec::new();
        let mut var_names: HashMap<NodeId, (String, IRType)> = HashMap::new();
        let mut current = start;
        let mut safety = 0u32;

        while let Some(id) = current {
            safety += 1;
            if safety > 5000 {
                out.push(IRStmt::Comment(
                    "execution chain too long or cyclic, stopped".to_string(),
                ));
                break;
            }
            let node = match self.nodes.get(&id) {
                Some(n) => n,
                None => break,
            };

            match &node.kind {
                NodeKind::SetVariable(name) => {
                    let mut visiting = Vec::new();
                    let value_var = match self.find_source_for_input(id, 0) {
                        Some(src) => self
                            .build_expr_ir(
                                src.node_id,
                                &mut var_names,
                                &mut out,
                                counters,
                                &mut visiting,
                            )
                            .map(|(n, _)| n)
                            .unwrap_or_else(|| "0.0".to_string()),
                        None => "0.0".to_string(),
                    };
                    out.push(IRStmt::SetVar {
                        name: name.clone(),
                        value_var,
                    });
                    current = self.find_exec_target(id, 0);
                }
                NodeKind::Print => {
                    let mut visiting = Vec::new();
                    match self.find_source_for_input(id, 0) {
                        Some(src) => match self.build_expr_ir(
                            src.node_id,
                            &mut var_names,
                            &mut out,
                            counters,
                            &mut visiting,
                        ) {
                            Some((v, _)) => out.push(IRStmt::Print { value_var: v }),
                            None => out.push(IRStmt::Comment(format!(
                                "Print node {} has a cycle or missing input",
                                id
                            ))),
                        },
                        None => out.push(IRStmt::Comment(format!(
                            "Print node {} has no input connected",
                            id
                        ))),
                    }
                    current = self.find_exec_target(id, 0);
                }
                NodeKind::WhileLoop => {
                    let mut cond_names: HashMap<NodeId, (String, IRType)> = HashMap::new();
                    let mut cond_lines: Vec<IRStmt> = Vec::new();
                    let mut visiting = Vec::new();
                    let cond_var = match self.find_source_for_input(id, 0) {
                        Some(src) => self
                            .build_expr_ir(
                                src.node_id,
                                &mut cond_names,
                                &mut cond_lines,
                                counters,
                                &mut visiting,
                            )
                            .map(|(n, _)| n)
                            .unwrap_or_else(|| "false".to_string()),
                        None => "false".to_string(),
                    };
                    let body_start = self.find_exec_target(id, 0);
                    let body = self.build_stmt_chain(body_start, counters);
                    out.push(IRStmt::While {
                        cond_lines,
                        cond_var,
                        body,
                    });
                    current = self.find_exec_target(id, 1);
                }
                NodeKind::FunctionCall { name } => {
                    let mut visiting_a = Vec::new();
                    let arg0 = self
                        .find_source_for_input(id, 0)
                        .and_then(|src| {
                            self.build_expr_ir(
                                src.node_id,
                                &mut var_names,
                                &mut out,
                                counters,
                                &mut visiting_a,
                            )
                        })
                        .map(|(n, _)| n)
                        .unwrap_or_else(|| "0.0".to_string());
                    let mut visiting_b = Vec::new();
                    let arg1 = self
                        .find_source_for_input(id, 1)
                        .and_then(|src| {
                            self.build_expr_ir(
                                src.node_id,
                                &mut var_names,
                                &mut out,
                                counters,
                                &mut visiting_b,
                            )
                        })
                        .map(|(n, _)| n)
                        .unwrap_or_else(|| "0.0".to_string());
                    let var_name = format!("call_{}", counters.4);
                    counters.4 += 1;
                    out.push(IRStmt::CallFunction {
                        var_name: var_name.clone(),
                        func_name: name.clone(),
                        args: vec![arg0, arg1],
                    });
                    var_names.insert(id, (var_name, IRType::Number));
                    current = self.find_exec_target(id, 0);
                }
                NodeKind::Return => {
                    let mut visiting = Vec::new();
                    let value_var = match self.find_source_for_input(id, 0) {
                        Some(src) => self
                            .build_expr_ir(
                                src.node_id,
                                &mut var_names,
                                &mut out,
                                counters,
                                &mut visiting,
                            )
                            .map(|(n, _)| n)
                            .unwrap_or_else(|| "0.0".to_string()),
                        None => "0.0".to_string(),
                    };
                    out.push(IRStmt::Return(value_var));
                    current = None;
                }
                _ => {
                    current = None;
                }
            }
        }

        out
    }

    fn build_full_ir(&self) -> (Vec<IRFunction>, Vec<IRStmt>) {
        let mut counters: Counters = (0, 0, 0, 0, 0, 0);
        let mut functions_ir: Vec<IRFunction> = Vec::new();

        let mut func_ids: Vec<NodeId> = self
            .nodes
            .iter()
            .filter(|(_, n)| matches!(n.kind, NodeKind::FunctionDef { .. }))
            .map(|(id, _)| *id)
            .collect();
        func_ids.sort_unstable();

        for id in func_ids {
            if let Some(node) = self.nodes.get(&id) {
                if let NodeKind::FunctionDef { name, params } = &node.kind {
                    let plist: Vec<String> = params
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    let body_start = self.find_exec_target(id, 0);
                    let body = self.build_stmt_chain(body_start, &mut counters);
                    functions_ir.push(IRFunction {
                        name: name.clone(),
                        params: plist,
                        body,
                    });
                }
            }
        }

        let main_stmts = if let Some(start_id) = self.find_start_node() {
            let first = self.find_exec_target(start_id, 0);
            self.build_stmt_chain(first, &mut counters)
        } else {
            let mut out = Vec::new();
            let mut var_names: HashMap<NodeId, (String, IRType)> = HashMap::new();
            let mut node_ids: Vec<NodeId> = self.nodes.keys().copied().collect();
            node_ids.sort_unstable();

            for id in node_ids {
                if let Some(n) = self.nodes.get(&id) {
                    if matches!(n.kind, NodeKind::Print) {
                        let mut visiting = Vec::new();
                        match self.find_source_for_input(id, 0) {
                            Some(src) => match self.build_expr_ir(
                                src.node_id,
                                &mut var_names,
                                &mut out,
                                &mut counters,
                                &mut visiting,
                            ) {
                                Some((v, _)) => out.push(IRStmt::Print { value_var: v }),
                                None => out.push(IRStmt::Comment(format!(
                                    "Print node {} has a cycle or missing input",
                                    id
                                ))),
                            },
                            None => out.push(IRStmt::Comment(format!(
                                "Print node {} has no input connected",
                                id
                            ))),
                        }
                    }
                }
            }
            out
        };

        (functions_ir, main_stmts)
    }

    fn generate_code_for(&self, language: TargetLanguage) -> String {
        let (functions, main_stmts) = self.build_full_ir();
        match language {
            TargetLanguage::Python => emit_python(&functions, &main_stmts),
            TargetLanguage::Rust => emit_rust(&functions, &main_stmts),
            TargetLanguage::JavaScript => emit_javascript(&functions, &main_stmts),
            TargetLanguage::Cpp => emit_cpp(&functions, &main_stmts),
        }
    }

    fn export_code(&mut self) {
        let code = self.generate_code_for(self.current_language);
        let filename = format!("blocko_export.{}", self.current_language.file_extension());

        match std::fs::write(&filename, code) {
            Ok(_) => {
                self.status_message = format!(
                    "Exported {} source to {}",
                    self.current_language.label(),
                    filename
                )
            }
            Err(e) => self.status_message = format!("Export failed: {}", e),
        }
    }

    fn save_project(&mut self) {
        let mut nodes: Vec<SerializableNode> = self
            .nodes
            .values()
            .map(|n| SerializableNode {
                id: n.id,
                kind: n.kind.clone(),
                x: n.pos.x,
                y: n.pos.y,
            })
            .collect();
        nodes.sort_by_key(|n| n.id);

        let project = ProjectFile {
            version: 3,
            next_id: self.next_id,
            nodes,
            connections: self.connections.clone(),
        };

        match serde_json::to_string_pretty(&project) {
            Ok(json_text) => match std::fs::write(PROJECT_FILE, json_text) {
                Ok(_) => self.status_message = format!("Project saved to {}", PROJECT_FILE),
                Err(e) => self.status_message = format!("Save failed: {}", e),
            },
            Err(e) => self.status_message = format!("Serialization failed: {}", e),
        }
    }

    fn load_project(&mut self) {
        let contents = match std::fs::read_to_string(PROJECT_FILE) {
            Ok(text) => text,
            Err(e) => {
                self.status_message =
                    format!("Load failed: could not read {} ({})", PROJECT_FILE, e);
                return;
            }
        };

        let project: ProjectFile = match serde_json::from_str(&contents) {
            Ok(p) => p,
            Err(e) => {
                self.status_message = format!("Load failed: invalid JSON ({})", e);
                return;
            }
        };

        self.nodes.clear();
        self.connections.clear();
        self.pin_positions.clear();
        self.dragging_connection = None;
        self.console_lines.clear();

        for sn in project.nodes {
            self.nodes.insert(
                sn.id,
                Node {
                    id: sn.id,
                    kind: sn.kind,
                    pos: Pos2::new(sn.x, sn.y),
                },
            );
        }

        self.connections = project.connections;
        self.next_id = project.next_id;

        self.status_message = format!(
            "Project loaded from {} ({} nodes, {} connections)",
            PROJECT_FILE,
            self.nodes.len(),
            self.connections.len()
        );
    }

    /// Stage 15: writes the current graph as a project package directory
    /// (`project.blocko` + `nodes/` + `assets/`) at `self.project_dir`.
    fn save_project_package(&mut self) {
        let mut nodes: Vec<SerializableNode> = self
            .nodes
            .values()
            .map(|n| SerializableNode {
                id: n.id,
                kind: n.kind.clone(),
                x: n.pos.x,
                y: n.pos.y,
            })
            .collect();
        nodes.sort_by_key(|n| n.id);

        let project_name = self
            .project_dir
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "BlockoProject".to_string());

        match project::save_package(
            &self.project_dir,
            &project_name,
            self.next_id,
            nodes,
            self.connections.clone(),
        ) {
            Ok(()) => {
                self.status_message =
                    format!("Project package saved to {}", self.project_dir.display());
            }
            Err(e) => {
                self.status_message = format!("Package save failed: {}", e);
            }
        }
    }

    /// Stage 15: loads a project package directory written by
    /// `save_project_package`.
    fn load_project_package(&mut self) {
        match project::load_package(&self.project_dir) {
            Ok((manifest, graph)) => {
                self.nodes.clear();
                self.connections.clear();
                self.pin_positions.clear();
                self.dragging_connection = None;
                self.console_lines.clear();

                for sn in graph.nodes {
                    self.nodes.insert(
                        sn.id,
                        Node {
                            id: sn.id,
                            kind: sn.kind,
                            pos: Pos2::new(sn.x, sn.y),
                        },
                    );
                }
                self.connections = graph.connections;
                self.next_id = manifest.next_id;

                self.status_message = format!(
                    "Loaded package \"{}\" ({} nodes, {} connections)",
                    manifest.project_name,
                    self.nodes.len(),
                    self.connections.len()
                );
            }
            Err(e) => {
                self.status_message = format!(
                    "Package load failed from {}: {}",
                    self.project_dir.display(),
                    e
                );
            }
        }
    }

    /// Stage 15: ingests an AI/externally-generated graph schema, validating
    /// it and remapping ids before merging it into the live graph. Runs
    /// ahead of type-checking/compiling so bad input fails fast with a
    /// readable error instead of a confusing IR or codegen crash.
    fn import_ai_graph(&mut self, schema: &AiGraphSchema) -> Result<usize, Vec<AiImportError>> {
        let mut errors = Vec::new();
        let mut seen_ids: HashSet<u64> = HashSet::new();
        for spec in &schema.nodes {
            if !seen_ids.insert(spec.id) {
                errors.push(AiImportError::DuplicateNodeId(spec.id));
            }
        }
        for conn in &schema.connections {
            if !seen_ids.contains(&conn.from_node) {
                errors.push(AiImportError::DanglingConnection { node_id: conn.from_node });
            }
            if !seen_ids.contains(&conn.to_node) {
                errors.push(AiImportError::DanglingConnection { node_id: conn.to_node });
            }
        }

        // Pre-resolve every NodeKind so an unknown-kind error is reported
        // before we mutate any state.
        let mut resolved_kinds: HashMap<u64, NodeKind> = HashMap::new();
        for spec in &schema.nodes {
            match spec.to_node_kind() {
                Ok(kind) => {
                    resolved_kinds.insert(spec.id, kind);
                }
                Err(e) => errors.push(e),
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        // Remap AI-schema ids -> fresh internal NodeIds so imports never
        // collide with the existing graph.
        let mut id_map: HashMap<u64, NodeId> = HashMap::new();
        let mut inserted = 0usize;
        for spec in &schema.nodes {
            let new_id = self.next_id;
            self.next_id += 1;
            id_map.insert(spec.id, new_id);
            let kind = resolved_kinds.remove(&spec.id).unwrap();
            self.nodes.insert(
                new_id,
                Node {
                    id: new_id,
                    kind,
                    pos: Pos2::new(spec.x, spec.y),
                },
            );
            inserted += 1;
        }

        for conn in &schema.connections {
            if let (Some(&from_id), Some(&to_id)) =
                (id_map.get(&conn.from_node), id_map.get(&conn.to_node))
            {
                self.connections.push(Connection {
                    from: PinRef {
                        node_id: from_id,
                        kind: PinKind::Output,
                        index: conn.from_index,
                        is_exec: conn.from_exec,
                    },
                    to: PinRef {
                        node_id: to_id,
                        kind: PinKind::Input,
                        index: conn.to_index,
                        is_exec: conn.to_exec,
                    },
                });
            }
        }

        self.pin_positions.clear();
        self.status_message = format!(
            "Imported AI graph: {} node(s), {} connection(s).",
            inserted,
            schema.connections.len()
        );
        Ok(inserted)
    }

    /// Convenience wrapper: reads a JSON `AiGraphSchema` from disk (e.g. the
    /// output of an LLM prompt-to-graph pipeline) and imports it.
    fn import_ai_graph_from_file(&mut self, path: &str) {
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) => {
                self.status_message = format!("AI graph import failed: could not read {} ({})", path, e);
                return;
            }
        };
        let schema: AiGraphSchema = match serde_json::from_str(&text) {
            Ok(s) => s,
            Err(e) => {
                self.status_message = format!("AI graph import failed: invalid JSON ({})", e);
                return;
            }
        };
        if let Err(errors) = self.import_ai_graph(&schema) {
            let joined = errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ");
            self.status_message = format!("AI graph import rejected: {}", joined);
        }
    }
}

fn pin_color(data_type: PinDataType) -> Color32 {
    match data_type {
        PinDataType::Number => theme::PIN_NUMBER,
        PinDataType::Bool => theme::PIN_BOOL,
        PinDataType::Any => theme::PIN_ANY,
        PinDataType::Exec => theme::PIN_EXEC,
    }
}

impl eframe::App for BlockoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Save Project").clicked() {
                        self.save_project();
                        ui.close_menu();
                    }
                    if ui.button("Load Project").clicked() {
                        self.load_project();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Clear Canvas").clicked() {
                        self.nodes.clear();
                        self.connections.clear();
                        self.pin_positions.clear();
                        self.console_lines.clear();
                        self.status_message = "Canvas cleared.".to_string();
                        ui.close_menu();
                    }
                });

                ui.separator();
                if ui.button("Run Program").clicked() {
                    self.run_program();
                }
                ui.separator();
                if ui.button("Export Source").clicked() {
                    self.export_code();
                }
                ui.separator();
                if ui.button("Save").clicked() {
                    self.save_project();
                }
                if ui.button("Load").clicked() {
                    self.load_project();
                }
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

        egui::TopBottomPanel::bottom("console_panel")
            .resizable(true)
            .default_height(170.0)
            .height_range(90.0..=420.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Console")
                            .size(15.0)
                            .strong()
                            .color(theme::TEXT_PRIMARY),
                    );
                    if ui.small_button("Clear").clicked() {
                        self.console_lines.clear();
                    }
                });
                ui.add_space(4.0);
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for line in &self.console_lines {
                            let lower = line.to_lowercase();
                            let color = if lower.contains("error")
                                || lower.contains("failed")
                                || lower.contains("could not")
                            {
                                theme::CONSOLE_ERR
                            } else if lower.contains("executed")
                                || lower.contains("no errors")
                                || lower.contains("saved")
                                || lower.contains("loaded")
                            {
                                theme::CONSOLE_OK
                            } else {
                                theme::TEXT_MUTED
                            };
                            ui.label(
                                egui::RichText::new(line)
                                    .monospace()
                                    .size(12.5)
                                    .color(color),
                            );
                        }
                    });
            });

        egui::SidePanel::right("code_preview_panel")
            .resizable(true)
            .default_width(430.0)
            .width_range(280.0..=750.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Code Preview")
                        .size(18.0)
                        .strong()
                        .color(theme::TEXT_PRIMARY),
                );
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    for lang in [
                        TargetLanguage::Python,
                        TargetLanguage::Rust,
                        TargetLanguage::JavaScript,
                        TargetLanguage::Cpp,
                    ] {
                        if ui
                            .selectable_label(self.current_language == lang, lang.label())
                            .clicked()
                        {
                            self.current_language = lang;
                        }
                    }
                });

                ui.add_space(6.0);
                if ui
                    .add(
                        egui::Button::new("Export Source")
                            .min_size(egui::vec2(ui.available_width(), 26.0)),
                    )
                    .clicked()
                {
                    self.export_code();
                }

                ui.add_space(8.0);

                let code = self.generate_code_for(self.current_language);
                code_preview_view(ui, &code, self.current_language);
            });

        egui::SidePanel::left("toolbox_panel")
            .resizable(true)
            .default_width(240.0)
            .width_range(180.0..=420.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("Toolbox")
                                .size(18.0)
                                .strong()
                                .color(theme::TEXT_PRIMARY),
                        );

                        section_header(ui, "Numbers");
                        if toolbox_button(ui, theme::ACCENT_NUMBERS, "Add Number") {
                            self.add_node(NodeKind::Number(0.0));
                        }
                        if toolbox_button(ui, theme::ACCENT_NUMBERS, "Add Math (Add)") {
                            self.add_node(NodeKind::Add);
                        }
                        if toolbox_button(ui, theme::ACCENT_NUMBERS, "Add Math (Subtract)") {
                            self.add_node(NodeKind::Sub);
                        }
                        if toolbox_button(ui, theme::ACCENT_NUMBERS, "Add Math (Multiply)") {
                            self.add_node(NodeKind::Mul);
                        }
                        if toolbox_button(ui, theme::ACCENT_NUMBERS, "Add Math (Divide)") {
                            self.add_node(NodeKind::Div);
                        }

                        section_header(ui, "Logic");
                        if toolbox_button(ui, theme::ACCENT_LOGIC, "Add Compare") {
                            self.add_node(NodeKind::Compare(CompareOp::GreaterThan));
                        }
                        if toolbox_button(ui, theme::ACCENT_LOGIC, "Add If / Else") {
                            self.add_node(NodeKind::Branch);
                        }
                        if toolbox_button(ui, theme::ACCENT_LOGIC, "Add Logic (AND)") {
                            self.add_node(NodeKind::And);
                        }
                        if toolbox_button(ui, theme::ACCENT_LOGIC, "Add Logic (OR)") {
                            self.add_node(NodeKind::Or);
                        }
                        if toolbox_button(ui, theme::ACCENT_LOGIC, "Add Logic (NOT)") {
                            self.add_node(NodeKind::Not);
                        }

                        section_header(ui, "Flow");
                        if toolbox_button(ui, theme::ACCENT_FLOW, "Add Start") {
                            self.add_node(NodeKind::Start);
                        }
                        if toolbox_button(ui, theme::ACCENT_FLOW, "Add Set Variable") {
                            self.add_node(NodeKind::SetVariable("x".to_string()));
                        }
                        if toolbox_button(ui, theme::ACCENT_FLOW, "Add Get Variable") {
                            self.add_node(NodeKind::GetVariable("x".to_string()));
                        }
                        if toolbox_button(ui, theme::ACCENT_FLOW, "Add While Loop") {
                            self.add_node(NodeKind::WhileLoop);
                        }
                        if toolbox_button(ui, theme::ACCENT_FLOW, "Add Print") {
                            self.add_node(NodeKind::Print);
                        }

                        section_header(ui, "Functions");
                        if toolbox_button(ui, theme::ACCENT_FUNCTIONS, "Add Function Def") {
                            self.add_node(NodeKind::FunctionDef {
                                name: "my_func".to_string(),
                                params: "a, b".to_string(),
                            });
                        }
                        if toolbox_button(ui, theme::ACCENT_FUNCTIONS, "Add Call Function") {
                            self.add_node(NodeKind::FunctionCall {
                                name: "my_func".to_string(),
                            });
                        }
                        if toolbox_button(ui, theme::ACCENT_FUNCTIONS, "Add Return") {
                            self.add_node(NodeKind::Return);
                        }

                        section_header(ui, "Layout");
                        if toolbox_button(ui, theme::ACCENT_FLOW, "🧭 Arrange Graph") {
                            self.auto_arrange();
                        }
                        ui.label(
                            egui::RichText::new("Untangles wires by dependency depth.")
                                .size(10.5)
                                .color(theme::TEXT_MUTED),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("Space / Ctrl+K — quick search")
                                .size(10.5)
                                .color(theme::TEXT_MUTED),
                        );

                        section_header(ui, "Project");
                        if toolbox_button(ui, theme::ACCENT_PROJECT, "Save Project (.json)") {
                            self.save_project();
                        }
                        if toolbox_button(ui, theme::ACCENT_PROJECT, "Load Project (.json)") {
                            self.load_project();
                        }
                        if toolbox_button(ui, theme::ACCENT_PROJECT, "💾 Save Project Package") {
                            self.save_project_package();
                        }
                        if toolbox_button(ui, theme::ACCENT_PROJECT, "📂 Load Project Package") {
                            self.load_project_package();
                        }
                        if toolbox_button(ui, theme::ACCENT_PROJECT, "🤖 Import AI Graph") {
                            self.import_ai_graph_from_file("ai_graph.json");
                        }
                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new(format!("Legacy file: {}", PROJECT_FILE))
                                .size(10.5)
                                .color(theme::TEXT_MUTED),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "Package dir: {}",
                                self.project_dir.display()
                            ))
                            .size(10.5)
                            .color(theme::TEXT_MUTED),
                        );
                    });
            });

        // Stage 14: global keyboard shortcuts (Space / Ctrl+K) open quick search.
        let quick_search_toggled = ctx.input(|i| {
            i.key_pressed(egui::Key::Space) && !i.pointer.any_down()
                || (i.modifiers.command && i.key_pressed(egui::Key::K))
        });
        if quick_search_toggled && !self.quick_search.open {
            self.quick_search.open_at_cursor();
            // Land new nodes roughly in the middle of the current viewport.
            self.quick_search.spawn_at = Pos2::new(
                self.camera.pan.x + 320.0 / self.camera.zoom,
                self.camera.pan.y + 220.0 / self.camera.zoom,
            );
        } else if quick_search_toggled {
            self.quick_search.close();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let origin = ui.min_rect().min;
            let canvas_rect = ui.max_rect();
            let z = self.camera.zoom;

            let painter = ui.painter();
            painter.rect_filled(canvas_rect, 0.0, Color32::BLACK);

            // Grid is drawn in world space so it pans/zooms with the content.
            let grid_spacing = 24.0 * z;
            let dot_color = Color32::from_gray(35);
            if grid_spacing > 3.0 {
                let world_min = self.camera.screen_to_world(origin, canvas_rect.min);
                let start_x = canvas_rect.left() - (world_min.x.rem_euclid(24.0)) * z;
                let start_y = canvas_rect.top() - (world_min.y.rem_euclid(24.0)) * z;
                let mut x = start_x;
                while x < canvas_rect.right() {
                    let mut y = start_y;
                    while y < canvas_rect.bottom() {
                        painter.circle_filled(Pos2::new(x, y), 1.0, dot_color);
                        y += grid_spacing;
                    }
                    x += grid_spacing;
                }
            }

            let bg_response = ui.interact(
                canvas_rect,
                ui.id().with("canvas_bg"),
                Sense::click_and_drag(),
            );

            // Pan: middle-mouse, right-mouse, or plain left-drag on empty canvas.
            if bg_response.dragged() {
                self.camera.pan -= bg_response.drag_delta() / z;
            }

            // Zoom: mouse wheel, centered on the cursor.
            if canvas_rect.contains(ctx.input(|i| i.pointer.hover_pos()).unwrap_or_default()) {
                let scroll = ctx.input(|i| i.raw_scroll_delta.y);
                if scroll.abs() > 0.0 {
                    if let Some(hover) = ctx.input(|i| i.pointer.hover_pos()) {
                        let factor = (scroll * 0.0015).exp();
                        self.camera.zoom_at(origin, hover, factor);
                    }
                }
            }

            for conn in &self.connections {
                if let (Some(&from_pos), Some(&to_pos)) = (
                    self.pin_positions.get(&conn.from),
                    self.pin_positions.get(&conn.to),
                ) {
                    let color = if conn.from.is_exec {
                        theme::PIN_EXEC
                    } else {
                        theme::PIN_NUMBER
                    };
                    draw_wire(ui.painter(), from_pos, to_pos, color);
                }
            }

            let mut node_ids: Vec<NodeId> = self.nodes.keys().copied().collect();
            node_ids.sort_unstable();

            let mut nodes_to_disconnect: Vec<NodeId> = Vec::new();
            let mut new_connection: Option<Connection> = None;
            let mut cancel_drag = false;

            for node_id in node_ids {
                let node = self.nodes.get_mut(&node_id).unwrap();
                let height = BlockoApp::node_height(&node.kind) * z;
                let widget_extra = node.kind.widget_extra_height() * z;
                // Structural constants scaled to the current zoom level. The canvas
                // pans/zooms as a real camera; node box, pins, and wires all track it.
                let node_w = NODE_WIDTH * z;
                let title_h = TITLE_HEIGHT * z;
                let row_h = ROW_HEIGHT * z;
                let body_pad = BODY_PADDING * z;
                let pin_r = PIN_RADIUS * z;
                let screen_pos = self.camera.world_to_screen(origin, node.pos);
                let node_rect = Rect::from_min_size(screen_pos, Vec2::new(node_w, height));
                let title_rect =
                    Rect::from_min_size(screen_pos, Vec2::new(node_w, title_h));

                let painter = ui.painter();
                painter.rect_filled(node_rect, 8.0, theme::BG_NODE);
                painter.rect_filled(title_rect, 8.0, theme::BG_NODE_HEADER);
                painter.rect_filled(
                    Rect::from_min_size(
                        Pos2::new(node_rect.left(), node_rect.top() + title_h - 8.0),
                        Vec2::new(node_w, 8.0),
                    ),
                    0.0,
                    theme::BG_NODE_HEADER,
                );
                painter.text(
                    Pos2::new(node_rect.left() + 12.0, title_rect.center().y),
                    Align2::LEFT_CENTER,
                    node.kind.title(),
                    FontId::proportional(14.0 * z),
                    theme::TEXT_PRIMARY,
                );
                painter.line_segment(
                    [
                        Pos2::new(node_rect.left(), node_rect.top() + title_h),
                        Pos2::new(node_rect.right(), node_rect.top() + title_h),
                    ],
                    Stroke::new(1.0, theme::BORDER_SOFT),
                );
                painter.rect_stroke(node_rect, 8.0, Stroke::new(1.2, theme::BORDER));

                let drag_id = ui.id().with(("node_drag", node_id));
                let drag_response = ui.interact(title_rect, drag_id, Sense::click_and_drag());
                if drag_response.dragged() {
                    node.pos += drag_response.drag_delta() / z;
                }
                if drag_response.double_clicked() {
                    nodes_to_disconnect.push(node_id);
                }

                if let NodeKind::Compare(op) = &mut node.kind {
                    let row_y = screen_pos.y + title_h + row_h * 0.5;
                    let painter = ui.painter();
                    painter.text(
                        Pos2::new(node_rect.left() + 12.0, row_y),
                        Align2::LEFT_CENTER,
                        "Op",
                        FontId::proportional(12.0 * z),
                        theme::TEXT_MUTED,
                    );
                    let combo_rect = Rect::from_min_size(
                        Pos2::new(node_rect.left() + 58.0, row_y - 9.0),
                        Vec2::new(node_w - 70.0, 18.0),
                    );
                    ui.allocate_ui_at_rect(combo_rect, |ui| {
                        egui::ComboBox::from_id_source(("compare_op", node_id))
                            .selected_text(op.label())
                            .show_ui(ui, |ui| {
                                for candidate in [
                                    CompareOp::GreaterThan,
                                    CompareOp::LessThan,
                                    CompareOp::EqualTo,
                                ] {
                                    ui.selectable_value(op, candidate, candidate.label());
                                }
                            });
                    });
                }

                match &mut node.kind {
                    NodeKind::Number(value) => {
                        let row_y = screen_pos.y + title_h + body_pad + row_h * 0.5;
                        let painter = ui.painter();
                        painter.text(
                            Pos2::new(node_rect.left() + 12.0, row_y),
                            Align2::LEFT_CENTER,
                            "Value",
                            FontId::proportional(12.0 * z),
                            theme::TEXT_MUTED,
                        );
                        let value_rect = Rect::from_min_size(
                            Pos2::new(node_rect.left() + 58.0, row_y - 9.0),
                            Vec2::new(node_w - 70.0, 18.0),
                        );
                        ui.put(value_rect, egui::DragValue::new(value).speed(0.1));
                    }
                    NodeKind::SetVariable(name) | NodeKind::GetVariable(name) => {
                        let row_y = screen_pos.y + title_h + row_h * 0.5;
                        let painter = ui.painter();
                        painter.text(
                            Pos2::new(node_rect.left() + 12.0, row_y),
                            Align2::LEFT_CENTER,
                            "Name",
                            FontId::proportional(12.0 * z),
                            theme::TEXT_MUTED,
                        );
                        let box_rect = Rect::from_min_size(
                            Pos2::new(node_rect.left() + 58.0, row_y - 9.0),
                            Vec2::new(node_w - 70.0, 18.0),
                        );
                        ui.put(box_rect, egui::TextEdit::singleline(name).hint_text("name"));
                    }
                    NodeKind::FunctionDef { name, params } => {
                        let row_y1 = screen_pos.y + title_h + row_h * 0.5;
                        let painter = ui.painter();
                        painter.text(
                            Pos2::new(node_rect.left() + 12.0, row_y1),
                            Align2::LEFT_CENTER,
                            "Name",
                            FontId::proportional(12.0 * z),
                            theme::TEXT_MUTED,
                        );
                        let name_rect = Rect::from_min_size(
                            Pos2::new(node_rect.left() + 58.0, row_y1 - 9.0),
                            Vec2::new(node_w - 70.0, 18.0),
                        );
                        ui.put(
                            name_rect,
                            egui::TextEdit::singleline(name).hint_text("function name"),
                        );

                        let row_y2 = screen_pos.y + title_h + row_h * 1.5;
                        let painter = ui.painter();
                        painter.text(
                            Pos2::new(node_rect.left() + 12.0, row_y2),
                            Align2::LEFT_CENTER,
                            "Params",
                            FontId::proportional(12.0 * z),
                            theme::TEXT_MUTED,
                        );
                        let params_rect = Rect::from_min_size(
                            Pos2::new(node_rect.left() + 58.0, row_y2 - 9.0),
                            Vec2::new(node_w - 70.0, 18.0),
                        );
                        ui.put(
                            params_rect,
                            egui::TextEdit::singleline(params).hint_text("param1, param2"),
                        );
                    }
                    NodeKind::FunctionCall { name } => {
                        let row_y = screen_pos.y + title_h + row_h * 0.5;
                        let painter = ui.painter();
                        painter.text(
                            Pos2::new(node_rect.left() + 12.0, row_y),
                            Align2::LEFT_CENTER,
                            "Name",
                            FontId::proportional(12.0 * z),
                            theme::TEXT_MUTED,
                        );
                        let name_rect = Rect::from_min_size(
                            Pos2::new(node_rect.left() + 58.0, row_y - 9.0),
                            Vec2::new(node_w - 70.0, 18.0),
                        );
                        ui.put(
                            name_rect,
                            egui::TextEdit::singleline(name).hint_text("function name"),
                        );
                    }
                    _ => {}
                }

                let exec_in_labels = node.kind.exec_input_labels();
                let exec_out_labels = node.kind.exec_output_labels();
                let exec_rows = exec_in_labels.len().max(exec_out_labels.len());

                for row in 0..exec_rows {
                    let row_y = screen_pos.y
                        + title_h
                        + widget_extra
                        + body_pad
                        + row_h * row as f32
                        + row_h * 0.5;

                    if row < exec_in_labels.len() {
                        let pin_pos = Pos2::new(node_rect.left(), row_y);
                        let pin_ref = PinRef {
                            node_id,
                            kind: PinKind::Input,
                            index: row,
                            is_exec: true,
                        };
                        self.pin_positions.insert(pin_ref, pin_pos);

                        let connected = is_input_connected(&self.connections, pin_ref);
                        let painter = ui.painter();
                        draw_pin(painter, pin_pos, pin_color(PinDataType::Exec), connected);
                        painter.text(
                            pin_pos + Vec2::new(10.0, 0.0),
                            Align2::LEFT_CENTER,
                            exec_in_labels[row],
                            FontId::proportional(12.0 * z),
                            theme::TEXT_MUTED,
                        );

                        let pin_rect =
                            Rect::from_center_size(pin_pos, Vec2::splat(pin_r * 3.0));
                        let pin_id = ui.id().with(("pin", node_id, "exec_in", row));
                        let pin_response = ui.interact(pin_rect, pin_id, Sense::click_and_drag());

                        if pin_response.hovered() {
                            if let Some(dragging) = &self.dragging_connection {
                                if dragging.from.kind == PinKind::Output
                                    && dragging.from.is_exec
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

                    if row < exec_out_labels.len() {
                        let pin_pos = Pos2::new(node_rect.right(), row_y);
                        let pin_ref = PinRef {
                            node_id,
                            kind: PinKind::Output,
                            index: row,
                            is_exec: true,
                        };
                        self.pin_positions.insert(pin_ref, pin_pos);

                        let connected = is_output_connected(&self.connections, pin_ref);
                        let painter = ui.painter();
                        draw_pin(painter, pin_pos, pin_color(PinDataType::Exec), connected);
                        painter.text(
                            pin_pos - Vec2::new(10.0, 0.0),
                            Align2::RIGHT_CENTER,
                            exec_out_labels[row],
                            FontId::proportional(12.0 * z),
                            theme::TEXT_MUTED,
                        );

                        let pin_rect =
                            Rect::from_center_size(pin_pos, Vec2::splat(pin_r * 3.0));
                        let pin_id = ui.id().with(("pin", node_id, "exec_out", row));
                        let pin_response = ui.interact(pin_rect, pin_id, Sense::click_and_drag());

                        if pin_response.drag_started() {
                            self.dragging_connection = Some(DraggingConnection {
                                from: pin_ref,
                                current_pos: pin_pos,
                            });
                        }
                    }
                }

                let input_labels = node.kind.input_labels();
                let output_labels = node.kind.output_labels();
                let input_types = node.kind.input_types();
                let output_types = node.kind.output_types();
                let data_rows = input_labels.len().max(output_labels.len()).max(1);
                let data_base_y = screen_pos.y
                    + title_h
                    + widget_extra
                    + body_pad
                    + exec_rows as f32 * row_h;

                for row in 0..data_rows {
                    let row_y = data_base_y + row_h * row as f32 + row_h * 0.5;

                    if row < input_labels.len() {
                        let pin_pos = Pos2::new(node_rect.left(), row_y);
                        let pin_ref = PinRef {
                            node_id,
                            kind: PinKind::Input,
                            index: row,
                            is_exec: false,
                        };
                        self.pin_positions.insert(pin_ref, pin_pos);

                        let color = pin_color(input_types[row]);
                        let connected = is_input_connected(&self.connections, pin_ref);
                        let painter = ui.painter();
                        draw_pin(painter, pin_pos, color, connected);
                        painter.text(
                            pin_pos + Vec2::new(10.0, 0.0),
                            Align2::LEFT_CENTER,
                            input_labels[row],
                            FontId::proportional(12.0 * z),
                            theme::TEXT_MUTED,
                        );

                        let pin_rect =
                            Rect::from_center_size(pin_pos, Vec2::splat(pin_r * 3.0));
                        let pin_id = ui.id().with(("pin", node_id, "in", row));
                        let pin_response = ui.interact(pin_rect, pin_id, Sense::click_and_drag());

                        if pin_response.hovered() {
                            if let Some(dragging) = &self.dragging_connection {
                                if dragging.from.kind == PinKind::Output
                                    && !dragging.from.is_exec
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
                            is_exec: false,
                        };
                        self.pin_positions.insert(pin_ref, pin_pos);

                        let color = pin_color(output_types[row]);
                        let connected = is_output_connected(&self.connections, pin_ref);
                        let painter = ui.painter();
                        draw_pin(painter, pin_pos, color, connected);
                        painter.text(
                            pin_pos - Vec2::new(10.0, 0.0),
                            Align2::RIGHT_CENTER,
                            output_labels[row],
                            FontId::proportional(12.0 * z),
                            theme::TEXT_MUTED,
                        );

                        let pin_rect =
                            Rect::from_center_size(pin_pos, Vec2::splat(pin_r * 3.0));
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
                    let color = if dragging.from.is_exec {
                        theme::PIN_EXEC
                    } else {
                        theme::ACCENT_NUMBERS
                    };
                    draw_wire(ui.painter(), from_pos, dragging.current_pos, color);
                }

                if !ctx.input(|i| i.pointer.primary_down()) {
                    cancel_drag = true;
                }
            }

            if cancel_drag {
                self.dragging_connection = None;
            }

            // Mini-map overlay (drawn last so it's always on top).
            self.draw_minimap(ui, canvas_rect);
        });

        // Stage 14: Universal Quick Search modal overlay.
        self.show_quick_search_overlay(ctx);
    }
}

impl BlockoApp {
    /// World-space axis-aligned bounding box of every node, with a small margin.
    /// Falls back to a sane default box around the origin when there are no nodes,
    /// so the mini-map never divides by zero / collapses to a point.
    fn world_bounds(&self) -> Rect {
        if self.nodes.is_empty() {
            return Rect::from_min_size(Pos2::new(-100.0, -100.0), Vec2::new(400.0, 300.0));
        }

        let mut min = Pos2::new(f32::MAX, f32::MAX);
        let mut max = Pos2::new(f32::MIN, f32::MIN);

        for node in self.nodes.values() {
            let h = BlockoApp::node_height(&node.kind);
            let node_min = node.pos;
            let node_max = node.pos + Vec2::new(NODE_WIDTH, h);
            min.x = min.x.min(node_min.x);
            min.y = min.y.min(node_min.y);
            max.x = max.x.max(node_max.x);
            max.y = max.y.max(node_max.y);
        }

        let margin = 80.0;
        Rect::from_min_max(min - Vec2::splat(margin), max + Vec2::splat(margin))
    }

    /// Renders the mini-map in the bottom-right corner of the canvas and handles
    /// click-to-jump / click-and-drag navigation.
    fn draw_minimap(&mut self, ui: &mut egui::Ui, canvas_rect: Rect) {
        const MAP_SIZE: Vec2 = Vec2::new(220.0, 160.0);
        const MAP_MARGIN: f32 = 16.0;

        let map_rect = Rect::from_min_size(
            canvas_rect.right_bottom() - MAP_SIZE - Vec2::splat(MAP_MARGIN),
            MAP_SIZE,
        );

        // World bounds of all nodes, fitted into map_rect preserving aspect ratio.
        let world_bounds = self.world_bounds();
        let world_size = world_bounds.size().max(Vec2::splat(1.0));
        let scale = (map_rect.width() / world_size.x).min(map_rect.height() / world_size.y);
        let content_size = world_size * scale;
        // Center the fitted content within the map rect (letterboxing).
        let content_origin = map_rect.center() - content_size / 2.0;

        let world_to_map = |p: Pos2| -> Pos2 {
            content_origin + (p - world_bounds.min) * scale
        };
        let map_to_world = |p: Pos2| -> Pos2 {
            world_bounds.min + (p - content_origin) / scale
        };

        let painter = ui.painter();

        // Panel background.
        painter.rect_filled(
            map_rect.expand(6.0),
            6.0,
            Color32::from_rgba_unmultiplied(18, 18, 22, 235),
        );
        painter.rect_stroke(map_rect.expand(6.0), 6.0, Stroke::new(1.0, theme::BORDER));

        // Node dots.
        for node in self.nodes.values() {
            let h = BlockoApp::node_height(&node.kind);
            let node_world_rect =
                Rect::from_min_size(node.pos, Vec2::new(NODE_WIDTH, h));
            let a = world_to_map(node_world_rect.min);
            let b = world_to_map(node_world_rect.max);
            let r = Rect::from_two_pos(a, b);
            painter.rect_filled(r, 2.0, Color32::from_rgb(90, 130, 220));
        }

        // Current viewport rectangle (semi-transparent overview box).
        let visible = self
            .camera
            .visible_world_rect(canvas_rect.min, canvas_rect);
        let viewport_a = world_to_map(visible.min);
        let viewport_b = world_to_map(visible.max);
        let viewport_rect = Rect::from_two_pos(viewport_a, viewport_b).intersect(map_rect.expand(6.0));
        painter.rect_filled(
            viewport_rect,
            2.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 40),
        );
        painter.rect_stroke(viewport_rect, 2.0, Stroke::new(1.5, Color32::WHITE));

        painter.text(
            map_rect.left_top() + Vec2::new(6.0, -14.0),
            Align2::LEFT_BOTTOM,
            "Map",
            FontId::proportional(11.0),
            theme::TEXT_MUTED,
        );

        // Click-and-drag / click-to-jump navigation: any click or drag on the
        // mini-map re-centers the camera on that point, and dragging keeps
        // following the pointer for a smooth "steer the viewport" feel.
        let map_id = ui.id().with("minimap_nav");
        let response = ui.interact(map_rect, map_id, Sense::click_and_drag());
        self.minimap_dragging = response.dragged();

        if response.dragged() || response.clicked() {
            if let Some(pointer) = response.interact_pointer_pos() {
                let target_world = map_to_world(pointer);
                self.camera.center_on(canvas_rect, target_world);
            }
        }
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