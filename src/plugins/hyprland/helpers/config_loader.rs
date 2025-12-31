use super::types::{EnvVar, ExecCommand, Gesture, Keybind, LayerRule, Monitor, WindowRule};
use crate::utils::hyprlang::{
    HyprConf, HyprLang,
    ast::{HyprLine, HyprValue, HyprValuePart},
};
use std::path::PathBuf;

pub struct ConfigLoader {
    pub config_path: PathBuf,
    hypr_lang: Option<HyprLang>,
    config: Option<HyprConf>,
}

impl ConfigLoader {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let path = PathBuf::from(home).join(".config/hypr/hyprland.conf");
        Self {
            config_path: path,
            hypr_lang: None,
            config: None,
        }
    }

    pub fn load(&mut self) -> Result<(), String> {
        let hypr = HyprLang::new(self.config_path.to_string_lossy().to_string());
        match hypr.load() {
            Ok(conf) => {
                self.config = Some(conf);
                self.hypr_lang = Some(hypr);
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        if let (Some(lang), Some(conf)) = (&self.hypr_lang, &self.config) {
            lang.save(conf)
                .map_err(|e: crate::utils::hyprlang::HyprError| e.to_string())
        } else {
            Err("No config loaded".to_string())
        }
    }

    pub fn get_hypr_conf(&self) -> Option<&HyprConf> {
        self.config.as_ref()
    }

    pub fn get_hypr_conf_mut(&mut self) -> Option<&mut HyprConf> {
        self.config.as_mut()
    }

    pub fn get_option(&self, path: &str) -> Option<String> {
        self.config.as_ref().and_then(|c: &HyprConf| c.get(path))
    }

    pub fn set_option(&mut self, path: &str, value: &str) -> bool {
        if let Some(conf) = &mut self.config {
            conf.set(path, value);
            true
        } else {
            false
        }
    }

    pub fn get_monitors(&self) -> Vec<Monitor> {
        let mut monitors = Vec::new();
        if let Some(conf) = &self.config {
            for line in &conf.lines {
                if line.key == "monitor" {
                    let raw = &line.value.raw;
                    let parts: Vec<&str> = raw.split(',').map(|s: &str| s.trim()).collect();

                    if parts.len() == 1 && parts[0] == "disable" {
                        monitors.push(Monitor {
                            name: parts[0].to_string(),
                            resolution: "".to_string(),
                            position: "".to_string(),
                            scale: "".to_string(),
                            extras: vec![],
                            disabled: true,
                            raw: raw.clone(),
                        });
                    } else if parts.len() >= 4 {
                        monitors.push(Monitor {
                            name: parts[0].to_string(),
                            resolution: parts[1].to_string(),
                            position: parts[2].to_string(),
                            scale: parts[3].to_string(),
                            extras: parts[4..].iter().map(|s: &&str| s.to_string()).collect(),
                            disabled: false,
                            raw: raw.clone(),
                        });
                    }
                }
            }
        }
        monitors
    }

    pub fn get_binds(&self) -> Vec<Keybind> {
        let mut binds = Vec::new();
        let bind_types = [
            "bind", "binde", "bindl", "bindr", "bindm", "bindc", "bindg", "bindd", "bindt",
            "binds", "bindo", "bindu",
        ];

        if let Some(conf) = &self.config {
            for line in &conf.lines {
                if bind_types.contains(&line.key.as_str()) || line.key.starts_with("bind") {
                    let raw = &line.value.raw;
                    let parts: Vec<&str> = raw.splitn(4, ',').map(|s: &str| s.trim()).collect();

                    binds.push(Keybind {
                        bind_type: line.key.clone(),
                        mods: parts.get(0).unwrap_or(&"").to_string(),
                        key: parts.get(1).unwrap_or(&"").to_string(),
                        dispatcher: parts.get(2).unwrap_or(&"").to_string(),
                        params: parts.get(3).unwrap_or(&"").to_string(),
                        raw: raw.clone(),
                    });
                }
            }
        }
        binds
    }

    pub fn get_gestures(&self) -> Vec<Gesture> {
        let mut gestures = Vec::new();
        if let Some(conf) = &self.config {
            for line in &conf.lines {
                if line.key == "gesture" {
                    let raw = &line.value.raw;
                    let parts: Vec<&str> = raw.split(',').map(|s: &str| s.trim()).collect();

                    if parts.len() >= 3 {
                        let fingers = parts[0].parse().unwrap_or(3);
                        let direction = parts[1].to_string();

                        let mut mod_key = String::new();
                        let mut scale = String::new();
                        let mut idx = 2;

                        while idx < parts.len() {
                            let part = parts[idx];
                            if part.starts_with("mod:") {
                                mod_key = part.replace("mod:", "").trim().to_string();
                                idx += 1;
                            } else if part.starts_with("scale:") {
                                scale = part.replace("scale:", "").trim().to_string();
                                idx += 1;
                            } else {
                                break;
                            }
                        }

                        if idx < parts.len() {
                            let mut action = parts[idx].to_string();
                            let mut dispatcher = String::new();
                            let mut params = String::new();

                            if action.to_lowercase() == "dispatcher" && idx + 1 < parts.len() {
                                dispatcher = parts[idx + 1].to_string();
                                if idx + 2 < parts.len() {
                                    params = parts[idx + 2..].join(", ");
                                }
                                action = "dispatcher".to_string();
                            } else {
                                if idx + 1 < parts.len() {
                                    params = parts[idx + 1..].join(", ");
                                }
                            }

                            gestures.push(Gesture {
                                fingers,
                                direction,
                                action,
                                dispatcher,
                                params,
                                mod_key,
                                scale,
                                raw: raw.clone(),
                            });
                        }
                    }
                }
            }
        }
        gestures
    }

    pub fn get_window_rules(&self) -> Vec<WindowRule> {
        let mut rules = Vec::new();
        if let Some(conf) = &self.config {
            for line in &conf.lines {
                if line.key == "windowrule" || line.key == "windowrulev2" {
                    let raw = &line.value.raw;
                    let rule = Self::parse_window_rule(&line.key, raw);
                    rules.push(rule);
                }
            }
        }
        rules
    }

    fn parse_window_rule(rule_type: &str, raw: &str) -> WindowRule {
        let mut props = Vec::new();
        let mut effects = Vec::new();
        let mut name = None;

        let parts: Vec<&str> = raw.split(',').map(|s| s.trim()).collect();

        let is_new_syntax = parts.iter().any(|p| p.starts_with("match:"));

        if is_new_syntax || rule_type == "windowrule" && parts.iter().any(|p| p.contains(' ')) {
            for part in &parts {
                let part = part.trim();
                if part.starts_with("match:") {
                    let rest = &part[6..];
                    if let Some(space_idx) = rest.find(' ') {
                        let prop = format!("match:{}", &rest[..space_idx]);
                        let val = rest[space_idx + 1..].trim().to_string();
                        props.push((prop, val));
                    } else {
                        props.push((format!("match:{}", rest), String::new()));
                    }
                } else if part.starts_with("name ") {
                    name = Some(part[5..].trim().to_string());
                } else {
                    let effect_parts: Vec<&str> = part.splitn(2, ' ').collect();
                    let effect_name = effect_parts[0].to_string();
                    let effect_val = effect_parts
                        .get(1)
                        .map(|s| s.to_string())
                        .unwrap_or_default();
                    effects.push((effect_name, effect_val));
                }
            }
        } else {
            if parts.len() >= 2 {
                let effect_str = parts[0];
                let effect_parts: Vec<&str> = effect_str.splitn(2, ' ').collect();
                effects.push((
                    effect_parts[0].to_string(),
                    effect_parts
                        .get(1)
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                ));

                for part in &parts[1..] {
                    if let Some((key, val)) = part.split_once(':') {
                        props.push((format!("match:{}", key), val.to_string()));
                    }
                }
            }
        }

        WindowRule {
            name,
            rule_type: "windowrule".to_string(),
            props,
            effects,
            raw: raw.to_string(),
            is_block: false,
        }
    }

    pub fn get_layer_rules(&self) -> Vec<LayerRule> {
        let mut rules = Vec::new();
        if let Some(conf) = &self.config {
            for line in &conf.lines {
                if line.key == "layerrule" {
                    let raw = &line.value.raw;
                    let parts: Vec<&str> = raw.split(',').map(|s| s.trim()).collect();

                    let mut props = Vec::new();
                    let mut effects = Vec::new();

                    for part in &parts {
                        if part.starts_with("match:namespace") {
                            let val = part.strip_prefix("match:namespace").unwrap_or("").trim();
                            props.push(("match:namespace".to_string(), val.to_string()));
                        } else {
                            let effect_parts: Vec<&str> = part.splitn(2, ' ').collect();
                            effects.push((
                                effect_parts[0].to_string(),
                                effect_parts
                                    .get(1)
                                    .map(|s| s.to_string())
                                    .unwrap_or_default(),
                            ));
                        }
                    }

                    if props.is_empty() && parts.len() >= 2 {
                        props.push(("match:namespace".to_string(), parts[1].to_string()));
                    }

                    rules.push(LayerRule {
                        props,
                        effects,
                        raw: raw.to_string(),
                    });
                }
            }
        }
        rules
    }

    pub fn get_exec(&self) -> Vec<ExecCommand> {
        let mut cmds = Vec::new();
        if let Some(conf) = &self.config {
            for line in &conf.lines {
                if line.key == "exec" || line.key == "exec-once" {
                    cmds.push(ExecCommand {
                        exec_type: line.key.clone(),
                        command: line.value.raw.clone(),
                        raw: line.value.raw.clone(),
                    });
                }
            }
        }
        cmds
    }

    pub fn get_env(&self) -> Vec<EnvVar> {
        let mut vars = Vec::new();
        if let Some(conf) = &self.config {
            for line in &conf.lines {
                if line.key == "env" {
                    let raw = &line.value.raw;
                    if let Some((name, val)) = raw.split_once(',') {
                        let name_str: String = name.trim().to_string();
                        let val_str: String = val.trim().to_string();
                        vars.push(EnvVar {
                            name: name_str,
                            value: val_str,
                            raw: raw.clone(),
                        });
                    }
                }
            }
        }
        vars
    }

    fn add_line(&mut self, key: &str, value: &str) {
        if let Some(conf) = &mut self.config {
            conf.lines.push(HyprLine {
                key: key.to_string(),
                value: HyprValue::new(
                    value.to_string(),
                    vec![HyprValuePart::Literal(value.to_string())],
                ),
                is_variable: false,
            });
        }
    }

    fn remove_line(&mut self, key_filter: &[&str], raw_value: &str) -> bool {
        if let Some(conf) = &mut self.config {
            if let Some(pos) = conf
                .lines
                .iter()
                .position(|l| key_filter.contains(&l.key.as_str()) && l.value.raw == raw_value)
            {
                conf.lines.remove(pos);
                return true;
            }
        }
        false
    }

    fn update_line(
        &mut self,
        key_filter: &[&str],
        old_raw: &str,
        new_key: &str,
        new_value: &str,
    ) -> bool {
        if let Some(conf) = &mut self.config {
            if let Some(pos) = conf
                .lines
                .iter()
                .position(|l| key_filter.contains(&l.key.as_str()) && l.value.raw == old_raw)
            {
                conf.lines[pos] = HyprLine {
                    key: new_key.to_string(),
                    value: HyprValue::new(
                        new_value.to_string(),
                        vec![HyprValuePart::Literal(new_value.to_string())],
                    ),
                    is_variable: false,
                };
                return true;
            }
        }
        false
    }

    fn format_window_rule_new(rule: &WindowRule) -> String {
        let mut parts = Vec::new();
        for (prop, val) in &rule.props {
            if val.is_empty() {
                parts.push(prop.clone());
            } else {
                parts.push(format!("{} {}", prop, val));
            }
        }
        for (effect, val) in &rule.effects {
            if val.is_empty() || val == "on" {
                parts.push(effect.clone());
            } else {
                parts.push(format!("{} {}", effect, val));
            }
        }
        parts.join(", ")
    }

    fn format_window_rule_legacy(rule: &WindowRule) -> String {
        let effect_str = rule
            .effects
            .iter()
            .map(|(k, v)| {
                if v.is_empty() || v == "on" {
                    k.clone()
                } else {
                    format!("{} {}", k, v)
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        let match_str = rule
            .props
            .iter()
            .map(|(k, v)| {
                let key = k.strip_prefix("match:").unwrap_or(k);
                format!("{}:{}", key, v)
            })
            .collect::<Vec<_>>()
            .join(", ");

        format!("{}, {}", effect_str, match_str)
    }

    pub fn add_window_rule(&mut self, rule: WindowRule, use_new_syntax: bool) {
        if use_new_syntax {
            let val = Self::format_window_rule_new(&rule);
            self.add_line("windowrule", &val);
        } else {
            let val = Self::format_window_rule_legacy(&rule);
            self.add_line("windowrulev2", &val);
        }
    }

    pub fn delete_window_rule(&mut self, raw: &str) {
        self.remove_line(&["windowrule", "windowrulev2", "layerrule"], raw);
    }

    pub fn update_window_rule(&mut self, old_raw: &str, rule: WindowRule, use_new_syntax: bool) {
        if use_new_syntax {
            let val = Self::format_window_rule_new(&rule);
            self.update_line(&["windowrule", "windowrulev2"], old_raw, "windowrule", &val);
        } else {
            let val = Self::format_window_rule_legacy(&rule);
            self.update_line(
                &["windowrule", "windowrulev2"],
                old_raw,
                "windowrulev2",
                &val,
            );
        }
    }

    pub fn add_layer_rule(&mut self, rule: LayerRule) {
        let mut parts = Vec::new();
        for (prop, val) in &rule.props {
            parts.push(format!("{} {}", prop, val));
        }
        for (effect, val) in &rule.effects {
            if val.is_empty() {
                parts.push(effect.clone());
            } else {
                parts.push(format!("{} {}", effect, val));
            }
        }
        self.add_line("layerrule", &parts.join(", "));
    }

    pub fn delete_layer_rule(&mut self, raw: &str) {
        self.remove_line(&["layerrule"], raw);
    }

    pub fn update_layer_rule(&mut self, old_raw: &str, rule: LayerRule) {
        let mut parts = Vec::new();
        for (prop, val) in &rule.props {
            parts.push(format!("{} {}", prop, val));
        }
        for (effect, val) in &rule.effects {
            if val.is_empty() {
                parts.push(effect.clone());
            } else {
                parts.push(format!("{} {}", effect, val));
            }
        }
        self.update_line(&["layerrule"], old_raw, "layerrule", &parts.join(", "));
    }

    pub fn add_exec(&mut self, cmd: ExecCommand) {
        self.add_line(&cmd.exec_type, &cmd.command);
    }

    pub fn delete_exec(&mut self, raw: &str) {
        self.remove_line(&["exec", "exec-once"], raw);
    }

    pub fn update_exec(&mut self, old_raw: &str, cmd: ExecCommand) {
        self.update_line(
            &["exec", "exec-once"],
            old_raw,
            &cmd.exec_type,
            &cmd.command,
        );
    }

    pub fn add_env(&mut self, env: EnvVar) {
        let val = format!("{},{}", env.name, env.value);
        self.add_line("env", &val);
    }

    pub fn delete_env(&mut self, raw: &str) {
        self.remove_line(&["env"], raw);
    }

    pub fn update_env(&mut self, old_raw: &str, env: EnvVar) {
        let val = format!("{},{}", env.name, env.value);
        self.update_line(&["env"], old_raw, "env", &val);
    }

    pub fn add_bind(&mut self, bind: Keybind) {
        let val = format!(
            "{}, {}, {}, {}",
            bind.mods, bind.key, bind.dispatcher, bind.params
        );
        self.add_line(&bind.bind_type, &val);
    }

    pub fn delete_bind(&mut self, raw: &str) {
        let bind_types = [
            "bind", "binde", "bindl", "bindr", "bindm", "bindc", "bindg", "bindd", "bindt",
            "binds", "bindo", "bindu",
        ];
        self.remove_line(&bind_types, raw);
    }

    pub fn update_bind(&mut self, old_raw: &str, bind: Keybind) {
        let bind_types = [
            "bind", "binde", "bindl", "bindr", "bindm", "bindc", "bindg", "bindd", "bindt",
            "binds", "bindo", "bindu",
        ];
        let val = format!(
            "{}, {}, {}, {}",
            bind.mods, bind.key, bind.dispatcher, bind.params
        );
        self.update_line(&bind_types, old_raw, &bind.bind_type, &val);
    }

    pub fn add_gesture(&mut self, gesture: Gesture) {
        let mut parts = vec![gesture.fingers.to_string(), gesture.direction];
        if !gesture.mod_key.is_empty() {
            parts.push(format!("mod:{}", gesture.mod_key));
        }
        if !gesture.scale.is_empty() {
            parts.push(format!("scale:{}", gesture.scale));
        }

        if gesture.action == "dispatcher" {
            parts.push("dispatcher".to_string());
            parts.push(gesture.dispatcher);
            if !gesture.params.is_empty() {
                parts.push(gesture.params);
            }
        } else {
            parts.push(gesture.action);
        }

        let val = parts.join(", ");
        self.add_line("gesture", &val);
    }

    pub fn delete_gesture(&mut self, raw: &str) {
        self.remove_line(&["gesture"], raw);
    }

    pub fn update_gesture(&mut self, old_raw: &str, gesture: Gesture) {
        let mut parts = vec![gesture.fingers.to_string(), gesture.direction];
        if !gesture.mod_key.is_empty() {
            parts.push(format!("mod:{}", gesture.mod_key));
        }
        if !gesture.scale.is_empty() {
            parts.push(format!("scale:{}", gesture.scale));
        }

        if gesture.action == "dispatcher" {
            parts.push("dispatcher".to_string());
            parts.push(gesture.dispatcher);
            if !gesture.params.is_empty() {
                parts.push(gesture.params);
            }
        } else {
            parts.push(gesture.action);
        }

        let val = parts.join(", ");
        self.update_line(&["gesture"], old_raw, "gesture", &val);
    }
}
