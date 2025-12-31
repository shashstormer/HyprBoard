#[cfg(test)]
mod tests {
    use crate::plugins::waybar::parser::{parse, set_value, to_json_value, to_string};

    #[test]
    fn test_ast_parsing_and_roundtrip() {
        let jsonc_content = r#"{
            "layer": "top",
            "modules-left": ["hyprland/workspaces"], // inline comment
            /* block comment */
            "clock": {
                "format": "value // not a comment"
            }
        }"#;

        let res = parse(jsonc_content);
        assert!(res.is_ok(), "Should parse JSONC with comments");
        let node = res.unwrap();

        let output = to_string(&node);
        assert_eq!(
            output, jsonc_content,
            "Roundtrip should preserve exact content including comments"
        );

        let json = to_json_value(&node);
        assert_eq!(json["layer"], "top");
        assert_eq!(json["clock"]["format"], "value // not a comment");
    }

    #[test]
    fn test_real_world_waybar_config() {
        let config = r#"{
    "layer": "top", // Waybar at top layer
    "position": "top", // Waybar position (top|bottom|left|right)
    "height": 40,
    "margin": "0 5 0 5",
    "spacing":0,


    "modules-left": ["custom/launcher", "hyprland/workspaces", "hyprland/window"],
    "modules-center": ["custom/network_traffic"],
    "modules-right": [
                "custom/updates",
                "backlight",
                "temperature",
                "cpu",
                "memory",
                "battery",
                "pulseaudio",
                "network",
                "tray",
                "idle_inhibitor",
                "clock",
        ],


  "hyprland/workspaces": {
    "format": "{icon}",
    "on-click": "activate",
    "all-outputs": true,
    "sort-by-number": true,
    "format-icons": {
      "1": "1",
      "2": "2",
      "3": "3",
      "4": "4",
      "5": "5",
      "6": "6",
      "7": "7",
      "8": "8",
      "9": "9",
      "10": "10",
      "focused": "",
      "default": ""
    },
    "on-scroll-up": "hyprctl dispatch workspace e+1",
    "on-scroll-down": "hyprctl dispatch workspace e-1",
  },
  "clock": {
        "format": " {:%a %d %b  %I:%M %p}",	//12 hour format
        //"format": " {:%a %d %b  %H:%M}",	//24 hour format
        "format-alt": " {:%d/%m/%Y  %H:%M:%S}",
        "interval": 1,
    }
}"#;

        let res = parse(config);
        assert!(
            res.is_ok(),
            "Should parse real world config. Error: {:?}",
            res.err()
        );
        let node = res.unwrap();

        let output = to_string(&node);
        assert_eq!(output, config, "Real world config round-trip failed");
    }

    #[test]
    fn test_complex_value_insertion() {
        let initial = r#"{
    "existing": "value"
}"#;
        let mut root = parse(initial).unwrap();

        let complex_json = serde_json::json!({
            "nested": {
                "key": "value",
                "list": [1, 2, 3]
            }
        });

        set_value(&mut root, &["new_complex"], complex_json);

        let output = to_string(&root);
        assert!(output.contains("\"new_complex\": {"));
        assert!(output.contains("\"nested\": {"));
        assert!(output.contains("\"list\": ["));
        assert!(output.contains("1,"));

        let reparsed = parse(&output);
        assert!(reparsed.is_ok());
    }

    #[test]
    fn test_deep_path_creation() {
        let initial = r#"{
    "level1": {}
}"#;
        let mut root = parse(initial).unwrap();

        set_value(
            &mut root,
            &["level1", "level2", "level3"],
            serde_json::Value::String("deep value".into()),
        );

        let output = to_string(&root);
        println!("Deep path output:\n{}", output);

        assert!(output.contains("\"level2\": {"));
        assert!(output.contains("\"level3\": \"deep value\""));

        let json = to_json_value(&root);
        assert_eq!(json["level1"]["level2"]["level3"], "deep value");
    }

    #[test]
    fn test_python_reference_parity() {
        let config_text = r#"{
    "layer": "top", // Waybar at top layer
    "position": "top", // Waybar position (top|bottom|left|right)
    "height": 40,
    "margin": "0 5 0 5",
    "spacing":0,


    "modules-left": ["custom/launcher", "hyprland/workspaces", "hyprland/window"],
    "modules-center": ["custom/network_traffic"],
    "modules-right": ["custom/updates", "backlight", "custom/keyboard-layout", "temperature", "cpu", "memory", "battery", "pulseaudio", "network", "tray", "idle_inhibitor", "clock","custom/power"],


  "hyprland/workspaces": {
    "format": "{icon}",
    "on-click": "activate",
    "all-outputs": true,
    "sort-by-number": true,
    "format-icons": {
      "1": "1",
      "2": "2",
      "3": "3",
      "4": "4",
      "5": "5",
      "6": "6",
      "7": "7",
      "8": "8",
      "9": "9",
      "10": "10",
      "focused": "",
      "default": ""
    },
    "on-scroll-up": "hyprctl dispatch workspace e+1",
    "on-scroll-down": "hyprctl dispatch workspace e-1",
    },
    "hyprland/window": {
        "format": "{}",
	"icon":true,
	"icon-size" : 20,
	"swap-icon-label": true
    },
    "idle_inhibitor": {
        "format": "{icon}",
       "format-icons": {
            "activated": "",
            "deactivated": "",
        },
        "on-click":"exec ~/.config/hypr/scripts/idle_inhibitor.sh",

    },
    "tray": {
        "icon-size": 20,
        "spacing": 5
    },
    "clock": {
        "tooltip-format": "<big>{:%A, %d.%B %Y }</big>\n<tt><small>{calendar}</small></tt>",
        "format": " {:%a %d %b  %I:%M %p}",	//12 hour format
        //"format": " {:%a %d %b  %H:%M}",	//24 hour format
        "format-alt": " {:%d/%m/%Y  %H:%M:%S}",
        //"max-length": 200
        "interval": 1,
        "on-click": "~/.config/waybar/scripts/OCV",
    },
    "cpu": {
        "format": "🖳{usage}%",
        "on-click": "foot -e htop"
    },
    "memory": {
        "format": "🖴 {: >3}%",
        "on-click": "foot -e htop"
    },
    "temperature": {
        "thermal-zone": 7,  // Check with: # cat /sys/class/hwmon/hwmon*/temp1_input
        "hwmon-path": "/sys/class/hwmon/hwmon7/temp1_input",
        "critical-threshold": 80,
        "format-critical": "{temperatureC}°C ",
        "format": "{temperatureC}°C "
    },
    "backlight": {
        "format": "{icon} {percent: >3}%",
        "format-icons": ["", ""],
        "on-scroll-down": "brightnessctl -c backlight set 1%-",
        "on-scroll-up": "brightnessctl -c backlight set +1%",
        "on-click": "~/.config/waybar/scripts/backlight-hint.sh"
    },
    "battery": {
        "states": {
            "warning": 30,
            "critical": 15
        },
        "format": "{icon} {capacity: >3}%",
        "format-icons": ["", "", "", "", ""]
        //"format-icons": ["", "", "", "", "", "", "", "", "", ""]
        //"format": "&#x202b;{icon}&#x202c; {capacity}%",
        //"format-icons": ["ﱉ","ﱊ","ﱌ","ﱍ","ﱋ"]
    },
      "network": {
        //"interface": "wlp0s20f3", // (Optional) To force the use of this interface  "format-wifi": "  {essid}",
        "format": "⚠Disabled",
        "format-wifi": "",
        "format-ethernet": "",
        "format-linked": "{ifname} (No IP)",
        "format-disconnected": "⚠Disabled",
        "format-alt": "{ifname}: {ipaddr}/{cidr}",
        "family": "ipv4",
        "tooltip-format-wifi": "  {ifname} @ {essid}\nIP: {ipaddr}\nStrength: {signalStrength}%\nFreq: {frequency}MHz\nUp: {bandwidthUpBits} Down: {bandwidthDownBits}",
        "tooltip-format-ethernet": " {ifname}\nIP: {ipaddr}\n up: {bandwidthUpBits} down: {bandwidthDownBits}",
        //"min-length": 2,
        //"max-length": 2,
        "on-click": "nm-connection-editor"
    },
   "custom/updates": {
       "format": "{} {icon}",
       "return-type": "json",
       "format-icons": {
           "has-updates": "󱍷",
           "updated": "󰂪",
       },
       "exec-if": "which waybar-module-pacman-updates",
       "exec": "waybar-module-pacman-updates --interval-seconds 5 --network-interval-seconds 7200 --network-interval-seconds",
       "on-click": "foot -e update"
   },
    "custom/power": {
      	"format":"⏻",
       	"on-click": "nwgbar",
      	"tooltip": false,
   },
   "custom/keyboard-layout": {
      	"format": " Cheat", // Icon: keyboard
        "on-click": "~/.config/waybar/scripts/keyhint.sh",
     },
    "custom/launcher": {
    "format":"    ",
    	"on-click": "exec nwg-drawer -c 7 -is 70 -spacing 23",
    	"tooltip": false,
     },
     "custom/network_traffic": {
             "exec": "~/.config/waybar/scripts/network_traffic.sh",
             "return-type": "json",
             "format-ethernet": "{icon} {ifname} ⇣{bandwidthDownBytes} ⇡{bandwidthUpBytes}",    // optional
},
    "pulseaudio": {
        "scroll-step": 3, // %, can be a float
        "format": "{icon} {volume}% {format_source}",
        "format-bluetooth": "{volume}% {icon} {format_source}",
        "format-bluetooth-muted": " {icon} {format_source}",
        "format-muted": " {format_source}",
        //"format-source": "{volume}% ",
        //"format-source-muted": "",
        "format-source": "",
        "format-source-muted": "",
        "format-icons": {
            "headphone": "",
            "hands-free": "",
            "headset": "",
            "phone": "",
            "portable": "",
            "car": "",
            "default": ["", "", ""]
        },
        "on-click": "footclient -T waybar_alsamixer -e alsamixer -M",
        "on-click-right": "pavucontrol"
        },
        "custom/weather": {
        "exec": "curl 'https://wttr.in/Essen?format=2'",
        "interval": 900,
	    "on-click": "yad --html --uri='https://wttr.in/Essen' --center --fixed --width=1000 --height=680 --timeout=60 --timeout-indicator=right"
        },
    }
"#;
        let res = parse(config_text);
        assert!(res.is_ok(), "Should parse the Python reference config");
        let node = res.unwrap();

        let output = to_string(&node);
        if output != config_text {
            let len_output = output.len();
            let len_expected = config_text.len();
            println!("Output len: {}, Expected len: {}", len_output, len_expected);

            let mismatched_idx = output
                .chars()
                .zip(config_text.chars())
                .position(|(a, b)| a != b);
            if let Some(idx) = mismatched_idx {
                println!("First mismatch at index {}:", idx);
                let start = if idx > 20 { idx - 20 } else { 0 };
                let end = if idx + 20 < output.len() {
                    idx + 20
                } else {
                    output.len()
                };
                println!("Output slice: {:?}", &output[start..end]);

                let end_exp = if idx + 20 < config_text.len() {
                    idx + 20
                } else {
                    config_text.len()
                };
                println!("Expctd slice: {:?}", &config_text[start..end_exp]);
            } else {
                println!("No mismatch found in common prefix?");
            }
            panic!(
                "Reference config round-trip failed (length mismatch or content mismatch). See output above."
            );
        }
    }
}
