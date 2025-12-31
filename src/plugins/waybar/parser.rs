use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Whitespace,
    Comment,
    String,
    Number,
    True,
    False,
    Null,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Colon,
    Comma,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenType,
    pub value: String,
    pub line: usize,
    pub col: usize,
    pub pos: usize,
}

#[derive(Debug, Clone)]
pub enum Node {
    Value(ValueNode),
    Dict(DictNode),
    List(ListNode),
}

#[derive(Debug, Clone)]
pub struct ValueNode {
    pub value: serde_json::Value,
    pub raw_text: String,
    pub leading_trivia: Vec<Token>,
    pub trailing_trivia: Vec<Token>,
}

#[derive(Debug, Clone)]
pub struct DictNode {
    pub children: Vec<(KeyNode, Node, Option<Token>)>,
    pub leading_trivia: Vec<Token>,
    pub trailing_trivia: Vec<Token>,
    pub internal_trailing_trivia: Vec<Token>,
}

#[derive(Debug, Clone)]
pub struct ListNode {
    pub children: Vec<(Node, Option<Token>)>,
    pub leading_trivia: Vec<Token>,
    pub trailing_trivia: Vec<Token>,
    pub internal_trailing_trivia: Vec<Token>,
}

#[derive(Debug, Clone)]
pub struct KeyNode {
    pub value: String,
    pub raw_text: String,
    pub leading_trivia: Vec<Token>,
    pub trailing_trivia: Vec<Token>,
}

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_n(&self, n: usize) -> Option<char> {
        self.chars.get(self.pos + n).copied()
    }

    fn advance(&mut self) -> Option<char> {
        if let Some(&c) = self.chars.get(self.pos) {
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            Some(c)
        } else {
            None
        }
    }

    fn advance_n(&mut self, n: usize) {
        for _ in 0..n {
            self.advance();
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        while self.pos < self.chars.len() {
            let start_pos = self.pos;
            let start_line = self.line;
            let start_col = self.col;
            let c = self.peek().unwrap();

            if c.is_whitespace() {
                let mut value = String::new();
                while let Some(current) = self.peek() {
                    if current.is_whitespace() {
                        value.push(current);
                        self.advance();
                    } else {
                        break;
                    }
                }
                tokens.push(Token {
                    kind: TokenType::Whitespace,
                    value,
                    line: start_line,
                    col: start_col,
                    pos: start_pos,
                });
                continue;
            }

            if c == '/' {
                if let Some(next) = self.peek_n(1) {
                    if next == '/' {
                        let mut value = String::new();
                        while let Some(current) = self.peek() {
                            value.push(current);
                            self.advance();
                            if current == '\n' {
                                break;
                            }
                        }
                        tokens.push(Token {
                            kind: TokenType::Comment,
                            value,
                            line: start_line,
                            col: start_col,
                            pos: start_pos,
                        });
                        continue;
                    } else if next == '*' {
                        let mut value = String::new();
                        value.push(c);
                        value.push(next);
                        self.advance_n(2);
                        while let Some(current) = self.peek() {
                            value.push(current);
                            self.advance();
                            if current == '/' && value.ends_with("*/") {
                                break;
                            }
                        }
                        tokens.push(Token {
                            kind: TokenType::Comment,
                            value,
                            line: start_line,
                            col: start_col,
                            pos: start_pos,
                        });
                        continue;
                    }
                }
            }

            if c == '"' {
                let mut value = String::new();
                value.push(c);
                self.advance();
                let mut escaped = false;
                while let Some(current) = self.peek() {
                    value.push(current);
                    self.advance();
                    if escaped {
                        escaped = false;
                    } else if current == '\\' {
                        escaped = true;
                    } else if current == '"' {
                        break;
                    }
                }
                tokens.push(Token {
                    kind: TokenType::String,
                    value,
                    line: start_line,
                    col: start_col,
                    pos: start_pos,
                });
                continue;
            }

            if c.is_digit(10) || c == '-' {
                let mut value = String::new();
                while let Some(current) = self.peek() {
                    if current.is_digit(10)
                        || current == '.'
                        || current == '-'
                        || current == '+'
                        || current == 'e'
                        || current == 'E'
                    {
                        value.push(current);
                        self.advance();
                    } else {
                        break;
                    }
                }
                tokens.push(Token {
                    kind: TokenType::Number,
                    value,
                    line: start_line,
                    col: start_col,
                    pos: start_pos,
                });
                continue;
            }

            match c {
                '{' => {
                    self.advance();
                    tokens.push(Token {
                        kind: TokenType::LBrace,
                        value: "{".into(),
                        line: start_line,
                        col: start_col,
                        pos: start_pos,
                    });
                }
                '}' => {
                    self.advance();
                    tokens.push(Token {
                        kind: TokenType::RBrace,
                        value: "}".into(),
                        line: start_line,
                        col: start_col,
                        pos: start_pos,
                    });
                }
                '[' => {
                    self.advance();
                    tokens.push(Token {
                        kind: TokenType::LBracket,
                        value: "[".into(),
                        line: start_line,
                        col: start_col,
                        pos: start_pos,
                    });
                }
                ']' => {
                    self.advance();
                    tokens.push(Token {
                        kind: TokenType::RBracket,
                        value: "]".into(),
                        line: start_line,
                        col: start_col,
                        pos: start_pos,
                    });
                }
                ':' => {
                    self.advance();
                    tokens.push(Token {
                        kind: TokenType::Colon,
                        value: ":".into(),
                        line: start_line,
                        col: start_col,
                        pos: start_pos,
                    });
                }
                ',' => {
                    self.advance();
                    tokens.push(Token {
                        kind: TokenType::Comma,
                        value: ",".into(),
                        line: start_line,
                        col: start_col,
                        pos: start_pos,
                    });
                }
                _ => {
                    let mut value = String::new();
                    while let Some(current) = self.peek() {
                        if current.is_alphabetic() {
                            value.push(current);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    if value == "true" {
                        tokens.push(Token {
                            kind: TokenType::True,
                            value,
                            line: start_line,
                            col: start_col,
                            pos: start_pos,
                        });
                    } else if value == "false" {
                        tokens.push(Token {
                            kind: TokenType::False,
                            value,
                            line: start_line,
                            col: start_col,
                            pos: start_pos,
                        });
                    } else if value == "null" {
                        tokens.push(Token {
                            kind: TokenType::Null,
                            value,
                            line: start_line,
                            col: start_col,
                            pos: start_pos,
                        });
                    } else {
                        return Err(format!(
                            "Unexpected character '{}' at line {}, col {}",
                            c, start_line, start_col
                        ));
                    }
                }
            }
        }
        tokens.push(Token {
            kind: TokenType::Eof,
            value: "".into(),
            line: self.line,
            col: self.col,
            pos: self.pos,
        });
        Ok(tokens)
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap()
    }

    fn consume(&mut self) -> Token {
        let t = self.tokens.get(self.pos).unwrap().clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        t
    }

    fn collect_trivia(&mut self) -> Vec<Token> {
        let mut trivia = Vec::new();
        while matches!(self.peek().kind, TokenType::Whitespace | TokenType::Comment) {
            trivia.push(self.consume());
        }
        trivia
    }

    pub fn parse(&mut self) -> Result<Node, String> {
        let leading = self.collect_trivia();
        if self.peek().kind == TokenType::Eof {
            return Err("Unexpected EOF".into());
        }
        let mut root = self.parse_element()?;

        match &mut root {
            Node::Value(n) => {
                n.leading_trivia.splice(0..0, leading);
                n.trailing_trivia = self.collect_trivia();
            }
            Node::Dict(n) => {
                n.leading_trivia.splice(0..0, leading);
                n.trailing_trivia = self.collect_trivia();
            }
            Node::List(n) => {
                n.leading_trivia.splice(0..0, leading);
                n.trailing_trivia = self.collect_trivia();
            }
        }

        Ok(root)
    }

    fn parse_element(&mut self) -> Result<Node, String> {
        let trivia = self.collect_trivia();
        let token = self.peek().clone();

        let mut node = match token.kind {
            TokenType::LBrace => Node::Dict(self.parse_object()?),
            TokenType::LBracket => Node::List(self.parse_array()?),
            TokenType::String => {
                self.consume();
                let s = token.value[1..token.value.len() - 1].to_string();
                Node::Value(ValueNode {
                    value: serde_json::Value::String(s),
                    raw_text: token.value,
                    leading_trivia: vec![],
                    trailing_trivia: vec![],
                })
            }
            TokenType::Number => {
                self.consume();
                let v = if token.value.contains('.') || token.value.contains('e') {
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(token.value.parse().unwrap_or(0.0)).unwrap(),
                    )
                } else {
                    serde_json::Value::Number(token.value.parse::<i64>().unwrap_or(0).into())
                };
                Node::Value(ValueNode {
                    value: v,
                    raw_text: token.value,
                    leading_trivia: vec![],
                    trailing_trivia: vec![],
                })
            }
            TokenType::True => {
                self.consume();
                Node::Value(ValueNode {
                    value: serde_json::Value::Bool(true),
                    raw_text: token.value,
                    leading_trivia: vec![],
                    trailing_trivia: vec![],
                })
            }
            TokenType::False => {
                self.consume();
                Node::Value(ValueNode {
                    value: serde_json::Value::Bool(false),
                    raw_text: token.value,
                    leading_trivia: vec![],
                    trailing_trivia: vec![],
                })
            }
            TokenType::Null => {
                self.consume();
                Node::Value(ValueNode {
                    value: serde_json::Value::Null,
                    raw_text: token.value,
                    leading_trivia: vec![],
                    trailing_trivia: vec![],
                })
            }
            _ => {
                return Err(format!(
                    "Unexpected token {:?} at line {}",
                    token.kind, token.line
                ));
            }
        };

        match &mut node {
            Node::Value(n) => n.leading_trivia = trivia,
            Node::Dict(n) => n.leading_trivia = trivia,
            Node::List(n) => n.leading_trivia = trivia,
        }

        Ok(node)
    }

    fn parse_object(&mut self) -> Result<DictNode, String> {
        self.consume();
        let mut children = Vec::new();
        let internal_trivia;

        loop {
            let trivia = self.collect_trivia();
            if self.peek().kind == TokenType::RBrace {
                internal_trivia = trivia;
                break;
            }

            if self.peek().kind != TokenType::String {
                return Err(format!("Expected string key at line {}", self.peek().line));
            }

            let key_token = self.consume();
            let key_val = key_token.value[1..key_token.value.len() - 1].to_string();

            let middle_trivia = self.collect_trivia();
            let key_node = KeyNode {
                value: key_val,
                raw_text: key_token.value,
                leading_trivia: trivia,
                trailing_trivia: middle_trivia,
            };

            if self.peek().kind != TokenType::Colon {
                return Err(format!("Expected ':' at line {}", self.peek().line));
            }
            self.consume();

            let mut val_node = self.parse_element()?;

            let comma_trivia = self.collect_trivia();
            let mut comma = None;
            if self.peek().kind == TokenType::Comma {
                comma = Some(self.consume());
            }

            match &mut val_node {
                Node::Value(n) => n.trailing_trivia = comma_trivia,
                Node::Dict(n) => n.trailing_trivia = comma_trivia,
                Node::List(n) => n.trailing_trivia = comma_trivia,
            }

            children.push((key_node, val_node, comma));

            if children.last().unwrap().2.is_none() {
                let _ = self.collect_trivia();
                if self.peek().kind != TokenType::RBrace {
                    return Err(format!("Expected ',' or '}}' at line {}", self.peek().line));
                }
            }
        }

        self.consume();
        Ok(DictNode {
            children,
            leading_trivia: vec![],
            trailing_trivia: vec![],
            internal_trailing_trivia: internal_trivia,
        })
    }

    fn parse_array(&mut self) -> Result<ListNode, String> {
        self.consume();
        let mut children = Vec::new();
        let internal_trivia;

        loop {
            let trivia = self.collect_trivia();
            if self.peek().kind == TokenType::RBracket {
                internal_trivia = trivia;
                break;
            }

            let mut val_node = self.parse_element()?;
            match &mut val_node {
                Node::Value(n) => n.leading_trivia.splice(0..0, trivia),
                Node::Dict(n) => n.leading_trivia.splice(0..0, trivia),
                Node::List(n) => n.leading_trivia.splice(0..0, trivia),
            };

            let comma_trivia = self.collect_trivia();
            let mut comma = None;
            if self.peek().kind == TokenType::Comma {
                comma = Some(self.consume());
            }

            match &mut val_node {
                Node::Value(n) => n.trailing_trivia = comma_trivia,
                Node::Dict(n) => n.trailing_trivia = comma_trivia,
                Node::List(n) => n.trailing_trivia = comma_trivia,
            }

            children.push((val_node, comma));

            if children.last().unwrap().1.is_none() {
                let _ = self.collect_trivia();
                if self.peek().kind != TokenType::RBracket {
                    return Err(format!("Expected ',' or ']' at line {}", self.peek().line));
                }
            }
        }
        self.consume();
        Ok(ListNode {
            children,
            leading_trivia: vec![],
            trailing_trivia: vec![],
            internal_trailing_trivia: internal_trivia,
        })
    }
}

pub struct Printer {
    output: String,
}

impl Printer {
    pub fn new() -> Self {
        Self {
            output: String::new(),
        }
    }

    fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn print_trivia(&mut self, trivia: &[Token]) {
        for t in trivia {
            self.write(&t.value);
        }
    }

    pub fn print_node(&mut self, node: &Node) {
        match node {
            Node::Value(n) => {
                self.print_trivia(&n.leading_trivia);
                self.write(&n.raw_text);
                self.print_trivia(&n.trailing_trivia);
            }
            Node::Dict(n) => {
                self.print_trivia(&n.leading_trivia);
                self.write("{");
                for (key, val, comma) in &n.children {
                    self.print_trivia(&key.leading_trivia);
                    self.write(&key.raw_text);
                    self.print_trivia(&key.trailing_trivia);
                    self.write(":");
                    self.print_node(val);
                    if let Some(c) = comma {
                        self.write(&c.value);
                    }
                }
                self.print_trivia(&n.internal_trailing_trivia);
                self.write("}");
                self.print_trivia(&n.trailing_trivia);
            }
            Node::List(n) => {
                self.print_trivia(&n.leading_trivia);
                self.write("[");
                for (val, comma) in &n.children {
                    self.print_node(val);
                    if let Some(c) = comma {
                        self.write(&c.value);
                    }
                }
                self.print_trivia(&n.internal_trailing_trivia);
                self.write("]");
                self.print_trivia(&n.trailing_trivia);
            }
        }
    }
}

pub fn parse(input: &str) -> Result<Node, String> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

pub fn to_string(node: &Node) -> String {
    let mut printer = Printer::new();
    printer.print_node(node);
    printer.output
}

pub fn to_json_value(node: &Node) -> serde_json::Value {
    match node {
        Node::Value(n) => n.value.clone(),
        Node::Dict(n) => {
            let mut map = serde_json::Map::new();
            for (k, v, _) in &n.children {
                map.insert(k.value.clone(), to_json_value(v));
            }
            serde_json::Value::Object(map)
        }
        Node::List(n) => {
            let mut vec = Vec::new();
            for (v, _) in &n.children {
                vec.push(to_json_value(v));
            }
            serde_json::Value::Array(vec)
        }
    }
}

pub fn detect_indent(node: &Node) -> String {
    let mut counts: HashMap<String, usize> = HashMap::new();

    fn walk(node: &Node, counts: &mut HashMap<String, usize>) {
        match node {
            Node::Value(n) => {
                for t in &n.leading_trivia {
                    if t.kind == TokenType::Whitespace && t.value.contains('\n') {
                        if let Some(last_line) = t.value.split('\n').last() {
                            if !last_line.is_empty() {
                                *counts.entry(last_line.to_string()).or_insert(0) += 1;
                            }
                        }
                    }
                }
            }
            Node::Dict(n) => {
                for t in &n.leading_trivia {
                    if t.kind == TokenType::Whitespace && t.value.contains('\n') {
                        if let Some(last_line) = t.value.split('\n').last() {
                            if !last_line.is_empty() {
                                *counts.entry(last_line.to_string()).or_insert(0) += 1;
                            }
                        }
                    }
                }
                for (k, v, _) in &n.children {
                    for t in &k.leading_trivia {
                        if t.kind == TokenType::Whitespace && t.value.contains('\n') {
                            if let Some(last_line) = t.value.split('\n').last() {
                                if !last_line.is_empty() {
                                    *counts.entry(last_line.to_string()).or_insert(0) += 1;
                                }
                            }
                        }
                    }
                    walk(v, counts);
                }
            }
            Node::List(n) => {
                for t in &n.leading_trivia {
                    if t.kind == TokenType::Whitespace && t.value.contains('\n') {
                        if let Some(last_line) = t.value.split('\n').last() {
                            if !last_line.is_empty() {
                                *counts.entry(last_line.to_string()).or_insert(0) += 1;
                            }
                        }
                    }
                }
                for (v, _) in &n.children {
                    walk(v, counts);
                }
            }
        }
    }

    walk(node, &mut counts);

    if counts.is_empty() {
        "    ".to_string()
    } else {
        counts.into_iter().max_by_key(|(_, v)| *v).unwrap().0
    }
}

pub fn create_node_from_value(v: &serde_json::Value, indent: &str, level: usize) -> Node {
    let current_indent = format!("\n{}", indent.repeat(level));
    let next_indent = format!("\n{}", indent.repeat(level + 1));

    match v {
        serde_json::Value::Null => Node::Value(ValueNode {
            value: v.clone(),
            raw_text: "null".into(),
            leading_trivia: vec![],
            trailing_trivia: vec![],
        }),
        serde_json::Value::Bool(b) => Node::Value(ValueNode {
            value: v.clone(),
            raw_text: b.to_string(),
            leading_trivia: vec![],
            trailing_trivia: vec![],
        }),
        serde_json::Value::Number(n) => Node::Value(ValueNode {
            value: v.clone(),
            raw_text: n.to_string(),
            leading_trivia: vec![],
            trailing_trivia: vec![],
        }),
        serde_json::Value::String(s) => Node::Value(ValueNode {
            value: v.clone(),
            raw_text: serde_json::to_string(s).unwrap(),
            leading_trivia: vec![],
            trailing_trivia: vec![],
        }),
        serde_json::Value::Object(map) => {
            let mut children = Vec::new();
            let items: Vec<_> = map.iter().collect();
            for (i, (k, val)) in items.iter().enumerate() {
                let leading = vec![Token {
                    kind: TokenType::Whitespace,
                    value: next_indent.clone(),
                    line: 0,
                    col: 0,
                    pos: 0,
                }];
                let key_node = KeyNode {
                    value: k.to_string(),
                    raw_text: format!("\"{}\"", k),
                    leading_trivia: leading,
                    trailing_trivia: vec![],
                };

                let mut val_node = create_node_from_value(val, indent, level + 1);
                match &mut val_node {
                    Node::Value(n) => n.leading_trivia.insert(
                        0,
                        Token {
                            kind: TokenType::Whitespace,
                            value: " ".to_string(),
                            line: 0,
                            col: 0,
                            pos: 0,
                        },
                    ),
                    Node::Dict(n) => n.leading_trivia.insert(
                        0,
                        Token {
                            kind: TokenType::Whitespace,
                            value: " ".to_string(),
                            line: 0,
                            col: 0,
                            pos: 0,
                        },
                    ),
                    Node::List(n) => n.leading_trivia.insert(
                        0,
                        Token {
                            kind: TokenType::Whitespace,
                            value: " ".to_string(),
                            line: 0,
                            col: 0,
                            pos: 0,
                        },
                    ),
                }

                let comma = if i < items.len() - 1 {
                    Some(Token {
                        kind: TokenType::Comma,
                        value: ",".into(),
                        line: 0,
                        col: 0,
                        pos: 0,
                    })
                } else {
                    None
                };

                children.push((key_node, val_node, comma));
            }

            let internal_trivia = if !children.is_empty() {
                vec![Token {
                    kind: TokenType::Whitespace,
                    value: current_indent,
                    line: 0,
                    col: 0,
                    pos: 0,
                }]
            } else {
                vec![]
            };

            Node::Dict(DictNode {
                children,
                leading_trivia: vec![],
                trailing_trivia: vec![],
                internal_trailing_trivia: internal_trivia,
            })
        }
        serde_json::Value::Array(arr) => {
            let mut children = Vec::new();
            for (i, val) in arr.iter().enumerate() {
                let mut val_node = create_node_from_value(val, indent, level + 1);
                val_node.set_leading_trivia(vec![Token {
                    kind: TokenType::Whitespace,
                    value: next_indent.clone(),
                    line: 0,
                    col: 0,
                    pos: 0,
                }]);

                let comma = if i < arr.len() - 1 {
                    Some(Token {
                        kind: TokenType::Comma,
                        value: ",".into(),
                        line: 0,
                        col: 0,
                        pos: 0,
                    })
                } else {
                    None
                };
                children.push((val_node, comma));
            }

            let internal_trivia = if !children.is_empty() {
                vec![Token {
                    kind: TokenType::Whitespace,
                    value: current_indent,
                    line: 0,
                    col: 0,
                    pos: 0,
                }]
            } else {
                vec![]
            };

            Node::List(ListNode {
                children,
                leading_trivia: vec![],
                trailing_trivia: vec![],
                internal_trailing_trivia: internal_trivia,
            })
        }
    }
}

impl Node {
    pub fn set_leading_trivia(&mut self, trivia: Vec<Token>) {
        match self {
            Node::Value(n) => n.leading_trivia = trivia,
            Node::Dict(n) => n.leading_trivia = trivia,
            Node::List(n) => n.leading_trivia = trivia,
        }
    }
    pub fn get_leading_trivia(&self) -> Vec<Token> {
        match self {
            Node::Value(n) => n.leading_trivia.clone(),
            Node::Dict(n) => n.leading_trivia.clone(),
            Node::List(n) => n.leading_trivia.clone(),
        }
    }
    pub fn get_trailing_trivia(&self) -> Vec<Token> {
        match self {
            Node::Value(n) => n.trailing_trivia.clone(),
            Node::Dict(n) => n.trailing_trivia.clone(),
            Node::List(n) => n.trailing_trivia.clone(),
        }
    }
}

pub fn set_indent(node: &mut Node, indent_str: &str) {
    let t = Token {
        kind: TokenType::Whitespace,
        value: indent_str.to_string(),
        line: 0,
        col: 0,
        pos: 0,
    };
    node.set_leading_trivia(vec![t]);
}

pub fn set_value(root: &mut Node, path: &[&str], value: serde_json::Value) {
    let indent_str = detect_indent(root);
    let current = root;

    fn update_recursive(
        node: &mut Node,
        path: &[&str],
        value: serde_json::Value,
        indent: &str,
        level: usize,
    ) {
        if path.is_empty() {
            return;
        }

        let key = path[0];
        let is_last = path.len() == 1;

        match node {
            Node::Dict(dict) => {
                let mut found_pos = None;
                for (i, (k, _, _)) in dict.children.iter().enumerate() {
                    if k.value == key {
                        found_pos = Some(i);
                        break;
                    }
                }

                if let Some(pos) = found_pos {
                    let (_, val_node, _) = &mut dict.children[pos];
                    if is_last {
                        let mut new_node = create_node_from_value(&value, indent, level + 1);
                        new_node.set_leading_trivia(val_node.get_leading_trivia());
                        match val_node {
                            Node::Value(v) => match &mut new_node {
                                Node::Value(n) => n.trailing_trivia = v.trailing_trivia.clone(),
                                _ => {}
                            },
                            _ => {}
                        }
                        *val_node = new_node;
                    } else {
                        update_recursive(val_node, &path[1..], value, indent, level + 1);
                    }
                } else {
                    if is_last {
                        let new_leading = vec![Token {
                            kind: TokenType::Whitespace,
                            value: format!("\n{}", indent.repeat(level + 1)),
                            line: 0,
                            col: 0,
                            pos: 0,
                        }];
                        let new_key = KeyNode {
                            value: key.to_string(),
                            raw_text: format!("\"{}\"", key),
                            leading_trivia: new_leading,
                            trailing_trivia: vec![],
                        };

                        let mut new_val = create_node_from_value(&value, indent, level + 1);
                        new_val.set_leading_trivia(vec![Token {
                            kind: TokenType::Whitespace,
                            value: " ".to_string(),
                            line: 0,
                            col: 0,
                            pos: 0,
                        }]);

                        if let Some((_, _, comma)) = dict.children.last_mut() {
                            if comma.is_none() {
                                *comma = Some(Token {
                                    kind: TokenType::Comma,
                                    value: ",".into(),
                                    line: 0,
                                    col: 0,
                                    pos: 0,
                                });
                            }
                        }
                        dict.children.push((new_key, new_val, None));
                    } else {
                        let next_key = path[1];
                        let is_next_key_int = next_key.parse::<usize>().is_ok();

                        let mut new_intermediate = if is_next_key_int {
                            Node::List(ListNode {
                                children: vec![],
                                leading_trivia: vec![],
                                trailing_trivia: vec![],
                                internal_trailing_trivia: vec![],
                            })
                        } else {
                            Node::Dict(DictNode {
                                children: vec![],
                                leading_trivia: vec![],
                                trailing_trivia: vec![],
                                internal_trailing_trivia: vec![],
                            })
                        };
                        new_intermediate.set_leading_trivia(vec![Token {
                            kind: TokenType::Whitespace,
                            value: " ".to_string(),
                            line: 0,
                            col: 0,
                            pos: 0,
                        }]);

                        let new_leading = vec![Token {
                            kind: TokenType::Whitespace,
                            value: format!("\n{}", indent.repeat(level + 1)),
                            line: 0,
                            col: 0,
                            pos: 0,
                        }];
                        let new_key = KeyNode {
                            value: key.to_string(),
                            raw_text: format!("\"{}\"", key),
                            leading_trivia: new_leading,
                            trailing_trivia: vec![],
                        };

                        if let Some((_, _, comma)) = dict.children.last_mut() {
                            if comma.is_none() {
                                *comma = Some(Token {
                                    kind: TokenType::Comma,
                                    value: ",".into(),
                                    line: 0,
                                    col: 0,
                                    pos: 0,
                                });
                            }
                        }

                        update_recursive(
                            &mut new_intermediate,
                            &path[1..],
                            value.clone(),
                            indent,
                            level + 1,
                        );

                        dict.children.push((new_key, new_intermediate, None));
                    }
                }
            }
            Node::List(list) => {
                if let Ok(idx) = key.parse::<usize>() {
                    if idx < list.children.len() {
                        let (val_node, _) = &mut list.children[idx];
                        if is_last {
                            let mut new_node = create_node_from_value(&value, indent, level + 1);
                            new_node.set_leading_trivia(val_node.get_leading_trivia());
                            *val_node = new_node;
                        } else {
                            update_recursive(val_node, &path[1..], value, indent, level + 1);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    update_recursive(current, path, value, &indent_str, 0);
}

pub fn get_node_mut<'a>(root: &'a mut Node, path: &[&str]) -> Option<&'a mut Node> {
    let mut current = root;
    for key in path {
        match current {
            Node::Dict(dict) => {
                if let Some(pos) = dict.children.iter().position(|(k, _, _)| k.value == *key) {
                    current = &mut dict.children[pos].1;
                } else {
                    return None;
                }
            }
            Node::List(list) => {
                if let Ok(idx) = key.parse::<usize>() {
                    if idx < list.children.len() {
                        current = &mut list.children[idx].0;
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
            _ => return None,
        }
    }
    Some(current)
}

pub fn remove_from_list_by_value(
    root: &mut Node,
    list_path: &[&str],
    value_to_remove: &str,
) -> Option<Node> {
    let list_node = get_node_mut(root, list_path)?;

    if let Node::List(list) = list_node {
        if let Some(pos) = list.children.iter().position(|(v, _)| match v {
            Node::Value(vn) => {
                vn.raw_text == format!("\"{}\"", value_to_remove)
                    || vn.value == serde_json::Value::String(value_to_remove.to_string())
            }
            _ => false,
        }) {
            let (removed_node, _comma) = list.children.remove(pos);

            return Some(removed_node);
        }
    }
    None
}

pub fn insert_into_list(
    root: &mut Node,
    list_path: &[&str],
    index: usize,
    node: Node,
) -> Result<(), String> {
    let indent_str = detect_indent(root);
    let level = list_path.len() + 1;
    let indent = format!("\n{}", indent_str.repeat(level));

    let list_node = get_node_mut(root, list_path).ok_or("List path not found")?;

    if let Node::List(list) = list_node {
        let mut new_node = node;

        new_node.set_leading_trivia(vec![Token {
            kind: TokenType::Whitespace,
            value: indent.clone(),
            line: 0,
            col: 0,
            pos: 0,
        }]);

        if index > 0 && index <= list.children.len() {
            if let Some((_, comma)) = list.children.get_mut(index - 1) {
                if comma.is_none() {
                    *comma = Some(Token {
                        kind: TokenType::Comma,
                        value: ",".into(),
                        line: 0,
                        col: 0,
                        pos: 0,
                    });
                }
            }
        }

        let comma = if index < list.children.len() {
            Some(Token {
                kind: TokenType::Comma,
                value: ",".into(),
                line: 0,
                col: 0,
                pos: 0,
            })
        } else {
            None
        };

        if index > list.children.len() {
            list.children.push((new_node, comma));
        } else {
            list.children.insert(index, (new_node, comma));
        }

        Ok(())
    } else {
        Err("Path does not point to a List".into())
    }
}

pub fn remove_key(root: &mut Node, path: &[&str]) -> Option<Node> {
    if path.is_empty() {
        return None;
    }

    let key_to_remove = path.last().unwrap();
    let parent_path = &path[..path.len() - 1];

    let parent = if parent_path.is_empty() {
        Some(root)
    } else {
        get_node_mut(root, parent_path)
    }?;

    if let Node::Dict(dict) = parent {
        if let Some(pos) = dict
            .children
            .iter()
            .position(|(k, _, _)| &k.value == key_to_remove)
        {
            let (_, removed_node, _) = dict.children.remove(pos);

            if pos == dict.children.len() && pos > 0 {
                if let Some((_, _, comma)) = dict.children.get_mut(pos - 1) {
                    *comma = None;
                }
            }

            return Some(removed_node);
        }
    }
    None
}
