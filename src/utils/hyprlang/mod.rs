use thiserror::Error;

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod tests;
pub mod token;

#[derive(Error, Debug)]
pub enum HyprError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse Error: {0}")]
    Parse(String),
}

type Result<T> = std::result::Result<T, HyprError>;

pub use ast::HyprConf;
use lexer::Lexer;
use std::collections::HashSet;
use std::path::Path;

pub struct HyprLang {
    file_path: Option<String>,
}

impl HyprLang {
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: Some(file_path.into()),
        }
    }

    pub fn parse(&self, content: &str) -> Result<HyprConf> {
        let base_dir = if let Some(p) = &self.file_path {
            Path::new(p)
                .parent()
                .unwrap_or(Path::new("."))
                .to_path_buf()
        } else {
            std::env::current_dir()?
        };

        let mut lexer = Lexer::new(content);
        let tokens = lexer.tokenize();

        let mut parsed_files = HashSet::new();
        if let Some(p) = &self.file_path {
            if let Ok(canon) = std::fs::canonicalize(p) {
                parsed_files.insert(canon.to_string_lossy().to_string());
            }
        }

        parser::parse(tokens, base_dir, parsed_files)
    }

    pub fn load(&self) -> Result<HyprConf> {
        match &self.file_path {
            Some(path) => {
                let content = std::fs::read_to_string(path)?;
                self.parse(&content)
            }
            None => Err(HyprError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No file path provided",
            ))),
        }
    }

    pub fn save(&self, conf: &HyprConf) -> Result<()> {
        if let Some(path) = &self.file_path {
            let content = conf.to_string();
            std::fs::write(path, content)?;
            Ok(())
        } else {
            Err(HyprError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No file path provided",
            )))
        }
    }
}
