use regex::Regex;

pub struct CssParser {
    content: String,
}

impl CssParser {
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
        }
    }

    pub fn set_property(&mut self, selector: &str, prop: &str, value: &str) {
        let escaped_selector = regex::escape(selector);
        
        let selector_re = Regex::new(&format!(r"(?m)^[\t ]*({})\s*\{{", escaped_selector)).unwrap();

        if let Some(mat) = selector_re.find(&self.content.clone()) {
            let match_start = mat.start(); 
            let _ = match_start; 
            let block_start_idx = mat.end();
            
            
            if let Some(block_end_idx) = self.find_closing_brace(block_start_idx) {
                let block_content = &self.content[block_start_idx..block_end_idx];
                
                
                let new_block_content = self.update_property_in_block(block_content, prop, value);
                
                
                self.content.replace_range(block_start_idx..block_end_idx, &new_block_content);
            }
        } else {
            
            self.append_new_block(selector, prop, value);
        }
    }

    fn find_closing_brace(&self, start_offset: usize) -> Option<usize> {
        let mut balance = 1;
        for (i, c) in self.content[start_offset..].char_indices() {
            if c == '{' {
                balance += 1;
            } else if c == '}' {
                balance -= 1;
                if balance == 0 {
                    return Some(start_offset + i);
                }
            }
        }
        None
    }

    fn update_property_in_block(&self, block_content: &str, prop: &str, value: &str) -> String {
        let escaped_prop = regex::escape(prop);
        let prop_re = Regex::new(&format!(r"(?m)(^\s*{}\s*:\s*)([^;]+)(;)", escaped_prop)).unwrap();

        if prop_re.is_match(block_content) {
            
            
            prop_re.replace(block_content, format!("${{1}}{}${{3}}", value)).to_string()
        } else {
            
            let trimmed = block_content.trim_end();
            if trimmed.is_empty() {
                format!("\n    {}: {};\n", prop, value)
            } else {
                 format!("{}\n    {}: {};\n", trimmed, prop, value)
            }
        }
    }

    fn append_new_block(&mut self, selector: &str, prop: &str, value: &str) {
        let new_block = format!("\n\n{} {{\n    {}: {};\n}}", selector, prop, value);
        self.content.push_str(&new_block);
    }

    pub fn get_property(&self, selector: &str, prop: &str) -> Option<String> {
        let escaped_selector = regex::escape(selector);
        
        let selector_re = Regex::new(&format!(r"(?m)^[\t ]*({})\s*\{{", escaped_selector)).unwrap();

        if let Some(mat) = selector_re.find(&self.content) {
            let block_start_idx = mat.end();
            if let Some(block_end_idx) = self.find_closing_brace(block_start_idx) {
                let block_content = &self.content[block_start_idx..block_end_idx];
                let escaped_prop = regex::escape(prop);
                let prop_re = Regex::new(&format!(r"(?m)^\s*{}\s*:\s*([^;]+);", escaped_prop)).unwrap();
                
                if let Some(cap) = prop_re.captures(block_content) {
                    return Some(cap[1].trim().to_string());
                }
            }
        }
        None
    }

    pub fn to_string(&self) -> String {
        self.content.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_existing_property() {
        let css = "#waybar {\n    background: #000000;\n    color: #ffffff;\n}";
        let mut parser = CssParser::new(css);
        parser.set_property("#waybar", "background", "#ff0000");
        let output = parser.to_string();
        assert!(output.contains("background: #ff0000;"));
        assert!(output.contains("color: #ffffff;"));
    }

    #[test]
    fn test_add_new_property() {
        let css = "#waybar {\n    background: #000000;\n}";
        let mut parser = CssParser::new(css);
        parser.set_property("#waybar", "color", "#ffffff");
        let output = parser.to_string();
        assert!(output.contains("background: #000000;"));
        assert!(output.contains("color: #ffffff;"));
    }

    #[test]
    fn test_create_new_block() {
        let css = "#waybar {}";
        let mut parser = CssParser::new(css);
        parser.set_property("#clock", "color", "red");
        let output = parser.to_string();
        assert!(output.contains("#clock {"));
        assert!(output.contains("color: red;"));
    }
    
    #[test]
    fn test_empty_file() {
         let mut parser = CssParser::new("");
         parser.set_property("#waybar", "font-size", "12px");
         let output = parser.to_string();
         assert!(output.contains("#waybar {"));
         assert!(output.contains("font-size: 12px;"));
    }
}
