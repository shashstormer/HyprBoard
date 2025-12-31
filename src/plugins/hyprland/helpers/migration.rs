use crate::utils::hyprlang::HyprConf;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct MigrationResult {
    pub migrated_rules: usize,
    pub renamed_options: usize,
    pub backup_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HyprlandVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl HyprlandVersion {
    pub fn detect() -> Option<Self> {
        let output = Command::new("hyprctl").arg("version").output().ok()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Self::parse(stdout.as_ref())
    }

    pub fn parse(output: &str) -> Option<Self> {
        for line in output.lines() {
            
            if line.contains("Hyprland l") {
                
                continue;
            }

            if let Some(pos) = line.find("v") {
                
                let version_part = &line[pos + 1..];
                
                let version_str = version_part
                    .split(|c: char| !c.is_numeric() && c != '.')
                    .next()
                    .unwrap_or("");
                
                let nums: Vec<&str> = version_str.split('.').collect();
                 if nums.len() >= 3 {
                    return Some(Self {
                        major: nums[0].parse().unwrap_or(0),
                        minor: nums[1].parse().unwrap_or(0),
                        patch: nums[2].parse().unwrap_or(0),
                    });
                }
            }

            
             if line.trim().starts_with("Tag:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let version_str = parts[1].trim_start_matches('v');
                    let nums: Vec<&str> = version_str.split('.').collect();
                    if nums.len() >= 3 {
                        return Some(Self {
                            major: nums[0].parse().unwrap_or(0),
                            minor: nums[1].parse().unwrap_or(0),
                            patch: nums[2].parse().unwrap_or(0),
                        });
                    }
                }
            }
        }
        
        
        
        None
    }

    pub fn supports_new_window_rules(&self) -> bool {
        self.major > 0 || (self.major == 0 && self.minor >= 53)
    }

    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

pub struct ConfigMigrator;

impl ConfigMigrator {
    pub fn needs_migration(conf: &HyprConf) -> bool {
        for line in &conf.lines {
            
            if line.key.eq_ignore_ascii_case("windowrulev2") {
                return true;
            }
            
            if line.key.eq_ignore_ascii_case("windowrule") && !line.value.raw.contains("match:") {
                 return true;
            }
            
            if line.key.eq_ignore_ascii_case("layerrule") && !line.value.raw.contains("match:") {
                return true;
            }
            if line.key.to_lowercase().contains("new_window_takes_over_fullscreen") {
                return true;
            }
        }
        false
    }

    pub fn backup_config(path: &Path) -> Result<PathBuf, String> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let backup_name = format!(
            "{}.bak.{}",
            path.file_name().unwrap_or_default().to_string_lossy(),
            timestamp
        );
        let backup_path = path.parent().unwrap_or(Path::new(".")).join(backup_name);

        fs::copy(path, &backup_path).map_err(|e| e.to_string())?;
        Ok(backup_path)
    }

    fn split_respecting_grouping(s: &str, delimiter: char, max_splits: usize) -> Vec<String> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut paren_depth = 0;
        let mut bracket_depth = 0;
        let mut brace_depth = 0;
        let mut splits = 0;

        for c in s.chars() {
            if max_splits > 0 && splits >= max_splits {
                current.push(c);
                continue;
            }

            match c {
                '(' => paren_depth += 1,
                ')' => if paren_depth > 0 { paren_depth -= 1 },
                '[' => bracket_depth += 1,
                ']' => if bracket_depth > 0 { bracket_depth -= 1 },
                '{' => brace_depth += 1,
                '}' => if brace_depth > 0 { brace_depth -= 1 },
                _ => {}
            }

            if c == delimiter && paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 {
                parts.push(current.trim().to_string());
                current.clear();
                splits += 1;
            } else {
                current.push(c);
            }
        }
        parts.push(current.trim().to_string());
        parts
    }

    pub fn migrate(conf: &mut HyprConf) -> MigrationResult {
        let mut migrated_rules = 0;
        let mut renamed_options = 0;

        for line in &mut conf.lines {
            let is_v2_key = line.key.eq_ignore_ascii_case("windowrulev2");
            let is_legacy_rule = line.key.eq_ignore_ascii_case("windowrule") && !line.value.raw.contains("match:");
            let is_legacy_layer = line.key.eq_ignore_ascii_case("layerrule") && !line.value.raw.contains("match:");

            if is_v2_key || is_legacy_rule {
                line.key = "windowrule".to_string();

                let raw = &line.value.raw;
                let parts = Self::split_respecting_grouping(raw, ',', 1);

                if parts.len() >= 2 {
                    let effect = &parts[0];
                    let match_part = &parts[1];

                    let mut new_parts = Vec::new();

                    
                    
                    let has_v2_keys = match_part.contains("class:") 
                        || match_part.contains("title:") 
                        || match_part.contains("initialClass:")
                        || match_part.contains("initialTitle:")
                        || match_part.contains("floating:")
                        || match_part.contains("xwayland:")
                        || match_part.contains("pinned:")
                        || match_part.contains("workspace:")
                        || match_part.contains("fullscreen:");

                    if is_v2_key || has_v2_keys {
                        
                        let matches = Self::split_respecting_grouping(match_part, ',', 0);
                        for m in matches {
                            if let Some((key, val)) = m.split_once(':') {
                                
                                
                                
                                
                                
                                
                                
                                
                                
                                
                                
                                
                                
                                
                                
                                let new_key = match key.trim() {
                                    "floating" => "float",
                                    k => k,
                                };
                                
                                new_parts.push(format!("match:{} {}", new_key, val));
                            }
                        }
                    } else {
                        
                        
                        new_parts.push(format!("match:class {}", match_part));
                    }

                    
                    let effect_parts: Vec<&str> = effect.splitn(2, ' ').collect();
                    if effect_parts.len() == 2 {
                        new_parts.push(format!("{} {}", effect_parts[0], effect_parts[1]));
                    } else {
                        new_parts.push(format!("{} on", effect));
                    }

                    let new_raw = new_parts.join(", ");
                    line.value.raw = new_raw.clone();
                    line.value.parts =
                        vec![crate::utils::hyprlang::ast::HyprValuePart::Literal(new_raw)];
                }

                migrated_rules += 1;
            } else if is_legacy_layer {
                
                
                
                
                
                let raw = &line.value.raw;
                let parts = Self::split_respecting_grouping(raw, ',', 1);
                
                if parts.len() >= 2 {
                    let mut effect = parts[0].trim().to_string();
                    let match_part = parts[1].trim();
                    
                    
                    if effect == "stayfocused" {
                        effect = "stay_focused".to_string();
                    }

                    let mut new_parts = Vec::new();
                    
                    
                    
                    
                    
                    
                    
                    
                    
                    
                    
                    if effect.contains(' ') {
                        new_parts.push(effect);
                    } else {
                        new_parts.push(format!("{} on", effect));
                    }
                    
                    new_parts.push(format!("match:namespace {}", match_part));
                    
                    let new_raw = new_parts.join(", ");
                    line.value.raw = new_raw.clone();
                    line.value.parts =
                        vec![crate::utils::hyprlang::ast::HyprValuePart::Literal(new_raw)];
                    
                    migrated_rules += 1;
                }
            }

            if line.key.to_lowercase() == "misc:new_window_takes_over_fullscreen" {
                line.key = "misc:new_window_takes_over_fs".to_string();
                renamed_options += 1;
            }
        }

        MigrationResult {
            migrated_rules,
            renamed_options,
            backup_path: None,
        }
    }

    pub fn get_migration_summary(conf: &HyprConf) -> String {
        let rule_count = conf
            .lines
            .iter()
            .filter(|l| {
                l.key.eq_ignore_ascii_case("windowrulev2") ||
                (l.key.eq_ignore_ascii_case("windowrule") && !l.value.raw.contains("match:"))
            })
            .count();
        
        let layer_count = conf
            .lines
            .iter()
            .filter(|l| {
                l.key.eq_ignore_ascii_case("layerrule") && !l.value.raw.contains("match:")
            })
            .count();
            
        let old_option = conf
            .lines
            .iter()
            .any(|l| l.key.to_lowercase().contains("new_window_takes_over_fullscreen"));

        let mut summary = String::new();
        if rule_count > 0 {
            summary.push_str(&format!(
                "• {} legacy window rules → windowrule (new syntax)\n",
                rule_count
            ));
        }
        if layer_count > 0 {
            summary.push_str(&format!(
                "• {} legacy layer rules → layerrule (new syntax)\n",
                layer_count
            ));
        }
        if old_option {
            summary.push_str(
                "• misc:new_window_takes_over_fullscreen → misc:new_window_takes_over_fs\n",
            );
        }
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::hyprlang::ast::{HyprLine, HyprValue, HyprValuePart};

    #[test]
    fn test_version_parsing() {
        
        let v1 = HyprlandVersion::parse("Hyprland v0.40.0");
        assert_eq!(
            v1,
            Some(HyprlandVersion {
                major: 0,
                minor: 40,
                patch: 0
            })
        );
        
        
        let v4 = HyprlandVersion::parse(
            "Hyprland, built from branch main at commit ea444c330040716c9431e51b697395066928236d (v0.53.0).
Date: Tue Dec 31 12:00:00 2024
Tag: v0.53.0"
        );
        assert_eq!(
            v4,
            Some(HyprlandVersion {
                major: 0,
                minor: 53,
                patch: 0
            })
        );
    }

    #[test]
    fn test_supports_new_rules() {
        let old = HyprlandVersion {
            major: 0,
            minor: 40,
            patch: 0,
        };
        assert!(!old.supports_new_window_rules());

        let new = HyprlandVersion {
            major: 0,
            minor: 53,
            patch: 0,
        };
        assert!(new.supports_new_window_rules());
    }

    #[test]
    fn test_needs_migration() {
        let mut conf = HyprConf {
            lines: vec![],
            variables: std::collections::HashMap::new(),
            categories: vec![],
        };

        
        assert!(!ConfigMigrator::needs_migration(&conf));

        
        conf.lines.push(HyprLine {
            key: "windowrulev2".to_string(),
            value: HyprValue::new("".to_string(), vec![]),
            is_variable: false,
        });
        assert!(ConfigMigrator::needs_migration(&conf));

        
        conf.lines[0].key = "WindowRuleV2".to_string();
        assert!(ConfigMigrator::needs_migration(&conf));

        
        conf.lines[0].key = "windowrule".to_string();
        conf.lines[0].value.raw = "float, ^(kitty)$".to_string();
        assert!(ConfigMigrator::needs_migration(&conf));
        
        
        conf.lines.push(HyprLine {
            key: "layerrule".to_string(),
            value: HyprValue::new("blur, waybar".to_string(), vec![]),
            is_variable: false,
        });
        assert!(ConfigMigrator::needs_migration(&conf));

        
        conf.lines[0].key = "windowrule".to_string();
        conf.lines[0].value.raw = "float on, match:class ^(kitty)$".to_string();
        assert!(ConfigMigrator::needs_migration(&conf)); 
    }

    #[test]
    fn test_migrate_logic() {
        let mut conf = HyprConf {
            lines: vec![
                
                HyprLine {
                    key: "windowrulev2".to_string(),
                    value: HyprValue::new(
                        "float,class:^(kitty)$".to_string(),
                        vec![HyprValuePart::Literal(
                            "float,class:^(kitty)$".to_string(),
                        )],
                    ),
                    is_variable: false,
                },
                
                HyprLine {
                    key: "windowrule".to_string(),
                    value: HyprValue::new(
                        "float, ^(firefox)$".to_string(),
                        vec![HyprValuePart::Literal(
                            "float, ^(firefox)$".to_string(),
                        )],
                    ),
                    is_variable: false,
                },
            ],
            variables: std::collections::HashMap::new(),
            categories: vec![],
        };

        let res = ConfigMigrator::migrate(&mut conf);

        assert_eq!(res.migrated_rules, 2);

        
        let v2_val = &conf.lines[0].value.raw;
        assert!(v2_val.contains("match:class ^(kitty)$"));
        assert!(v2_val.contains("float on"));

        
        let v1_val = &conf.lines[1].value.raw;
        assert!(v1_val.contains("match:class ^(firefox)$"));
        assert!(v1_val.contains("float on"));
    }
    #[test]
    fn test_migrate_with_commas() {
        let mut conf = HyprConf {
            lines: vec![
                HyprLine {
                    key: "windowrulev2".to_string(),
                    value: HyprValue::new(
                        "float,title:^(foo, bar)$".to_string(),
                        vec![HyprValuePart::Literal(
                            "float,title:^(foo, bar)$".to_string(),
                        )],
                    ),
                    is_variable: false,
                },
            ],
            variables: std::collections::HashMap::new(),
            categories: vec![],
        };

        let res = ConfigMigrator::migrate(&mut conf);
        assert_eq!(res.migrated_rules, 1);
        
        let new_val = &conf.lines[0].value.raw;
        
        
        assert!(new_val.contains("match:title ^(foo, bar)$"));
        assert!(new_val.contains("float on"));
    }

    #[test]
    fn test_migrate_user_full_config() {
        
        let lines = vec![
            ("windowrule", "float, title:(^(kitty)$)"),
            ("windowrule", "opacity 0.85 override 0.85 override, title:(^(thunar)$)"),
            ("windowrule", "opacity 0.9, class:^(google-chrome)$, title:(.*ArchBoard.*)"),
            ("windowrule", "fullscreen, class:spotify"),
        ];

        let mut conf = HyprConf {
             lines: lines.into_iter().map(|(k, v)| HyprLine {
                 key: k.to_string(),
                 value: HyprValue::new(v.to_string(), vec![HyprValuePart::Literal(v.to_string())]),
                 is_variable: false,
             }).collect(),
             variables: std::collections::HashMap::new(),
             categories: vec![],
        };

        let res = ConfigMigrator::migrate(&mut conf);

        assert_eq!(res.migrated_rules, 4);

        
        let f0 = &conf.lines[0].value.raw;
        assert!(f0.contains("match:title ^(kitty)$") || f0.contains("match:title (^(kitty)$)"));
        assert!(f0.contains("float on"));

        
        let f1 = &conf.lines[1].value.raw;
        assert!(f1.contains("match:title ^(thunar)$") || f1.contains("match:title (^(thunar)$)"));
        assert!(f1.contains("opacity 0.85 override 0.85 override"));

        
        let f2 = &conf.lines[2].value.raw;
        assert!(f2.contains("match:class ^(google-chrome)$"));
        assert!(f2.contains("match:title .*ArchBoard.*") || f2.contains("match:title (.*ArchBoard.*)"));
        assert!(f2.contains("opacity 0.9"));

        
        let f3 = &conf.lines[3].value.raw;
        assert!(f3.contains("match:class spotify"));
        assert!(f3.contains("fullscreen on"));
    }

    #[test]
    fn test_layerrule_migration() {
        let mut conf = HyprConf {
            lines: vec![
                HyprLine {
                    key: "layerrule".to_string(),
                    value: HyprValue::new(
                        "blur, waybar".to_string(),
                        vec![HyprValuePart::Literal("blur, waybar".to_string())],
                    ),
                    is_variable: false,
                },
                HyprLine {
                    key: "layerrule".to_string(),
                    value: HyprValue::new(
                        "ignore_alpha 0.5, rofi".to_string(),
                        vec![HyprValuePart::Literal("ignore_alpha 0.5, rofi".to_string())],
                    ),
                    is_variable: false,
                },
                HyprLine {
                    key: "layerrule".to_string(),
                    value: HyprValue::new(
                        "stayfocused, wofi".to_string(),
                        vec![HyprValuePart::Literal("stayfocused, wofi".to_string())],
                    ),
                    is_variable: false,
                },
            ],
            variables: std::collections::HashMap::new(),
            categories: vec![],
        };

        let res = ConfigMigrator::migrate(&mut conf);
        assert_eq!(res.migrated_rules, 3);

        let l0 = &conf.lines[0].value.raw;
        assert!(l0.contains("match:namespace waybar"));
        assert!(l0.contains("blur on"));

        let l1 = &conf.lines[1].value.raw;
        assert!(l1.contains("match:namespace rofi"));
        assert!(l1.contains("ignore_alpha 0.5"));

        let l2 = &conf.lines[2].value.raw;
        assert!(l2.contains("match:namespace wofi"));
        assert!(l2.contains("stay_focused on"));
    }
}
