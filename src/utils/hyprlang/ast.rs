use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum HyprValuePart {
    Literal(String),
    VarRef(String),
    Arithmetic(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct HyprValue {
    pub raw: String,
    pub parts: Vec<HyprValuePart>,
}

impl HyprValue {
    pub fn new(raw: String, parts: Vec<HyprValuePart>) -> Self {
        Self { raw, parts }
    }

    pub fn resolve(&self, variables: &HashMap<String, String>) -> String {
        if self.parts.is_empty() {
            return self.raw.clone();
        }

        let mut result = String::new();
        for (i, part) in self.parts.iter().enumerate() {
            let part_str = match part {
                HyprValuePart::Literal(s) => s.clone(),
                HyprValuePart::VarRef(name) => variables
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| format!("${}", name)),
                HyprValuePart::Arithmetic(expr) => {
                    let mut substituted = expr.clone();
                    for (k, v) in variables {
                        substituted = substituted.replace(&format!("${}", k), v);
                    }

                    if let Some(val) = eval_math(&substituted) {
                        if val.fract() == 0.0 {
                            format!("{}", val as i64)
                        } else {
                            val.to_string()
                        }
                    } else {
                        format!("{{{{{}}}}}", expr)
                    }
                }
            };

            if i > 0 && part_str != "," {
                result.push(' ');
            }
            result.push_str(&part_str);
        }
        result
    }
}

fn eval_math(expr: &str) -> Option<f64> {
    let expr = expr.replace([' ', '\t', '\n', '\r'], "");

    let chars: Vec<char> = expr.chars().collect();
    let mut pos = 0;

    fn parse_e(chars: &[char], pos: &mut usize) -> Option<f64> {
        let mut val = parse_t(chars, pos)?;

        while *pos < chars.len() {
            match chars[*pos] {
                '+' => {
                    *pos += 1;
                    val += parse_t(chars, pos)?;
                }
                '-' => {
                    *pos += 1;
                    val -= parse_t(chars, pos)?;
                }
                _ => break,
            }
        }
        Some(val)
    }

    fn parse_t(chars: &[char], pos: &mut usize) -> Option<f64> {
        let mut val = parse_f(chars, pos)?;

        while *pos < chars.len() {
            match chars[*pos] {
                '*' => {
                    *pos += 1;
                    val *= parse_f(chars, pos)?;
                }
                '/' => {
                    *pos += 1;
                    let divisor = parse_f(chars, pos)?;
                    if divisor == 0.0 {
                        return None;
                    }
                    val /= divisor;
                }
                '%' => {
                    *pos += 1;
                    let divisor = parse_f(chars, pos)?;
                    if divisor == 0.0 {
                        return None;
                    }
                    val %= divisor;
                }
                _ => break,
            }
        }
        Some(val)
    }

    fn parse_f(chars: &[char], pos: &mut usize) -> Option<f64> {
        if *pos >= chars.len() {
            return None;
        }

        if chars[*pos] == '(' {
            *pos += 1;
            let val = parse_e(chars, pos)?;
            if *pos < chars.len() && chars[*pos] == ')' {
                *pos += 1;
                return Some(val);
            }
            return None;
        }

        let start = *pos;
        if chars[*pos] == '-' {
            *pos += 1;
        }
        while *pos < chars.len() && (chars[*pos].is_numeric() || chars[*pos] == '.') {
            *pos += 1;
        }

        if start == *pos {
            return None;
        }
        let num_str: String = chars[start..*pos].iter().collect();
        num_str.parse::<f64>().ok()
    }

    parse_e(&chars, &mut pos)
}

#[derive(Debug, Clone)]
pub struct HyprLine {
    pub key: String,
    pub value: HyprValue,
    pub is_variable: bool,
}

#[derive(Debug, Clone)]
pub struct HyprCategory {
    pub name: String,
    pub key: Option<String>,
    pub lines: Vec<HyprLine>,
    pub categories: Vec<HyprCategory>,
}

impl HyprCategory {
    pub fn new(name: String, key: Option<String>) -> Self {
        Self {
            name,
            key,
            lines: Vec::new(),
            categories: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HyprConf {
    pub variables: HashMap<String, HyprValue>,
    pub lines: Vec<HyprLine>,
    pub categories: Vec<HyprCategory>,
}

impl HyprConf {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            lines: Vec::new(),
            categories: Vec::new(),
        }
    }

    pub fn get_var_dict(&self) -> HashMap<String, String> {
        let mut result = HashMap::new();
        for (name, val) in &self.variables {
            result.insert(name.clone(), val.raw.clone());
        }
        result
    }

    pub fn get(&self, path: &str) -> Option<String> {
        Self::get_recursive(
            path.split(':').collect::<Vec<&str>>().as_slice(),
            &self.lines,
            &self.categories,
        )
    }

    fn get_recursive(
        parts: &[&str],
        lines: &[HyprLine],
        categories: &[HyprCategory],
    ) -> Option<String> {
        if parts.is_empty() {
            return None;
        }

        if parts.len() == 1 {
            for line in lines {
                if line.key == parts[0] {
                    return Some(line.value.raw.clone());
                }
            }
        } else {
            for cat in categories {
                if cat.name == parts[0] {
                    return Self::get_recursive(&parts[1..], &cat.lines, &cat.categories);
                }
            }
        }
        None
    }

    pub fn set(&mut self, path: &str, value: &str) {
        let parts: Vec<&str> = path.split(':').collect();
        if parts.is_empty() {
            return;
        }

        let new_value = HyprValue::new(
            value.to_string(),
            vec![HyprValuePart::Literal(value.to_string())],
        );

        Self::set_recursive(
            parts.as_slice(),
            new_value,
            &mut self.lines,
            &mut self.categories,
        );
    }

    fn set_recursive(
        parts: &[&str],
        value: HyprValue,
        lines: &mut Vec<HyprLine>,
        categories: &mut Vec<HyprCategory>,
    ) {
        if parts.len() == 1 {
            for line in lines.iter_mut() {
                if line.key == parts[0] {
                    line.value = value;
                    return;
                }
            }
            lines.push(HyprLine {
                key: parts[0].to_string(),
                value,
                is_variable: false,
            });
        } else {
            let cat_name = parts[0];
            let mut cat_idx = None;

            for (i, cat) in categories.iter().enumerate() {
                if cat.name == cat_name {
                    cat_idx = Some(i);
                    break;
                }
            }

            if let Some(idx) = cat_idx {
                let (_, cats_ref) = categories.split_at_mut(idx);
                let cat = &mut cats_ref[0];
                Self::set_recursive(&parts[1..], value, &mut cat.lines, &mut cat.categories);
            } else {
                let mut new_cat = HyprCategory::new(cat_name.to_string(), None);
                Self::set_recursive(
                    &parts[1..],
                    value,
                    &mut new_cat.lines,
                    &mut new_cat.categories,
                );
                categories.push(new_cat);
            }
        }
    }

    pub fn to_string(&self) -> String {
        let mut output = String::new();

        let mut vars: Vec<_> = self.variables.iter().collect();
        vars.sort_by_key(|(k, _)| *k);
        for (key, val) in vars {
            output.push_str(&format!("${} = {}\n", key, val.raw));
        }

        if !self.variables.is_empty() && (!self.lines.is_empty() || !self.categories.is_empty()) {
            output.push('\n');
        }

        self.append_content(&mut output, &self.lines, &self.categories, 0);

        output
    }

    fn append_content(
        &self,
        output: &mut String,
        lines: &[HyprLine],
        categories: &[HyprCategory],
        indent: usize,
    ) {
        let prefix = "    ".repeat(indent);

        for line in lines {
            let var_prefix = if line.is_variable { "$" } else { "" };
            output.push_str(&format!(
                "{}{}{} = {}\n",
                prefix, var_prefix, line.key, line.value.raw
            ));
        }

        for cat in categories {
            if !output.ends_with("\n\n") {
                output.push('\n');
            }
            let key_str = if let Some(k) = &cat.key {
                format!("[{}]", k)
            } else {
                "".to_string()
            };
            output.push_str(&format!("{}{}{} {{\n", prefix, cat.name, key_str));

            self.append_content(output, &cat.lines, &cat.categories, indent + 1);

            output.push_str(&format!("{}}}\n", prefix));
        }
    }
}
