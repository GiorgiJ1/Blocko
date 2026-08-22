use eframe::egui;
use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use egui::epaint::CubicBezierShape;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

type NodeId = u64;
type Counters = (usize, usize, usize, usize, usize);

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
            NodeKind::Add
            | NodeKind::Sub
            | NodeKind::Mul
            | NodeKind::Div => vec!["A", "B"],
            NodeKind::Print => vec!["In"],
            NodeKind::Compare(_) => vec!["A", "B"],
            NodeKind::Branch => vec!["Cond", "Then", "Else"],
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
            NodeKind::Add | NodeKind::Sub | NodeKind::Mul | NodeKind::Div => vec![PinDataType::Number, PinDataType::Number],
            NodeKind::Print => vec![PinDataType::Any],
            NodeKind::Compare(_) => vec![PinDataType::Number, PinDataType::Number],
            NodeKind::Branch => vec![PinDataType::Bool, PinDataType::Number, PinDataType::Number],
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
            NodeKind::Add | NodeKind::Sub | NodeKind::Mul | NodeKind::Div => vec![PinDataType::Number],
            NodeKind::Print => vec![],
            NodeKind::Compare(_) => vec![PinDataType::Bool],
            NodeKind::Branch => vec![PinDataType::Number],
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
    SetVar { name: String, value_var: String },
    Print { value_var: String },
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
                IROp::Literal(v) => lines.push(format!("{}{} = {}", pad, s.var_name, format_literal(*v))),
                IROp::Add(a, b) => lines.push(format!("{}{} = {} + {}", pad, s.var_name, a, b)),
                IROp::Sub(a, b) => lines.push(format!("{}{} = {} - {}", pad, s.var_name, a, b)),
                IROp::Mul(a, b) => lines.push(format!("{}{} = {} * {}", pad, s.var_name, a, b)),
                IROp::Div(a, b) => lines.push(format!("{}{} = {} / {}", pad, s.var_name, a, b)),
                IROp::Compare(op, a, b) => {
                    lines.push(format!("{}{} = {} {} {}", pad, s.var_name, a, op.symbol(), b))
                }
                IROp::Branch { cond, then_val, else_val } => {
                    lines.push(format!("{}if {}:", pad, cond));
                    lines.push(format!("{}    {} = {}", pad, s.var_name, then_val));
                    lines.push(format!("{}else:", pad));
                    lines.push(format!("{}    {} = {}", pad, s.var_name, else_val));
                }
            },
            IRStmt::SetVar { name, value_var } => lines.push(format!("{}{} = {}", pad, name, value_var)),
            IRStmt::Print { value_var } => lines.push(format!("{}print({})", pad, value_var)),
            IRStmt::Comment(msg) => lines.push(format!("{}# {}", pad, msg)),
            IRStmt::CallFunction { var_name, func_name, args } => {
                lines.push(format!("{}{} = {}({})", pad, var_name, func_name, args.join(", ")))
            }
            IRStmt::Return(v) => lines.push(format!("{}return {}", pad, v)),
            IRStmt::While { cond_lines, cond_var, body } => {
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
        lines.push("# Add Start, Set Variable, Print, and While Loop nodes to see Python code.".to_string());
    } else {
        emit_python_stmts(main_stmts, 0, &mut lines);
    }

    lines.join("\n")
}

fn emit_rust_stmts(stmts: &[IRStmt], indent: usize, lines: &mut Vec<String>, declared: &mut HashSet<String>) {
    let pad = "    ".repeat(indent);
    for stmt in stmts {
        match stmt {
            IRStmt::Compute(s) => match &s.op {
                IROp::Literal(v) => lines.push(format!("{}let {} = {};", pad, s.var_name, format_literal(*v))),
                IROp::Add(a, b) => lines.push(format!("{}let {} = {} + {};", pad, s.var_name, a, b)),
                IROp::Sub(a, b) => lines.push(format!("{}let {} = {} - {};", pad, s.var_name, a, b)),
                IROp::Mul(a, b) => lines.push(format!("{}let {} = {} * {};", pad, s.var_name, a, b)),
                IROp::Div(a, b) => lines.push(format!("{}let {} = {} / {};", pad, s.var_name, a, b)),
                IROp::Compare(op, a, b) => {
                    lines.push(format!("{}let {} = {} {} {};", pad, s.var_name, a, op.symbol(), b))
                }
                IROp::Branch { cond, then_val, else_val } => {
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
            IRStmt::Print { value_var } => lines.push(format!("{}println!(\"{{}}\", {});", pad, value_var)),
            IRStmt::Comment(msg) => lines.push(format!("{}// {}", pad, msg)),
            IRStmt::CallFunction { var_name, func_name, args } => {
                lines.push(format!("{}let {} = {}({});", pad, var_name, func_name, args.join(", ")))
            }
            IRStmt::Return(v) => lines.push(format!("{}return {};", pad, v)),
            IRStmt::While { cond_lines, cond_var, body } => {
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
        out.push_str(&format!("fn {}({}) -> f64 {{\n", func.name, params_sig.join(", ")));
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

fn emit_js_stmts(stmts: &[IRStmt], indent: usize, lines: &mut Vec<String>, declared: &mut HashSet<String>) {
    let pad = "  ".repeat(indent);
    for stmt in stmts {
        match stmt {
            IRStmt::Compute(s) => match &s.op {
                IROp::Literal(v) => lines.push(format!("{}let {} = {};", pad, s.var_name, format_literal(*v))),
                IROp::Add(a, b) => lines.push(format!("{}let {} = {} + {};", pad, s.var_name, a, b)),
                IROp::Sub(a, b) => lines.push(format!("{}let {} = {} - {};", pad, s.var_name, a, b)),
                IROp::Mul(a, b) => lines.push(format!("{}let {} = {} * {};", pad, s.var_name, a, b)),
                IROp::Div(a, b) => lines.push(format!("{}let {} = {} / {};", pad, s.var_name, a, b)),
                IROp::Compare(op, a, b) => {
                    lines.push(format!("{}let {} = {} {} {};", pad, s.var_name, a, op.symbol(), b))
                }
                IROp::Branch { cond, then_val, else_val } => {
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
            IRStmt::Print { value_var } => lines.push(format!("{}console.log({});", pad, value_var)),
            IRStmt::Comment(msg) => lines.push(format!("{}// {}", pad, msg)),
            IRStmt::CallFunction { var_name, func_name, args } => {
                lines.push(format!("{}let {} = {}({});", pad, var_name, func_name, args.join(", ")))
            }
            IRStmt::Return(v) => lines.push(format!("{}return {};", pad, v)),
            IRStmt::While { cond_lines, cond_var, body } => {
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
        lines.push(format!("function {}({}) {{", func.name, func.params.join(", ")));
        let mut declared = HashSet::new();
        emit_js_stmts(&func.body, 1, &mut lines, &mut declared);
        lines.push("}".to_string());
        lines.push(String::new());
    }

    if main_stmts.is_empty() && functions.is_empty() {
        lines.push("// Your generated code will appear here.".to_string());
        lines.push("// Add Start, Set Variable, Print, and While Loop nodes to see JavaScript code.".to_string());
    } else {
        let mut declared = HashSet::new();
        emit_js_stmts(main_stmts, 0, &mut lines, &mut declared);
    }

    lines.join("\n")
}

fn emit_cpp_stmts(stmts: &[IRStmt], indent: usize, lines: &mut Vec<String>, declared: &mut HashSet<String>) {
    let pad = "    ".repeat(indent);
    for stmt in stmts {
        match stmt {
            IRStmt::Compute(s) => match &s.op {
                IROp::Literal(v) => lines.push(format!("{}auto {} = {};", pad, s.var_name, format_literal(*v))),
                IROp::Add(a, b) => lines.push(format!("{}auto {} = {} + {};", pad, s.var_name, a, b)),
                IROp::Sub(a, b) => lines.push(format!("{}auto {} = {} - {};", pad, s.var_name, a, b)),
                IROp::Mul(a, b) => lines.push(format!("{}auto {} = {} * {};", pad, s.var_name, a, b)),
                IROp::Div(a, b) => lines.push(format!("{}auto {} = {} / {};", pad, s.var_name, a, b)),
                IROp::Compare(op, a, b) => {
                    lines.push(format!("{}auto {} = {} {} {};", pad, s.var_name, a, op.symbol(), b))
                }
                IROp::Branch { cond, then_val, else_val } => {
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
            IRStmt::CallFunction { var_name, func_name, args } => {
                lines.push(format!("{}auto {} = {}({});", pad, var_name, func_name, args.join(", ")))
            }
            IRStmt::Return(v) => lines.push(format!("{}return {};", pad, v)),
            IRStmt::While { cond_lines, cond_var, body } => {
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
        let params_sig: Vec<String> = func.params.iter().map(|p| format!("double {}", p)).collect();
        out.push_str(&format!("double {}({}) {{\n", func.name, params_sig.join(", ")));
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

#[derive(Serialize, Deserialize)]
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
    console_lines: Vec<String>,
    current_language: TargetLanguage,
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

    fn node_height(kind: &NodeKind) -> f32 {
        let data_rows = kind.input_labels().len().max(kind.output_labels().len());
        let exec_rows = kind.exec_input_labels().len().max(kind.exec_output_labels().len());
        let total_rows = (data_rows + exec_rows).max(1);
        TITLE_HEIGHT + kind.widget_extra_height() + total_rows as f32 * ROW_HEIGHT + BODY_PADDING * 2.0
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
        self.connections.iter().find(|c| c.to == target).map(|c| c.from)
    }

    fn find_exec_target(&self, node_id: NodeId, output_index: usize) -> Option<NodeId> {
        let target = PinRef {
            node_id,
            kind: PinKind::Output,
            index: output_index,
            is_exec: true,
        };
        self.connections.iter().find(|c| c.from == target).map(|c| c.to.node_id)
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
                let a_val = self.evaluate_output(a_source.node_id, visiting, variables, call_cache)?.as_number()?;
                let b_val = self.evaluate_output(b_source.node_id, visiting, variables, call_cache)?.as_number()?;
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
                let a_val = self.evaluate_output(a_source.node_id, visiting, variables, call_cache)?.as_number()?;
                let b_val = self.evaluate_output(b_source.node_id, visiting, variables, call_cache)?.as_number()?;
                Some(Value::Bool(op.apply(a_val, b_val)))
            }
            NodeKind::Branch => {
                let cond_source = self.find_source_for_input(node_id, 0)?;
                let then_source = self.find_source_for_input(node_id, 1)?;
                let else_source = self.find_source_for_input(node_id, 2)?;
                let cond_val = self.evaluate_output(cond_source.node_id, visiting, variables, call_cache)?.as_bool()?;
                if cond_val {
                    let then_val = self.evaluate_output(then_source.node_id, visiting, variables, call_cache)?.as_number()?;
                    Some(Value::Number(then_val))
                } else {
                    let else_val = self.evaluate_output(else_source.node_id, visiting, variables, call_cache)?.as_number()?;
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
                console.push("... execution stopped: step limit reached (possible infinite loop) ...".to_string());
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
                        .and_then(|src| self.evaluate_output(src.node_id, &mut visiting, variables, call_cache))
                        .and_then(|v| v.as_number())
                        .unwrap_or(0.0);
                    variables.insert(name.clone(), value);
                    current = self.find_exec_target(id, 0);
                }
                NodeKind::Print => {
                    let mut visiting = Vec::new();
                    match self.find_source_for_input(id, 0) {
                        Some(src) => match self.evaluate_output(src.node_id, &mut visiting, variables, call_cache) {
                            Some(value) => console.push(format!("Print[{}] -> {}", id, value)),
                            None => console.push(format!("Print[{}] -> <could not evaluate input>", id)),
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
                            .and_then(|src| self.evaluate_output(src.node_id, &mut visiting, variables, call_cache))
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        if !cond {
                            break;
                        }
                        if let Some(body_start) = self.find_exec_target(id, 0) {
                            self.exec_statement(body_start, variables, console, steps, functions, call_cache, return_slot);
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
                        .and_then(|src| self.evaluate_output(src.node_id, &mut visiting_a, variables, call_cache))
                        .and_then(|v| v.as_number())
                        .unwrap_or(0.0);
                    let mut visiting_b = Vec::new();
                    let a1 = self
                        .find_source_for_input(id, 1)
                        .and_then(|src| self.evaluate_output(src.node_id, &mut visiting_b, variables, call_cache))
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
                            self.exec_statement(*body_start, variables, console, steps, functions, call_cache, &mut inner_return);
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
                        .and_then(|src| self.evaluate_output(src.node_id, &mut visiting, variables, call_cache))
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
                Some(source) => match self.evaluate_output(source.node_id, &mut visiting, &variables, &call_cache) {
                    Some(value) => console.push(format!("Print[{}] -> {}", print_id, value)),
                    None => console.push(format!("Print[{}] -> <could not evaluate input>", print_id)),
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
                self.exec_statement(first, &mut variables, &mut console, &mut steps, &functions, &mut call_cache, &mut return_slot);
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
            NodeKind::Add
            | NodeKind::Sub
            | NodeKind::Mul
            | NodeKind::Div => {
                let a_source = self.find_source_for_input(node_id, 0)?;
                let b_source = self.find_source_for_input(node_id, 1)?;
                let (a_name, _) = self.build_expr_ir(a_source.node_id, var_names, out, counters, visiting)?;
                let (b_name, _) = self.build_expr_ir(b_source.node_id, var_names, out, counters, visiting)?;
                
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
                let (a_name, _) = self.build_expr_ir(a_source.node_id, var_names, out, counters, visiting)?;
                let (b_name, _) = self.build_expr_ir(b_source.node_id, var_names, out, counters, visiting)?;
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
            NodeKind::Branch => {
                let cond_source = self.find_source_for_input(node_id, 0)?;
                let then_source = self.find_source_for_input(node_id, 1)?;
                let else_source = self.find_source_for_input(node_id, 2)?;
                let (cond_name, _) = self.build_expr_ir(cond_source.node_id, var_names, out, counters, visiting)?;
                let (then_name, _) = self.build_expr_ir(then_source.node_id, var_names, out, counters, visiting)?;
                let (else_name, _) = self.build_expr_ir(else_source.node_id, var_names, out, counters, visiting)?;
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
                out.push(IRStmt::Comment("execution chain too long or cyclic, stopped".to_string()));
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
                            .build_expr_ir(src.node_id, &mut var_names, &mut out, counters, &mut visiting)
                            .map(|(n, _)| n)
                            .unwrap_or_else(|| "0.0".to_string()),
                        None => "0.0".to_string(),
                    };
                    out.push(IRStmt::SetVar { name: name.clone(), value_var });
                    current = self.find_exec_target(id, 0);
                }
                NodeKind::Print => {
                    let mut visiting = Vec::new();
                    match self.find_source_for_input(id, 0) {
                        Some(src) => match self.build_expr_ir(src.node_id, &mut var_names, &mut out, counters, &mut visiting) {
                            Some((v, _)) => out.push(IRStmt::Print { value_var: v }),
                            None => out.push(IRStmt::Comment(format!("Print node {} has a cycle or missing input", id))),
                        },
                        None => out.push(IRStmt::Comment(format!("Print node {} has no input connected", id))),
                    }
                    current = self.find_exec_target(id, 0);
                }
                NodeKind::WhileLoop => {
                    let mut cond_names: HashMap<NodeId, (String, IRType)> = HashMap::new();
                    let mut cond_lines: Vec<IRStmt> = Vec::new();
                    let mut visiting = Vec::new();
                    let cond_var = match self.find_source_for_input(id, 0) {
                        Some(src) => self
                            .build_expr_ir(src.node_id, &mut cond_names, &mut cond_lines, counters, &mut visiting)
                            .map(|(n, _)| n)
                            .unwrap_or_else(|| "false".to_string()),
                        None => "false".to_string(),
                    };
                    let body_start = self.find_exec_target(id, 0);
                    let body = self.build_stmt_chain(body_start, counters);
                    out.push(IRStmt::While { cond_lines, cond_var, body });
                    current = self.find_exec_target(id, 1);
                }
                NodeKind::FunctionCall { name } => {
                    let mut visiting_a = Vec::new();
                    let arg0 = self
                        .find_source_for_input(id, 0)
                        .and_then(|src| self.build_expr_ir(src.node_id, &mut var_names, &mut out, counters, &mut visiting_a))
                        .map(|(n, _)| n)
                        .unwrap_or_else(|| "0.0".to_string());
                    let mut visiting_b = Vec::new();
                    let arg1 = self
                        .find_source_for_input(id, 1)
                        .and_then(|src| self.build_expr_ir(src.node_id, &mut var_names, &mut out, counters, &mut visiting_b))
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
                            .build_expr_ir(src.node_id, &mut var_names, &mut out, counters, &mut visiting)
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
        let mut counters: Counters = (0, 0, 0, 0, 0);
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
                    functions_ir.push(IRFunction { name: name.clone(), params: plist, body });
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
                            Some(src) => match self.build_expr_ir(src.node_id, &mut var_names, &mut out, &mut counters, &mut visiting) {
                                Some((v, _)) => out.push(IRStmt::Print { value_var: v }),
                                None => out.push(IRStmt::Comment(format!("Print node {} has a cycle or missing input", id))),
                            },
                            None => out.push(IRStmt::Comment(format!("Print node {} has no input connected", id))),
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
            Ok(_) => self.status_message = format!("Exported {} source to {}", self.current_language.label(), filename),
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
                self.status_message = format!("Load failed: could not read {} ({})", PROJECT_FILE, e);
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
}

fn pin_color(kind: PinKind, data_type: PinDataType) -> Color32 {
    match (kind, data_type) {
        (PinKind::Input, PinDataType::Number) => Color32::from_rgb(220, 180, 90),
        (PinKind::Output, PinDataType::Number) => Color32::from_rgb(120, 220, 150),
        (PinKind::Input, PinDataType::Bool) => Color32::from_rgb(220, 120, 200),
        (PinKind::Output, PinDataType::Bool) => Color32::from_rgb(180, 130, 230),
        (PinKind::Input, PinDataType::Any) => Color32::from_rgb(200, 200, 200),
        (PinKind::Output, PinDataType::Any) => Color32::from_rgb(200, 200, 200),
        (PinKind::Input, PinDataType::Exec) => Color32::from_rgb(235, 235, 235),
        (PinKind::Output, PinDataType::Exec) => Color32::from_rgb(255, 255, 255),
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
            .default_height(160.0)
            .height_range(80.0..=400.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Console");
                    if ui.button("Clear").clicked() {
                        self.console_lines.clear();
                    }
                });
                ui.separator();
                egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                    for line in &self.console_lines {
                        ui.monospace(line);
                    }
                });
            });

        egui::SidePanel::right("code_preview_panel")
            .resizable(true)
            .default_width(420.0)
            .width_range(260.0..=700.0)
            .show(ctx, |ui| {
                ui.heading("Code Preview");
                ui.separator();

                ui.horizontal(|ui| {
                    for lang in [
                        TargetLanguage::Python,
                        TargetLanguage::Rust,
                        TargetLanguage::JavaScript,
                        TargetLanguage::Cpp,
                    ] {
                        if ui.selectable_label(self.current_language == lang, lang.label()).clicked() {
                            self.current_language = lang;
                        }
                    }
                });

                ui.add_space(6.0);
                if ui
                    .add(egui::Button::new("Export Source").min_size(egui::vec2(ui.available_width(), 26.0)))
                    .clicked()
                {
                    self.export_code();
                }

                ui.add_space(6.0);
                ui.separator();

                let code = self.generate_code_for(self.current_language);
                egui::ScrollArea::both().show(ui, |ui| {
                    let mut code_display = code.clone();
                    ui.add(
                        egui::TextEdit::multiline(&mut code_display)
                            .font(egui::TextStyle::Monospace)
                            .code_editor()
                            .desired_width(f32::INFINITY)
                            .interactive(false),
                    );
                });
            });

        egui::SidePanel::left("toolbox_panel")
            .resizable(true)
            .default_width(240.0)
            .width_range(150.0..=420.0)
            .show(ctx, |ui| {
                ui.heading("Toolbox");
                ui.separator();

                if ui.add(egui::Button::new("Add Number").min_size(egui::vec2(ui.available_width(), 30.0))).clicked() {
                    self.add_node(NodeKind::Number(0.0));
                }
                ui.add_space(4.0);
                if ui.add(egui::Button::new("Add Math (Add)").min_size(egui::vec2(ui.available_width(), 30.0))).clicked() {
                    self.add_node(NodeKind::Add);
                }
                ui.add_space(4.0);
                if ui.add(egui::Button::new("Add Math (Subtract)").min_size(egui::vec2(ui.available_width(), 30.0))).clicked() {
                    self.add_node(NodeKind::Sub);
                }
                ui.add_space(4.0);
                if ui.add(egui::Button::new("Add Math (Multiply)").min_size(egui::vec2(ui.available_width(), 30.0))).clicked() {
                    self.add_node(NodeKind::Mul);
                }
                ui.add_space(4.0);
                if ui.add(egui::Button::new("Add Math (Divide)").min_size(egui::vec2(ui.available_width(), 30.0))).clicked() {
                    self.add_node(NodeKind::Div);
                }
                ui.add_space(4.0);
                if ui.add(egui::Button::new("Add Print").min_size(egui::vec2(ui.available_width(), 30.0))).clicked() {
                    self.add_node(NodeKind::Print);
                }

                ui.add_space(10.0);
                ui.separator();
                ui.label("Logic:");
                ui.add_space(4.0);
                if ui.add(egui::Button::new("Add Compare").min_size(egui::vec2(ui.available_width(), 30.0))).clicked() {
                    self.add_node(NodeKind::Compare(CompareOp::GreaterThan));
                }
                ui.add_space(4.0);
                if ui.add(egui::Button::new("Add If / Else").min_size(egui::vec2(ui.available_width(), 30.0))).clicked() {
                    self.add_node(NodeKind::Branch);
                }

                ui.add_space(10.0);
                ui.separator();
                ui.label("Flow:");
                ui.add_space(4.0);
                if ui.add(egui::Button::new("Add Start").min_size(egui::vec2(ui.available_width(), 30.0))).clicked() {
                    self.add_node(NodeKind::Start);
                }
                ui.add_space(4.0);
                if ui.add(egui::Button::new("Add Set Variable").min_size(egui::vec2(ui.available_width(), 30.0))).clicked() {
                    self.add_node(NodeKind::SetVariable("x".to_string()));
                }
                ui.add_space(4.0);
                if ui.add(egui::Button::new("Add Get Variable").min_size(egui::vec2(ui.available_width(), 30.0))).clicked() {
                    self.add_node(NodeKind::GetVariable("x".to_string()));
                }
                ui.add_space(4.0);
                if ui.add(egui::Button::new("Add While Loop").min_size(egui::vec2(ui.available_width(), 30.0))).clicked() {
                    self.add_node(NodeKind::WhileLoop);
                }

                ui.add_space(10.0);
                ui.separator();
                ui.label("Functions:");
                ui.add_space(4.0);
                if ui.add(egui::Button::new("Add Function Def").min_size(egui::vec2(ui.available_width(), 30.0))).clicked() {
                    self.add_node(NodeKind::FunctionDef {
                        name: "my_func".to_string(),
                        params: "a, b".to_string(),
                    });
                }
                ui.add_space(4.0);
                if ui.add(egui::Button::new("Add Call Function").min_size(egui::vec2(ui.available_width(), 30.0))).clicked() {
                    self.add_node(NodeKind::FunctionCall { name: "my_func".to_string() });
                }
                ui.add_space(4.0);
                if ui.add(egui::Button::new("Add Return").min_size(egui::vec2(ui.available_width(), 30.0))).clicked() {
                    self.add_node(NodeKind::Return);
                }

                ui.add_space(10.0);
                ui.separator();
                ui.label("Project:");
                ui.add_space(4.0);
                if ui.add(egui::Button::new("Save Project").min_size(egui::vec2(ui.available_width(), 28.0))).clicked() {
                    self.save_project();
                }
                ui.add_space(4.0);
                if ui.add(egui::Button::new("Load Project").min_size(egui::vec2(ui.available_width(), 28.0))).clicked() {
                    self.load_project();
                }
                ui.add_space(4.0);
                ui.small(format!("File: {}", PROJECT_FILE));
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let origin = ui.min_rect().min;
            let canvas_rect = ui.max_rect();

            let painter = ui.painter();
            painter.rect_filled(canvas_rect, 0.0, Color32::BLACK);

            let grid_spacing = 24.0;
            let dot_color = Color32::from_gray(35);
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
                if let (Some(&from_pos), Some(&to_pos)) = (self.pin_positions.get(&conn.from), self.pin_positions.get(&conn.to)) {
                    let color = if conn.from.is_exec {
                        Color32::from_rgb(255, 255, 255)
                    } else {
                        Color32::from_rgb(120, 180, 255)
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
                let height = BlockoApp::node_height(&node.kind);
                let widget_extra = node.kind.widget_extra_height();
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

                if let NodeKind::Compare(op) = &mut node.kind {
                    let combo_rect = Rect::from_min_size(
                        screen_pos + Vec2::new(8.0, TITLE_HEIGHT + 2.0),
                        Vec2::new(NODE_WIDTH - 16.0, ROW_HEIGHT - 4.0),
                    );
                    ui.allocate_ui_at_rect(combo_rect, |ui| {
                        egui::ComboBox::from_id_source(("compare_op", node_id))
                            .selected_text(op.label())
                            .show_ui(ui, |ui| {
                                for candidate in [CompareOp::GreaterThan, CompareOp::LessThan, CompareOp::EqualTo] {
                                    ui.selectable_value(op, candidate, candidate.label());
                                }
                            });
                    });
                }

                match &mut node.kind {
                    NodeKind::Number(value) => {
                        let row_y = screen_pos.y + TITLE_HEIGHT + BODY_PADDING + ROW_HEIGHT * 0.5;
                        let value_rect = Rect::from_center_size(
                            Pos2::new(node_rect.left() + NODE_WIDTH * 0.42, row_y),
                            Vec2::new(60.0, 18.0),
                        );
                        ui.put(value_rect, egui::DragValue::new(value).speed(0.1));
                    }
                    NodeKind::SetVariable(name) | NodeKind::GetVariable(name) => {
                        let box_rect = Rect::from_min_size(
                            screen_pos + Vec2::new(8.0, TITLE_HEIGHT + 2.0),
                            Vec2::new(NODE_WIDTH - 16.0, ROW_HEIGHT - 4.0),
                        );
                        ui.put(box_rect, egui::TextEdit::singleline(name).hint_text("name"));
                    }
                    NodeKind::FunctionDef { name, params } => {
                        let name_rect = Rect::from_min_size(
                            screen_pos + Vec2::new(8.0, TITLE_HEIGHT + 2.0),
                            Vec2::new(NODE_WIDTH - 16.0, ROW_HEIGHT - 4.0),
                        );
                        ui.put(name_rect, egui::TextEdit::singleline(name).hint_text("function name"));

                        let params_rect = Rect::from_min_size(
                            screen_pos + Vec2::new(8.0, TITLE_HEIGHT + 2.0 + ROW_HEIGHT),
                            Vec2::new(NODE_WIDTH - 16.0, ROW_HEIGHT - 4.0),
                        );
                        ui.put(params_rect, egui::TextEdit::singleline(params).hint_text("param1, param2"));
                    }
                    NodeKind::FunctionCall { name } => {
                        let name_rect = Rect::from_min_size(
                            screen_pos + Vec2::new(8.0, TITLE_HEIGHT + 2.0),
                            Vec2::new(NODE_WIDTH - 16.0, ROW_HEIGHT - 4.0),
                        );
                        ui.put(name_rect, egui::TextEdit::singleline(name).hint_text("function name"));
                    }
                    _ => {}
                }

                let exec_in_labels = node.kind.exec_input_labels();
                let exec_out_labels = node.kind.exec_output_labels();
                let exec_rows = exec_in_labels.len().max(exec_out_labels.len());

                for row in 0..exec_rows {
                    let row_y = screen_pos.y + TITLE_HEIGHT + widget_extra + BODY_PADDING + ROW_HEIGHT * row as f32 + ROW_HEIGHT * 0.5;

                    if row < exec_in_labels.len() {
                        let pin_pos = Pos2::new(node_rect.left(), row_y);
                        let pin_ref = PinRef { node_id, kind: PinKind::Input, index: row, is_exec: true };
                        self.pin_positions.insert(pin_ref, pin_pos);

                        let painter = ui.painter();
                        painter.circle_filled(pin_pos, PIN_RADIUS, pin_color(PinKind::Input, PinDataType::Exec));
                        painter.text(
                            pin_pos + Vec2::new(10.0, 0.0),
                            Align2::LEFT_CENTER,
                            exec_in_labels[row],
                            FontId::proportional(12.0),
                            Color32::from_gray(220),
                        );

                        let pin_rect = Rect::from_center_size(pin_pos, Vec2::splat(PIN_RADIUS * 3.0));
                        let pin_id = ui.id().with(("pin", node_id, "exec_in", row));
                        let pin_response = ui.interact(pin_rect, pin_id, Sense::click_and_drag());

                        if pin_response.hovered() {
                            if let Some(dragging) = &self.dragging_connection {
                                if dragging.from.kind == PinKind::Output
                                    && dragging.from.is_exec
                                    && ctx.input(|i| i.pointer.any_released())
                                {
                                    new_connection = Some(Connection { from: dragging.from, to: pin_ref });
                                }
                            }
                        }
                    }

                    if row < exec_out_labels.len() {
                        let pin_pos = Pos2::new(node_rect.right(), row_y);
                        let pin_ref = PinRef { node_id, kind: PinKind::Output, index: row, is_exec: true };
                        self.pin_positions.insert(pin_ref, pin_pos);

                        let painter = ui.painter();
                        painter.circle_filled(pin_pos, PIN_RADIUS, pin_color(PinKind::Output, PinDataType::Exec));
                        painter.text(
                            pin_pos - Vec2::new(10.0, 0.0),
                            Align2::RIGHT_CENTER,
                            exec_out_labels[row],
                            FontId::proportional(12.0),
                            Color32::from_gray(220),
                        );

                        let pin_rect = Rect::from_center_size(pin_pos, Vec2::splat(PIN_RADIUS * 3.0));
                        let pin_id = ui.id().with(("pin", node_id, "exec_out", row));
                        let pin_response = ui.interact(pin_rect, pin_id, Sense::click_and_drag());

                        if pin_response.drag_started() {
                            self.dragging_connection = Some(DraggingConnection { from: pin_ref, current_pos: pin_pos });
                        }
                    }
                }

                let input_labels = node.kind.input_labels();
                let output_labels = node.kind.output_labels();
                let input_types = node.kind.input_types();
                let output_types = node.kind.output_types();
                let data_rows = input_labels.len().max(output_labels.len()).max(1);
                let data_base_y = screen_pos.y + TITLE_HEIGHT + widget_extra + BODY_PADDING + exec_rows as f32 * ROW_HEIGHT;

                for row in 0..data_rows {
                    let row_y = data_base_y + ROW_HEIGHT * row as f32 + ROW_HEIGHT * 0.5;

                    if row < input_labels.len() {
                        let pin_pos = Pos2::new(node_rect.left(), row_y);
                        let pin_ref = PinRef { node_id, kind: PinKind::Input, index: row, is_exec: false };
                        self.pin_positions.insert(pin_ref, pin_pos);

                        let color = pin_color(PinKind::Input, input_types[row]);
                        let painter = ui.painter();
                        painter.circle_filled(pin_pos, PIN_RADIUS, color);
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
                                    && !dragging.from.is_exec
                                    && ctx.input(|i| i.pointer.any_released())
                                {
                                    new_connection = Some(Connection { from: dragging.from, to: pin_ref });
                                }
                            }
                        }
                    }

                    if row < output_labels.len() {
                        let pin_pos = Pos2::new(node_rect.right(), row_y);
                        let pin_ref = PinRef { node_id, kind: PinKind::Output, index: row, is_exec: false };
                        self.pin_positions.insert(pin_ref, pin_pos);

                        let color = pin_color(PinKind::Output, output_types[row]);
                        let painter = ui.painter();
                        painter.circle_filled(pin_pos, PIN_RADIUS, color);
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
                            self.dragging_connection = Some(DraggingConnection { from: pin_ref, current_pos: pin_pos });
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
                        Color32::from_rgb(255, 255, 255)
                    } else {
                        Color32::from_rgb(255, 210, 120)
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
        });
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