use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeAnnotation {
    Number,
    String,
    Boolean,
    List,
    Custom(String),
}

impl TypeAnnotation {
    pub fn to_rust_str(&self) -> &str {
        match self {
            TypeAnnotation::Number => "i32",
            TypeAnnotation::String => "String",
            TypeAnnotation::Boolean => "bool",
            TypeAnnotation::List => "Vec<serde_json::Value>",
            TypeAnnotation::Custom(s) => s,
        }
    }

    pub fn from_type_str(s: &str) -> Self {
        match s.trim() {
            "n" | "number" => TypeAnnotation::Number,
            "s" | "string" => TypeAnnotation::String,
            "b" | "bool" => TypeAnnotation::Boolean,
            "l" | "list" => TypeAnnotation::List,
            other => TypeAnnotation::Custom(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub ty: TypeAnnotation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AstNode {
    Function {
        name: String,
        params: Vec<Parameter>,
        return_type: TypeAnnotation,
        body: Vec<AstNode>,
    },
    VariableDecl {
        name: String,
        ty: Option<TypeAnnotation>,
        expr: String,
    },
    Assignment {
        name: String,
        expr: String,
    },
    Conditional {
        condition: String,
        then_branch: Vec<AstNode>,
        else_branch: Option<Vec<AstNode>>,
    },
    WhileLoop {
        condition: String,
        body: Vec<AstNode>,
    },
    ReadOp {
        target: String,
        action: String,
        payload: String,
    },
    WriteOp {
        target: String,
        action: String,
        payload: String,
    },
    Return(String),
    Expression(String),
}

pub struct Transpiler;

impl Default for Transpiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Transpiler {
    pub fn new() -> Self {
        Self
    }

    pub fn transpile(&self, pseudo_code: &str) -> Result<String, String> {
        let ast = self.parse(pseudo_code)?;
        Ok(self.generate_rust(&ast))
    }

    pub fn parse(&self, code: &str) -> Result<AstNode, String> {
        let trimmed = code.trim();
        if trimmed.starts_with('f') {
            self.parse_function(trimmed)
        } else {
            Err(format!("Unsupported pseudo-lang root construct: {}", trimmed))
        }
    }

    fn parse_function(&self, code: &str) -> Result<AstNode, String> {
        let rest = code.strip_prefix('f').ok_or("Invalid function declaration")?.trim();
        let paren_start = rest.find('(').ok_or("Missing parameter list start '('")?;
        let name = rest[..paren_start].trim().to_string();
        
        let paren_end = rest.find(')').ok_or("Missing parameter list end ')'")?;
        let params_str = &rest[paren_start + 1..paren_end];
        let mut params = Vec::new();
        if !params_str.trim().is_empty() {
            for p in params_str.split(',') {
                let parts: Vec<&str> = p.split(':').collect();
                if parts.len() == 2 {
                    params.push(Parameter {
                        name: parts[0].trim().to_string(),
                        ty: TypeAnnotation::from_type_str(parts[1].trim()),
                    });
                } else {
                    return Err(format!("Invalid parameter syntax: {}", p));
                }
            }
        }

        let return_type = TypeAnnotation::Number;

        let bracket_start = rest.find('[').ok_or("Missing function body start '['")?;
        let bracket_end = rest.rfind(']').ok_or("Missing function body end ']'")?;
        let body_str = &rest[bracket_start + 1..bracket_end];

        let body = self.parse_statements(body_str)?;

        Ok(AstNode::Function {
            name,
            params,
            return_type,
            body,
        })
    }

    fn parse_statements(&self, statements_str: &str) -> Result<Vec<AstNode>, String> {
        let cleaned = statements_str.trim();
        if cleaned.is_empty() {
            return Ok(Vec::new());
        }

        let mut statements = Vec::new();
        let mut current = String::new();
        let mut depth = 0;
        let mut in_quote = false;

        for c in cleaned.chars() {
            match c {
                '"' => {
                    in_quote = !in_quote;
                    current.push(c);
                }
                '[' | '{' | '(' if !in_quote => {
                    depth += 1;
                    current.push(c);
                }
                ']' | '}' | ')' if !in_quote => {
                    depth -= 1;
                    current.push(c);
                }
                ';' | '\n' if depth == 0 && !in_quote => {
                    let stmt = current.trim();
                    if !stmt.is_empty() {
                        statements.push(stmt.to_string());
                    }
                    current.clear();
                }
                _ => {
                    current.push(c);
                }
            }
        }
        let stmt = current.trim();
        if !stmt.is_empty() {
            statements.push(stmt.to_string());
        }

        let mut nodes = Vec::new();
        for s in statements {
            nodes.push(self.parse_statement(&s)?);
        }

        Ok(nodes)
    }

    fn parse_statement(&self, stmt: &str) -> Result<AstNode, String> {
        let trimmed = stmt.trim();
        if trimmed.starts_with("r ") {
            let expr = trimmed.strip_prefix("r ").unwrap().trim().to_string();
            Ok(AstNode::Return(expr))
        } else if trimmed.starts_with("v ") {
            let rest = trimmed.strip_prefix("v ").unwrap().trim();
            if let Some(eq_pos) = rest.find('=') {
                let lhs = rest[..eq_pos].trim();
                let expr = rest[eq_pos + 1..].trim().to_string();
                if let Some(colon_pos) = lhs.find(':') {
                    let name = lhs[..colon_pos].trim().to_string();
                    let ty = TypeAnnotation::from_type_str(lhs[colon_pos + 1..].trim());
                    Ok(AstNode::VariableDecl { name, ty: Some(ty), expr })
                } else {
                    Ok(AstNode::VariableDecl { name: lhs.to_string(), ty: None, expr })
                }
            } else {
                Err(format!("Invalid variable declaration: {}", stmt))
            }
        } else if trimmed.starts_with("? ") {
            let rest = trimmed.strip_prefix("? ").unwrap().trim();
            let b1_start = rest.find('[').ok_or("Conditional missing then branch '['")?;
            let b1_end = rest.find(']').ok_or("Conditional missing then branch ']'")?;
            let condition = rest[..b1_start].trim().to_string();
            let then_str = &rest[b1_start + 1..b1_end];
            let then_branch = self.parse_statements(then_str)?;

            let remaining = rest[b1_end + 1..].trim();
            let else_branch = if remaining.starts_with('[') {
                let b2_end = remaining.rfind(']').ok_or("Conditional missing else branch ']'")?;
                let else_str = &remaining[1..b2_end];
                Some(self.parse_statements(else_str)?)
            } else {
                None
            };

            Ok(AstNode::Conditional {
                condition,
                then_branch,
                else_branch,
            })
        } else if trimmed.starts_with("w ") {
            let rest = trimmed.strip_prefix("w ").unwrap().trim();
            let b_start = rest.find('[').ok_or("While loop missing body '['")?;
            let b_end = rest.rfind(']').ok_or("While loop missing body ']'")?;
            let condition = rest[..b_start].trim().to_string();
            let body_str = &rest[b_start + 1..b_end];
            let body = self.parse_statements(body_str)?;
            Ok(AstNode::WhileLoop { condition, body })
        } else if trimmed.contains('=') && !trimmed.starts_with("v ") {
            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                let name = parts[0].trim().to_string();
                let expr = parts[1].trim().to_string();
                Ok(AstNode::Assignment { name, expr })
            } else {
                Err(format!("Invalid assignment: {}", stmt))
            }
        } else {
            Ok(AstNode::Expression(trimmed.to_string()))
        }
    }

    pub fn generate_rust(&self, node: &AstNode) -> String {
        match node {
            AstNode::Function { name, params, return_type, body } => {
                let params_str = params
                    .iter()
                    .map(|p| format!("{}: {}", p.name, p.ty.to_rust_str()))
                    .collect::<Vec<_>>()
                    .join(", ");
                let body_str = body
                    .iter()
                    .map(|n| format!("    {}", self.generate_node_rust(n)))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!(
                    "pub fn {}({}) -> {} {{\n{}\n}}",
                    name,
                    params_str,
                    return_type.to_rust_str(),
                    body_str
                )
            }
            _ => self.generate_node_rust(node),
        }
    }

    fn generate_node_rust(&self, node: &AstNode) -> String {
        match node {
            AstNode::VariableDecl { name, ty, expr } => {
                let transpiled_expr = transpile_expr(expr);
                if let Some(t) = ty {
                    format!("let {}: {} = {};", name, t.to_rust_str(), transpiled_expr)
                } else {
                    format!("let {} = {};", name, transpiled_expr)
                }
            }
            AstNode::Assignment { name, expr } => {
                format!("{} = {};", name, transpile_expr(expr))
            }
            AstNode::Conditional { condition, then_branch, else_branch } => {
                let then_str = then_branch
                    .iter()
                    .map(|n| format!("    {}", self.generate_node_rust(n)))
                    .collect::<Vec<_>>()
                    .join("\n");
                if let Some(else_b) = else_branch {
                    let else_str = else_b
                        .iter()
                        .map(|n| format!("    {}", self.generate_node_rust(n)))
                        .collect::<Vec<_>>()
                        .join("\n");
                    format!(
                        "if {} {{\n{}\n}} else {{\n{}\n}}",
                        transpile_expr(condition),
                        then_str,
                        else_str
                    )
                } else {
                    format!(
                        "if {} {{\n{}\n}}",
                        transpile_expr(condition),
                        then_str
                    )
                }
            }
            AstNode::WhileLoop { condition, body } => {
                let body_str = body
                    .iter()
                    .map(|n| format!("    {}", self.generate_node_rust(n)))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!(
                    "while {} {{\n{}\n}}",
                    transpile_expr(condition),
                    body_str
                )
            }
            AstNode::ReadOp { target, action, payload } => {
                format!("sandbox.dispatch(\"{}\", \"{}\", &serde_json::json!({}))", target, action, payload)
            }
            AstNode::WriteOp { target, action, payload } => {
                format!("sandbox.dispatch(\"{}\", \"{}\", &serde_json::json!({}))", target, action, payload)
            }
            AstNode::Return(expr) => {
                transpile_expr(expr)
            }
            AstNode::Expression(expr) => {
                format!("{};", transpile_expr(expr))
            }
            AstNode::Function { .. } => "".into(),
        }
    }
}

pub fn transpile_expr(expr: &str) -> String {
    let trimmed = expr.trim();
    
    if trimmed.contains("@rd ") || trimmed.contains("@wt ") {
        return transpile_tool_op(trimmed);
    }

    if trimmed.starts_with('(') && trimmed.ends_with(')') {
        let inner = &trimmed[1..trimmed.len() - 1].trim();
        let parts = split_args(inner);
        if !parts.is_empty() {
            let op = parts[0];
            let args: Vec<String> = parts[1..].iter().map(|a| transpile_expr(a)).collect();
            match op {
                "+" | "-" | "*" | "/" | "==" | "!=" | "<" | ">" | "<=" | ">="
                    if args.len() >= 2 => {
                        return format!("({} {} {})", args[0], op, args[1]);
                    }
                _ => {}
            }
        }
    }

    trimmed.to_string()
}

fn transpile_tool_op(op_str: &str) -> String {
    if let Some(rest) = op_str.strip_prefix("@rd ") {
        parse_tool_call(rest, "read")
    } else if let Some(rest) = op_str.strip_prefix("@wt ") {
        parse_tool_call(rest, "write")
    } else {
        op_str.to_string()
    }
}

fn parse_tool_call(rest: &str, _op_type: &str) -> String {
    let parts = split_args(rest);
    if parts.len() >= 3 {
        let target = parts[0].trim_matches('"');
        let action = parts[1].trim_matches('"');
        let payload = parts[2..].join(" ");
        format!("sandbox.dispatch(\"{}\", \"{}\", &serde_json::json!({}))", target, action, payload)
    } else {
        rest.to_string()
    }
}

fn split_args(s: &str) -> Vec<&str> {
    let mut args = Vec::new();
    let mut current = 0;
    let mut depth = 0;
    let mut in_quote = false;
    let chars = s.char_indices().peekable();

    for (idx, c) in chars {
        match c {
            '"' => {
                in_quote = !in_quote;
            }
            '(' | '{' | '[' if !in_quote => {
                depth += 1;
            }
            ')' | '}' | ']' if !in_quote => {
                depth -= 1;
            }
            ' ' if depth == 0 && !in_quote => {
                if idx > current {
                    args.push(&s[current..idx]);
                }
                current = idx + 1;
            }
            _ => {}
        }
    }
    if current < s.len() {
        args.push(&s[current..]);
    }
    args
}
