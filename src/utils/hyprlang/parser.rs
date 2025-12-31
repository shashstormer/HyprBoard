use super::Lexer;
use super::Result;
use super::ast::{HyprCategory, HyprConf, HyprLine, HyprValue, HyprValuePart};
use super::token::{Token, TokenType};
use glob::glob;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub fn parse(
    tokens: Vec<Token>,
    base_dir: PathBuf,
    parsed_files: HashSet<String>,
) -> Result<HyprConf> {
    let mut parser = Parser {
        tokens,
        pos: 0,
        conditionals: Vec::new(),
        base_dir,
        parsed_files,
        variables: HashMap::new(),
    };
    parser.parse_block()
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    conditionals: Vec<bool>,
    base_dir: PathBuf,
    parsed_files: HashSet<String>,
    variables: HashMap<String, HyprValue>,
}

impl Parser {
    fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len() || self.tokens[self.pos].kind == TokenType::Eof
    }

    fn advance(&mut self) -> Token {
        if self.pos < self.tokens.len() {
            let t = self.tokens[self.pos].clone();
            if t.kind != TokenType::Eof {
                self.pos += 1;
            }
            return t;
        }
        Token::new(TokenType::Eof, "", 0, 0, 0)
    }

    fn skip_newlines(&mut self) {
        while !self.is_eof() && self.tokens[self.pos].kind == TokenType::Newline {
            self.pos += 1;
        }
    }

    fn skip_to_newline(&mut self) {
        while !self.is_eof() && self.tokens[self.pos].kind != TokenType::Newline {
            self.pos += 1;
        }
    }

    fn parse_block(&mut self) -> Result<HyprConf> {
        let mut lines = Vec::new();
        let mut categories = Vec::new();

        while !self.is_eof() {
            self.skip_newlines();
            if self.is_eof() {
                break;
            }

            if self.tokens[self.pos].kind == TokenType::RBrace {
                break;
            }

            if let Some(&false) = self.conditionals.last() {
                self.skip_conditional_block();
                self.conditionals.pop();
                continue;
            }

            let token = self.tokens[self.pos].clone();

            match token.kind {
                TokenType::Comment => {
                    self.pos += 1;
                }
                TokenType::Directive => {
                    self.handle_directive(&token.value);
                    self.pos += 1;
                }
                TokenType::Variable => {
                    self.parse_variable()?;
                }
                TokenType::Ident => {
                    self.parse_assignment_or_category(&mut lines, &mut categories)?;
                }
                _ => {
                    self.pos += 1;
                }
            }
        }

        Ok(HyprConf {
            variables: self.variables.clone(),
            lines,
            categories,
        })
    }

    fn parse_variable(&mut self) -> Result<()> {
        let var_token = self.advance();
        self.skip_newlines();

        if self.is_eof() || self.tokens[self.pos].kind != TokenType::Equals {
            return Ok(());
        }
        self.advance();

        let value = self.parse_value();
        self.variables.insert(var_token.value, value);
        Ok(())
    }

    fn parse_assignment_or_category(
        &mut self,
        lines: &mut Vec<HyprLine>,
        categories: &mut Vec<HyprCategory>,
    ) -> Result<()> {
        let mut path_parts = Vec::new();

        loop {
            if self.is_eof() {
                break;
            }
            if self.tokens[self.pos].kind == TokenType::Ident {
                let name = self.advance().value;
                let mut key = None;

                if !self.is_eof() && self.tokens[self.pos].kind == TokenType::LBracket {
                    self.advance();
                    if !self.is_eof() {
                        let k = &self.tokens[self.pos];
                        if k.kind == TokenType::Ident
                            || k.kind == TokenType::String
                            || k.kind == TokenType::Number
                        {
                            key = Some(k.value.clone());
                            self.advance();
                        }
                    }
                    if !self.is_eof() && self.tokens[self.pos].kind == TokenType::RBracket {
                        self.advance();
                    }
                }
                path_parts.push((name, key));

                if !self.is_eof() && self.tokens[self.pos].kind == TokenType::Colon {
                    self.advance();
                    continue;
                }
            }
            break;
        }

        if path_parts.is_empty() {
            self.skip_to_newline();
            return Ok(());
        }

        self.skip_newlines();
        if self.is_eof() {
            return Ok(());
        }

        if self.tokens[self.pos].kind == TokenType::LBrace {
            self.parse_category_block(path_parts, categories)?;
            return Ok(());
        }

        if self.tokens[self.pos].kind == TokenType::Equals {
            self.advance();
            let value = self.parse_value();

            let (final_name, _) = path_parts.last().unwrap().clone();
            if final_name == "source" && path_parts.len() == 1 {
                let sourced = self.handle_source(&value.raw)?;
                self.variables.extend(sourced.variables);
            }

            let (key_name, _) = path_parts.pop().unwrap();

            if path_parts.is_empty() {
                lines.push(HyprLine {
                    key: key_name,
                    value,
                    is_variable: false,
                });
            } else {
                Self::insert_line(
                    categories,
                    path_parts,
                    HyprLine {
                        key: key_name,
                        value,
                        is_variable: false,
                    },
                );
            }
            return Ok(());
        }

        self.skip_to_newline();
        Ok(())
    }

    fn insert_line(
        categories: &mut Vec<HyprCategory>,
        mut path: Vec<(String, Option<String>)>,
        line: HyprLine,
    ) {
        if path.is_empty() {
            return;
        }

        let (name, key) = path.remove(0);
        let mut idx = None;
        for (i, cat) in categories.iter().enumerate() {
            if cat.name == name && cat.key == key {
                idx = Some(i);
                break;
            }
        }

        if idx.is_none() {
            categories.push(HyprCategory::new(name.clone(), key.clone()));
            idx = Some(categories.len() - 1);
        }

        let cat = &mut categories[idx.unwrap()];

        if path.is_empty() {
            cat.lines.push(line);
        } else {
            Self::insert_line(&mut cat.categories, path, line);
        }
    }

    fn parse_category_block(
        &mut self,
        path_parts: Vec<(String, Option<String>)>,
        categories: &mut Vec<HyprCategory>,
    ) -> Result<()> {
        self.advance();

        let inner_conf = self.parse_block()?;

        if !self.is_eof() && self.tokens[self.pos].kind == TokenType::RBrace {
            self.advance();
        }

        Self::insert_category_content(categories, path_parts, inner_conf);
        Ok(())
    }

    fn insert_category_content(
        categories: &mut Vec<HyprCategory>,
        mut path: Vec<(String, Option<String>)>,
        content: HyprConf,
    ) {
        if path.is_empty() {
            return;
        }

        let (name, key) = path.remove(0);
        let mut idx = None;
        for (i, cat) in categories.iter().enumerate() {
            if cat.name == name && cat.key == key {
                idx = Some(i);
                break;
            }
        }

        if idx.is_none() {
            categories.push(HyprCategory::new(name.clone(), key.clone()));
            idx = Some(categories.len() - 1);
        }

        let cat = &mut categories[idx.unwrap()];

        if path.is_empty() {
            cat.lines.extend(content.lines);
            cat.categories.extend(content.categories);
        } else {
            Self::insert_category_content(&mut cat.categories, path, content);
        }
    }

    fn parse_value(&mut self) -> HyprValue {
        let mut parts = Vec::new();
        let mut raw = String::new();
        let mut last_was_space_needing = false;

        loop {
            if self.is_eof() {
                break;
            }
            let t = self.tokens[self.pos].clone();
            if t.kind == TokenType::Newline
                || t.kind == TokenType::Eof
                || t.kind == TokenType::RBrace
                || t.kind == TokenType::Comment
            {
                break;
            }

            let is_punctuation = matches!(
                t.kind,
                TokenType::Comma | TokenType::Colon | TokenType::LBracket | TokenType::RBracket
            );

            if !raw.is_empty() && last_was_space_needing && !is_punctuation {
                raw.push(' ');
            }

            match t.kind {
                TokenType::Variable => {
                    parts.push(HyprValuePart::VarRef(t.value.clone()));
                    raw.push_str(&format!("${}", t.value));
                    last_was_space_needing = true;
                    self.pos += 1;
                }
                TokenType::Arithmetic => {
                    parts.push(HyprValuePart::Arithmetic(t.value.clone()));
                    raw.push_str(&format!("{{{{{}}}}}", t.value));
                    last_was_space_needing = true;
                    self.pos += 1;
                }
                TokenType::Comma => {
                    parts.push(HyprValuePart::Literal(t.value.clone()));
                    raw.push(',');
                    raw.push(' ');
                    last_was_space_needing = false;
                    self.pos += 1;
                }
                TokenType::Colon => {
                    parts.push(HyprValuePart::Literal(t.value.clone()));
                    raw.push(':');
                    last_was_space_needing = false;
                    self.pos += 1;
                }
                TokenType::LBracket | TokenType::RBracket => {
                    parts.push(HyprValuePart::Literal(t.value.clone()));
                    raw.push_str(&t.value);
                    last_was_space_needing = false;
                    self.pos += 1;
                }
                TokenType::Ident | TokenType::String | TokenType::Number | TokenType::Equals => {
                    parts.push(HyprValuePart::Literal(t.value.clone()));
                    raw.push_str(&t.value);
                    last_was_space_needing = true;
                    self.pos += 1;
                }
                _ => {
                    break;
                }
            }
        }

        HyprValue::new(raw, parts)
    }

    fn handle_directive(&mut self, val: &str) {
        let parts: Vec<&str> = val.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        match parts[0] {
            "if" => {
                let var_name = parts.get(1).unwrap_or(&"");
                let negate = var_name.starts_with('!');
                let name_without_bang = if negate { &var_name[1..] } else { var_name };
                let actual_name = name_without_bang.trim_start_matches('$');

                let val = self.variables.get(actual_name);
                let is_true = if let Some(v) = val {
                    !v.raw.is_empty()
                } else {
                    std::env::var(actual_name)
                        .map(|v| !v.is_empty())
                        .unwrap_or(false)
                };

                self.conditionals
                    .push(if negate { !is_true } else { is_true });
            }
            "endif" => {
                self.conditionals.pop();
            }
            _ => {}
        }
    }

    fn skip_conditional_block(&mut self) {
        let mut depth = 1;
        while !self.is_eof() && depth > 0 {
            let t = &self.tokens[self.pos];
            if t.kind == TokenType::Directive {
                if t.value.trim().starts_with("if") {
                    depth += 1;
                } else if t.value.trim() == "endif" {
                    depth -= 1;
                }
            }
            self.pos += 1;
        }
    }

    fn handle_source(&mut self, path_pattern: &str) -> Result<HyprConf> {
        let mut result = HyprConf::new();

        let path_pattern = if path_pattern.starts_with("~") {
            if let Some(home) = std::env::var_os("HOME") {
                let mut p = PathBuf::from(home);
                p.push(&path_pattern[2..]);
                p.to_string_lossy().to_string()
            } else {
                path_pattern.to_string()
            }
        } else {
            path_pattern.to_string()
        };

        let current_dir = self.base_dir.clone();

        let pattern_path = Path::new(&path_pattern);
        let full_pattern = if pattern_path.is_absolute() {
            path_pattern.clone()
        } else {
            current_dir.join(path_pattern).to_string_lossy().to_string()
        };

        if let Ok(paths) = glob(&full_pattern) {
            for entry in paths {
                if let Ok(path) = entry {
                    if path.is_dir() {
                        continue;
                    }
                    let abs_path = match std::fs::canonicalize(&path) {
                        Ok(p) => p.to_string_lossy().to_string(),
                        Err(_) => continue,
                    };

                    if self.parsed_files.contains(&abs_path) {
                        continue;
                    }

                    if let Ok(content) = fs::read_to_string(&path) {
                        let mut new_parsed = self.parsed_files.clone();
                        new_parsed.insert(abs_path.clone());

                        let mut lexer = Lexer::new(&content);
                        let tokens = lexer.tokenize();

                        let mut sub_parser = Parser {
                            tokens,
                            pos: 0,
                            conditionals: Vec::new(),
                            base_dir: path.parent().unwrap_or(Path::new(".")).to_path_buf(),
                            parsed_files: new_parsed,
                            variables: self.variables.clone(),
                        };

                        let sub_conf = sub_parser.parse_block()?;

                        result.variables.extend(sub_conf.variables);
                        result.lines.extend(sub_conf.lines);
                        result.categories.extend(sub_conf.categories);
                    }
                }
            }
        }
        Ok(result)
    }
}
