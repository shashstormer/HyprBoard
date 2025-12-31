#[cfg(test)]
use super::lexer::Lexer;
#[cfg(test)]
use super::parser::parse;
#[cfg(test)]
use std::collections::HashSet;
#[cfg(test)]
use std::path::PathBuf;

#[test]
fn test_basic_assignment() {
    let input = "
    border_size = 2
    active_opacity = 0.9
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    assert_eq!(config.lines.len(), 2);
    assert_eq!(config.lines[0].key, "border_size");
    assert_eq!(config.lines[0].value.raw, "2");
}

#[test]
fn test_categories() {
    let input = "
    general {
        gaps_in = 5
        gaps_out = 10
    }
    
    input {
        kb_layout = us
        touchpad {
            natural_scroll = true
        }
    }
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    assert_eq!(config.categories.len(), 2);
    assert_eq!(config.categories[0].name, "general");
    assert_eq!(config.categories[0].lines.len(), 2);

    assert_eq!(config.categories[1].name, "input");
    assert_eq!(config.categories[1].lines.len(), 1);
    assert_eq!(config.categories[1].categories.len(), 1);
    assert_eq!(config.categories[1].categories[0].name, "touchpad");
}

#[test]
fn test_inline_category() {
    let input = "
    input:touchpad:natural_scroll = true
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    assert_eq!(config.categories.len(), 1);
    assert_eq!(config.categories[0].name, "input");
    assert_eq!(config.categories[0].categories.len(), 1);
    assert_eq!(config.categories[0].categories[0].name, "touchpad");
    assert_eq!(
        config.categories[0].categories[0].lines[0].key,
        "natural_scroll"
    );
}

#[test]
fn test_variables_and_resolution() {
    let input = "
    $mainMod = SUPER
    bind = $mainMod, Q, exec, kitty
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    assert_eq!(config.variables.len(), 1);
    assert_eq!(config.variables.get("mainMod").unwrap().raw, "SUPER");

    let vars = config.get_var_dict();
    let resolved = config.lines[0].value.resolve(&vars);
    assert_eq!(resolved, "SUPER, Q, exec, kitty");
}

#[test]
fn test_arithmetic() {
    let input = "
    $gap = 10
    gaps_out = {{$gap * 2}}
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    let val = &config.lines[0].value;
    assert!(
        val.parts
            .iter()
            .any(|p| matches!(p, super::ast::HyprValuePart::Arithmetic(_)))
    );
}

#[test]
fn test_ifs() {
    let input = "
     $flag = 
     # hyprlang if $flag
     enabled = true
     # hyprlang endif
     
     $flag2 = 1
     # hyprlang if $flag2
     enabled2 = true
     # hyprlang endif
     ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    assert_eq!(config.lines.len(), 1);
    assert_eq!(config.lines[0].key, "enabled2");
}

#[test]
fn test_serialization() {
    let input = "
$myVar = 10

general {
    border_size = 2
}
";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    let output = config.to_string();

    assert!(output.contains("$myVar = 10"));
    assert!(output.contains("general {"));
    assert!(output.contains("border_size = 2"));
}

#[test]
fn test_real_world_config() {
    let input = r#"$mainMod = SUPER

monitor = eDP-2, 2560x1600@165.002, 0x0, 1.6
exec-once = nm-applet --indicator
gesture = 3, up, dispatcher, exec, garuda-rani
gesture = 3, down, dispatcher, exec, nwg-drawer -mb 10 -mr 10 -ml 10 -mt 10
gesture = 3, horizontal, workspace
gesture = 4, horizontal, workspace
windowrule = float, title:(^(kitty)$)
windowrule = float, title:(^(garuda-assistant)$)
windowrule = float, title:(^(garuda-boot-options)$)
windowrule = float, title:(^(garuda-boot-repair)$)
windowrule = float, title:(^(garuda-gamer)$)
windowrule = float, title:(^(garuda-network-assistant)$)
windowrule = float, title:(^(garuda-settings-manager)$)
windowrule = float, title:(^(garuda-welcome)$)
windowrule = float, title:(^(calamares)$)
bind = $mainMod SHIFT, R, exec, hyprctl reload
bind = $mainMod, 36, exec, footclient
bind = $mainMod, T, exec, footclient
bind = $mainMod, Q, killactive,
bind = $mainMod SHIFT, E, exec, nwgbar
bind = $mainMod SHIFT, 65, togglefloating,
bindr = SUPER, SUPER_L, exec, pkill wofi || wofi --normal-window --show drun --allow-images
bind = $mainMod SHIFT, D, exec, nwg-drawer -mb 10 -mr 10 -ml 10 -mt 10
bind = $mainMod, P, pseudo,
bind = $mainMod SHIFT, P, togglesplit,
bind = $mainMod, F1, exec, firedragon
bind = $mainMod, F2, exec, thunderbird
bind = $mainMod, F3, exec, thunar
bind = $mainMod, F4, exec, geany
bind = $mainMod, F5, exec, github-desktop
bind = $mainMod, F6, exec, gparted
bind = $mainMod, F7, exec, inkscape
bind = $mainMod, F8, exec, blender
bind = $mainMod, F9, exec, meld
bind = $mainMod, F10, exec, joplin-desktop
bind = $mainMod, F11, exec, snapper-tools
bind = $mainMod, F12, exec, galculator
bind = $mainMod, left, movefocus, l
bind = $mainMod, H, movefocus, l
bind = $mainMod, right, movefocus, r
bind = $mainMod, L, movefocus, r
bind = $mainMod, up, movefocus, u
bind = $mainMod, K, movefocus, u
bind = $mainMod, down, movefocus, d
bind = $mainMod, J, movefocus, d
bind = $mainMod, 1, workspace, 1
bind = $mainMod, 2, workspace, 2
bind = $mainMod, 3, workspace, 3
bind = $mainMod, 4, workspace, 4
bind = $mainMod, 5, workspace, 5
bind = $mainMod, 6, workspace, 6
bind = $mainMod, 7, workspace, 7
bind = $mainMod, 8, workspace, 8
bind = $mainMod, 9, workspace, 9
bind = $mainMod, 0, workspace, 10
bind = ALT SHIFT, 1, movetoworkspace, 1
bind = ALT SHIFT, 2, movetoworkspace, 2
bind = ALT SHIFT, 3, movetoworkspace, 3
bind = ALT SHIFT, 4, movetoworkspace, 4
bind = ALT SHIFT, 5, movetoworkspace, 5
bind = ALT SHIFT, 6, movetoworkspace, 6
bind = ALT SHIFT, 7, movetoworkspace, 7
bind = ALT SHIFT, 8, movetoworkspace, 8
bind = ALT SHIFT, 9, movetoworkspace, 9
bind = ALT SHIFT, 0, movetoworkspace, 10
bind = $mainMod SHIFT, 1, movetoworkspacesilent, 1
bind = $mainMod SHIFT, 2, movetoworkspacesilent, 2
bind = $mainMod SHIFT, 3, movetoworkspacesilent, 3
bind = $mainMod SHIFT, 4, movetoworkspacesilent, 4
bind = $mainMod SHIFT, 5, movetoworkspacesilent, 5
bind = $mainMod SHIFT, 6, movetoworkspacesilent, 6
bind = $mainMod SHIFT, 7, movetoworkspacesilent, 7
bind = $mainMod SHIFT, 8, movetoworkspacesilent, 8
bind = $mainMod SHIFT, 9, movetoworkspacesilent, 9
bind = $mainMod SHIFT, 0, movetoworkspacesilent, 10
bind = $mainMod, mouse_down, workspace, e+1
bind = $mainMod, mouse_up, workspace, e-1
bindm = $mainMod, mouse:272, movewindow
bindm = $mainMod, mouse:273, resizewindow
exec-once = wpaperd
exec-once = waybar
layerrule = blur , waybar
layerrule = ignorezero , waybar
bind = ,122, exec, pamixer --decrease 5; notify-send " Volume: "$(pamixer --get-volume) -t 500
bind = ,123, exec, pamixer --increase 5; notify-send " Volume: "$(pamixer --get-volume) -t 500
bind = ,121, exec, pamixer --toggle-mute; notify-send " Volume: Toggle-mute" -t 500
bind = ,XF86AudioMicMute, exec, pactl set-source-mute @DEFAULT_SOURCE@ toggle; notify-send "System Mic: Toggle-mute" -t 500
bind = $mainMod, O, exec, firedragon
bind = $mainMod, M, fullscreen, 1
bind = $mainMod, F, fullscreen, 0
bind = $mainMod SHIFT, F, fullscreenstate, 0 2
bind = ,232,exec,brightnessctl -c backlight set 5%-
bind = ,233,exec,brightnessctl -c backlight set +5%
bind = $mainMod SHIFT,C, exec, killall -9 wpaperd && wpaperd
bind = ,Print, exec, grim -g "$(slurp)" - | swappy -f -
bind = CTRL, Print, exec, .config/hypr/scripts/screenshot_window.sh
bind = SHIFT, Print, exec, .config/hypr/scripts/screenshot_display.sh
bind = $mainMod,R,submap,resize
submap = resize
binde = ,right,resizeactive,50 0
binde = ,L,resizeactive,50 0
binde = ,left,resizeactive,-50 0
binde = ,H,resizeactive,-50 0
binde = ,up,resizeactive,0 -50
binde = ,K,resizeactive,0 -50
binde = ,down,resizeactive,0 50
binde = ,J,resizeactive,0 50
bind = ,escape,submap,reset
submap = reset
bind = $mainMod SHIFT,up, movewindow, u
bind = $mainMod SHIFT,K, movewindow, u
bind = $mainMod SHIFT,down, movewindow, d
bind = $mainMod SHIFT,J, movewindow, d
bind = $mainMod SHIFT,left, movewindow, l
bind = $mainMod SHIFT,H, movewindow, l
bind = $mainMod SHIFT,right, movewindow, r
bind = $mainMod SHIFT,L, movewindow, r
bind = SUPER SHIFT, R, exec, ~/.config/hypr/scripts/screen_record.sh
blurls = wofi
blurls = thunar
blurls = gedit
blurls = gtk-layer-shell
blurls = catfish
windowrule = opacity 0.85 override 0.85 override,title:(^(thunar)$)
windowrule = opacity 0.85 override 0.85 override,title:(^(gedit)$)
windowrule = opacity 0.85 override 0.85 override,title:(^(catfish)$)
windowrule = stayfocused, title:(^(wofi)$)
windowrule = opacity 0.85 0.85,floating:1
windowrule = opacity 0.9,class:^(google-chrome)$, title:(.*ArchBoard.*)
windowrule = fullscreen,class:spotify
windowrule = opacity 0.9,class:^(brave-browser)$, title:(.*ArchBoard.*)
exec-once = mako
exec-once = /usr/lib/polkit-gnome/polkit-gnome-authentication-agent-1
exec-once = foot --server
exec-once = exec xrdb -load ~/.Xresources
exec-once = wl-paste --type text --watch cliphist store
exec-once = wl-paste --type image --watch cliphist store
bind = SUPER, V, exec, cliphist list | wofi --dmenu | cliphist decode | wl-copy
bind = ,172,exec,playerctl play-pause
bind = ,171,exec,playerctl next
bind = ,173,exec,playerctl previous
exec-once = apply-gsettings
bind = $mainMod SHIFT, G, exec, footclient -e ~/.local/bin/bear/implement_gum.sh enable
env = XDG_CURRENT_DESKTOP,Hyprland
env = XCURSOR_THEME,catppuccin-mocha-lavender-cursors
env = XCURSOR_SIZE,24
env = STEAM_FORCE_DESKTOPUI_SCALING,1.2
env = ELECTRON_OZONE_PLATFORM_HINT,auto
env = QT_QPA_PLATFORMTHEME,qt6ct
env = QT_STYLE_OVERRIDE,kvantum
env = QT_QPA_PLATFORM,wayland
env = XDG_SESSION_TYPE,wayland
env = XDG_SESSION_DESKTOP,Hyprland
exec-once = hypridle
exec-once = spotify-launcher
bind = ALT, Tab, cyclenext, --hist
bind = ALT, Tab, bringactivetotop,
bind = , mouse:274, exec, playerctl play-pause
bind = SUPER, L, exec, hyprlock
bind = SUPER SHIFT, S, exec, bash -c 'grim -g "$(slurp)" - | tee >(wl-copy) | swappy -f -'
bind = SUPER SHIFT, N, exec, ~/.config/hypr/scripts/nightlight.sh
bind = SUPER, E, exec, dolphin
binde = ALTSHIFT,TAB,cyclenext,prev
bind = SUPER,B,exec,google-chrome-stable

input {
    kb_layout = us
    kb_variant =
    kb_model =
    kb_options =
    kb_rules =
    numlock_by_default = true
    follow_mouse = 1
    sensitivity = 0
    touchpad {
        natural_scroll = true
        tap-to-click = true
        disable_while_typing = true
    }
}

general {
    gaps_in = 0
    gaps_out = 0
    border_size = 1
    col.active_border = 0xff1b9bc5
    col.inactive_border = rgba(595959aa)
    layout = dwindle
    resize_on_border = true
    gaps_workspaces = 0
}

decoration {
    rounding = 10
    rounding_power = 2
    inactive_opacity = 0.85
    dim_inactive = true
    blur {
        enabled = true
        size = 5
        passes = 1
    }
    shadow {
        enabled = true
        range = 4
        render_power = 3
        color = rgba(1a1a1aee)
    }
}

animations {
    enabled = true
    bezier = myBezier, 0.05, 0.9, 0.1, 1.05
    animation = windows, 1, 7, myBezier
    animation = windowsOut, 1, 7, default, popin 80%
    animation = border, 1, 10, default
    animation = fade, 1, 7, default
    animation = workspaces, 1, 6, default
}

dwindle {
    pseudotile = true
    preserve_split = false
    smart_split = true
}

master {
    new_status = master
}

misc {
    disable_hyprland_logo = true
    vfr = true
    background_color = 0xff151313
}

xwayland {
    force_zero_scaling = true
}
"#;

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    assert_eq!(config.variables.get("mainMod").unwrap().raw, "SUPER");

    let general = config
        .categories
        .iter()
        .find(|c| c.name == "general")
        .expect("general category missing");
    let border_size = general
        .lines
        .iter()
        .find(|l| l.key == "border_size")
        .expect("border_size missing");
    assert_eq!(border_size.value.raw, "1");

    let monitor = config
        .lines
        .iter()
        .find(|l| l.key == "monitor")
        .expect("monitor missing");
    assert!(monitor.value.raw.contains("eDP-2"));

    let execs = config.lines.iter().filter(|l| l.key == "exec-once").count();
    assert!(execs >= 10);

    let rules = config
        .lines
        .iter()
        .filter(|l| l.key == "windowrule")
        .count();
    assert!(rules >= 10);

    let binds = config.lines.iter().filter(|l| l.key == "bind").count();
    assert!(binds > 20);

    let output = config.to_string();
    assert!(output.contains("monitor = eDP-2"));
    assert!(output.contains("general {"));
    assert!(output.contains("border_size = 1"));
}

#[test]
fn test_lexer_token_types() {
    use super::token::TokenType;

    let input = "$var = value { } [ ] : , # comment\n";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    assert!(tokens.iter().any(|t| t.kind == TokenType::Variable));
    assert!(tokens.iter().any(|t| t.kind == TokenType::Equals));
    assert!(tokens.iter().any(|t| t.kind == TokenType::LBrace));
    assert!(tokens.iter().any(|t| t.kind == TokenType::RBrace));
    assert!(tokens.iter().any(|t| t.kind == TokenType::LBracket));
    assert!(tokens.iter().any(|t| t.kind == TokenType::RBracket));
    assert!(tokens.iter().any(|t| t.kind == TokenType::Colon));
    assert!(tokens.iter().any(|t| t.kind == TokenType::Comma));
    assert!(tokens.iter().any(|t| t.kind == TokenType::Comment));
    assert!(tokens.iter().any(|t| t.kind == TokenType::Newline));
    assert!(tokens.iter().any(|t| t.kind == TokenType::Eof));
}

#[test]
fn test_lexer_quoted_strings() {
    let input = r#"
    title = "Hello World"
    cmd = 'single quotes'
    escaped = "with \"escape\""
    "#;
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    assert_eq!(config.lines.len(), 3);
    assert_eq!(config.lines[0].value.raw, "Hello World");
    assert_eq!(config.lines[1].value.raw, "single quotes");
    assert!(config.lines[2].value.raw.contains("escape"));
}

#[test]
fn test_lexer_comments() {
    let input = "
    # This is a comment
    key = value # inline comment not parsed
    # Another comment
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    assert_eq!(config.lines.len(), 1);
    assert_eq!(config.lines[0].key, "key");
}

#[test]
fn test_lexer_escaped_hash() {
    let input = "
    color = ##ff0000
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    assert_eq!(config.lines.len(), 1);
    assert!(config.lines[0].value.raw.contains("#"));
}

#[test]
fn test_lexer_arithmetic_expression() {
    let input = "
    $base = 5
    computed = {{$base * 2 + 3}}
    nested = {{(10 - 2) / 4}}
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    let vars = config.get_var_dict();
    let resolved1 = config.lines[0].value.resolve(&vars);
    let resolved2 = config.lines[1].value.resolve(&vars);

    assert_eq!(resolved1, "13");
    assert_eq!(resolved2, "2");
}

#[test]
fn test_empty_config() {
    let input = "";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    assert_eq!(config.lines.len(), 0);
    assert_eq!(config.categories.len(), 0);
    assert_eq!(config.variables.len(), 0);
}

#[test]
fn test_whitespace_only() {
    let input = "   \n\n  \t  \n   ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    assert_eq!(config.lines.len(), 0);
}

#[test]
fn test_negated_if() {
    let input = "
    $disabled = 1
    # hyprlang if !$disabled
    should_not_appear = true
    # hyprlang endif
    
    $enabled = 
    # hyprlang if !$enabled
    should_appear = true
    # hyprlang endif
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    assert_eq!(config.lines.len(), 1);
    assert_eq!(config.lines[0].key, "should_appear");
}

#[test]
fn test_nested_ifs() {
    let input = "
    $outer = 1
    $inner = 1
    # hyprlang if $outer
    outer_line = true
    # hyprlang if $inner
    inner_line = true
    # hyprlang endif
    # hyprlang endif
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    assert_eq!(config.lines.len(), 2);
    assert!(config.lines.iter().any(|l| l.key == "outer_line"));
    assert!(config.lines.iter().any(|l| l.key == "inner_line"));
}

#[test]
fn test_empty_value() {
    let input = "
    empty_key = 
    another = value
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    assert_eq!(config.lines.len(), 2);
    assert_eq!(config.lines[0].key, "empty_key");
    assert_eq!(config.lines[0].value.raw, "");
}

#[test]
fn test_special_characters_in_values() {
    let input = r#"
    regex = ^(firefox|chrome)$
    path = /home/user/.config/hypr
    url = https:
    "#;
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    assert_eq!(config.lines.len(), 3);
    assert!(config.lines[0].value.raw.contains("firefox"));
    assert!(config.lines[1].value.raw.contains(".config"));
    assert!(config.lines[2].value.raw.contains("example.com"));
}

#[test]
fn test_keybind_formats() {
    let input = "
    bind = SUPER, Q, killactive
    binde = , right, resizeactive, 50 0
    bindr = SUPER, SUPER_L, exec, wofi
    bindm = SUPER, mouse:272, movewindow
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    assert_eq!(config.lines.len(), 4);
    assert_eq!(config.lines[0].key, "bind");
    assert_eq!(config.lines[1].key, "binde");
    assert_eq!(config.lines[2].key, "bindr");
    assert_eq!(config.lines[3].key, "bindm");
}

#[test]
fn test_color_formats() {
    let input = "
    col.active_border = 0xff1b9bc5
    col.inactive_border = rgba(595959aa)
    background = rgb(ff0000)
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    assert_eq!(config.lines.len(), 3);
    assert!(config.lines[0].value.raw.contains("0xff"));
    assert!(config.lines[1].value.raw.contains("rgba"));
    assert!(config.lines[2].value.raw.contains("rgb"));
}

#[test]
fn test_multiple_variables() {
    let input = "
    $mod = SUPER
    $term = kitty
    $browser = firefox
    bind = $mod, T, exec, $term
    bind = $mod, B, exec, $browser
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    assert_eq!(config.variables.len(), 3);

    let vars = config.get_var_dict();
    let resolved1 = config.lines[0].value.resolve(&vars);
    let resolved2 = config.lines[1].value.resolve(&vars);

    assert!(resolved1.contains("SUPER"));
    assert!(resolved1.contains("kitty"));
    assert!(resolved2.contains("firefox"));
}

#[test]
fn test_deeply_nested_categories() {
    let input = "
    level1 {
        level2 {
            level3 {
                deep_key = deep_value
            }
        }
    }
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    assert_eq!(config.categories.len(), 1);
    assert_eq!(config.categories[0].name, "level1");
    assert_eq!(config.categories[0].categories[0].name, "level2");
    assert_eq!(
        config.categories[0].categories[0].categories[0].name,
        "level3"
    );
    assert_eq!(
        config.categories[0].categories[0].categories[0].lines[0].key,
        "deep_key"
    );
}

#[test]
fn test_category_with_key() {
    let input = "
    device:my-mouse {
        sensitivity = 0.5
    }
    workspace = name:special, on-created-empty
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    assert!(config.categories.len() >= 1);
    assert_eq!(config.categories[0].name, "device");
}

#[test]
fn test_unclosed_brace_graceful() {
    let input = "
    general {
        gaps = 5
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let result = parse(tokens, PathBuf::from("."), HashSet::new());

    assert!(result.is_ok());
}

#[test]
fn test_malformed_variable() {
    let input = "
    $ = broken
    $valid = works
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    assert!(config.variables.contains_key("valid"));
}

#[test]
fn test_directive_without_if() {
    let input = "
    # hyprlang unknowndirective
    key = value
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    assert_eq!(config.lines.len(), 1);
}

#[test]
fn test_consecutive_newlines() {
    let input = "
    
    
    key1 = value1
    
    
    
    key2 = value2
    
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    assert_eq!(config.lines.len(), 2);
}

#[test]
fn test_mixed_content() {
    let input = "
    $var = test
    
    # Regular comment
    top_level = value
    
    general {
        # Comment inside category
        inner = 123
    }
    
    # hyprlang if $var
    conditional = enabled
    # hyprlang endif
    
    input:touchpad:scroll = natural
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    assert!(config.variables.contains_key("var"));
    assert!(config.lines.iter().any(|l| l.key == "top_level"));
    assert!(config.lines.iter().any(|l| l.key == "conditional"));
    assert!(config.categories.iter().any(|c| c.name == "general"));
    assert!(config.categories.iter().any(|c| c.name == "input"));
}

#[test]
fn test_window_rules() {
    let input = r#"
    windowrule = float, ^(kitty)$
    windowrule = center, ^(firefox)$
    windowrulev2 = opacity 0.8 0.8,class:^(kitty)$
    windowrulev2 = float,class:^(pavucontrol)$,title:^(Volume Control)$
    "#;
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    let rules: Vec<_> = config
        .lines
        .iter()
        .filter(|l| l.key == "windowrule" || l.key == "windowrulev2")
        .collect();

    assert_eq!(rules.len(), 4);

    assert!(rules[0].key == "windowrule");
    assert!(rules[0].value.raw.contains("float"));
    assert!(rules[0].value.raw.contains("kitty"));

    assert!(rules[2].key == "windowrulev2");
    assert!(rules[2].value.raw.contains("opacity"));
    assert!(rules[2].value.raw.contains("class"));
}

#[test]
fn test_gesture_config() {
    let input = "
    gesture {
        workspace_swipe = true
        workspace_swipe_fingers = 3
        workspace_swipe_distance = 300
    }
    
    gesture = 3, up, workspace, +1
    gesture = 3, down, workspace, -1
    gesture = 4, left, exec, wofi
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    let gesture_cat = config.categories.iter().find(|c| c.name == "gesture");
    assert!(gesture_cat.is_some());
    let gesture_cat = gesture_cat.unwrap();
    assert!(gesture_cat.lines.iter().any(|l| l.key == "workspace_swipe"));
    assert!(
        gesture_cat
            .lines
            .iter()
            .any(|l| l.key == "workspace_swipe_fingers")
    );

    let gestures: Vec<_> = config.lines.iter().filter(|l| l.key == "gesture").collect();
    assert_eq!(gestures.len(), 3);
    assert!(gestures[0].value.raw.contains("up"));
    assert!(gestures[1].value.raw.contains("down"));
    assert!(gestures[2].value.raw.contains("exec"));
}

#[test]
fn test_monitor_config() {
    let input = "
    monitor = HDMI-A-1, 1920x1080@60, 0x0, 1
    monitor = DP-1, 2560x1440@144, 1920x0, 1
    monitor = eDP-1, 1920x1080@60, auto, 1.5
    monitor = , preferred, auto, 1
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    let monitors: Vec<_> = config.lines.iter().filter(|l| l.key == "monitor").collect();
    assert_eq!(monitors.len(), 4);

    assert!(monitors[0].value.raw.contains("HDMI-A-1"));
    assert!(monitors[0].value.raw.contains("1920x1080@60"));

    assert!(monitors[1].value.raw.contains("144"));

    assert!(monitors[2].value.raw.contains("1.5"));

    assert!(monitors[3].value.raw.contains("preferred"));
}

#[test]
fn test_gradient_colors() {
    let input = "
    general {
        col.active_border = rgba(33ccffee) rgba(00ff99ee) 45deg
        col.inactive_border = rgba(595959aa)
    }
    
    decoration {
        col.shadow = rgba(1a1a1aee)
    }
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    let general = config
        .categories
        .iter()
        .find(|c| c.name == "general")
        .unwrap();

    let active_border = general
        .lines
        .iter()
        .find(|l| l.key == "col.active_border")
        .unwrap();
    assert!(active_border.value.raw.contains("rgba(33ccffee)"));
    assert!(active_border.value.raw.contains("45deg"));

    let inactive_border = general
        .lines
        .iter()
        .find(|l| l.key == "col.inactive_border")
        .unwrap();
    assert!(inactive_border.value.raw.contains("rgba(595959aa)"));
}

#[test]
fn test_exec_variants() {
    let input = "
    exec-once = waybar
    exec-once = nm-applet --indicator
    exec = swaybg -i ~/wallpaper.png
    exec-once = /usr/lib/polkit-gnome/polkit-gnome-authentication-agent-1
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    let exec_once: Vec<_> = config
        .lines
        .iter()
        .filter(|l| l.key == "exec-once")
        .collect();
    let exec: Vec<_> = config.lines.iter().filter(|l| l.key == "exec").collect();

    assert_eq!(exec_once.len(), 3);
    assert_eq!(exec.len(), 1);

    assert!(exec_once[0].value.raw.contains("waybar"));
    assert!(exec[0].value.raw.contains("swaybg"));
}

#[test]
fn test_env_parsing() {
    let input = "
    env = XCURSOR_SIZE, 24
    env = QT_QPA_PLATFORMTHEME, qt5ct
    env = MOZ_ENABLE_WAYLAND, 1
    env = XDG_CURRENT_DESKTOP, Hyprland
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    let envs: Vec<_> = config.lines.iter().filter(|l| l.key == "env").collect();
    assert_eq!(envs.len(), 4);

    assert!(envs[0].value.raw.contains("XCURSOR_SIZE"));
    assert!(envs[1].value.raw.contains("qt5ct"));
    assert!(envs[2].value.raw.contains("MOZ_ENABLE_WAYLAND"));
}

#[test]
fn test_bind_variants() {
    let input = "
    $mainMod = SUPER
    bind = $mainMod, Q, exec, kitty
    bind = $mainMod, C, killactive,
    bind = $mainMod SHIFT, E, exit,
    binde = , XF86AudioRaiseVolume, exec, pamixer -i 5
    bindr = SUPER, SUPER_L, exec, pkill wofi || wofi
    bindm = $mainMod, mouse:272, movewindow
    bindl = , switch:Lid Switch, exec, swaylock
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    let vars = config.get_var_dict();

    assert!(config.lines.iter().any(|l| l.key == "bind"));
    assert!(config.lines.iter().any(|l| l.key == "binde"));
    assert!(config.lines.iter().any(|l| l.key == "bindr"));
    assert!(config.lines.iter().any(|l| l.key == "bindm"));
    assert!(config.lines.iter().any(|l| l.key == "bindl"));

    let q_bind = config.lines.iter().find(|l| l.key == "bind").unwrap();
    let resolved = q_bind.value.resolve(&vars);
    assert!(resolved.contains("SUPER"));
}

#[test]
fn test_layerrule() {
    let input = "
    layerrule = blur, waybar
    layerrule = ignorezero, waybar
    layerrule = noanim, wofi
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    let layerrules: Vec<_> = config
        .lines
        .iter()
        .filter(|l| l.key == "layerrule")
        .collect();
    assert_eq!(layerrules.len(), 3);
    assert!(layerrules[0].value.raw.contains("blur"));
    assert!(layerrules[1].value.raw.contains("ignorezero"));
}

#[test]
fn test_submap() {
    let input = "
    bind = SUPER, R, submap, resize
    
    submap = resize
    binde = , right, resizeactive, 10 0
    binde = , left, resizeactive, -10 0
    bind = , escape, submap, reset
    submap = reset
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    let submaps: Vec<_> = config.lines.iter().filter(|l| l.key == "submap").collect();
    assert_eq!(submaps.len(), 2);
    assert!(submaps[0].value.raw.contains("resize"));
    assert!(submaps[1].value.raw.contains("reset"));
}

#[test]
fn test_workspace_rules() {
    let input = "
    workspace = 1, monitor:HDMI-A-1, default:true
    workspace = 2, monitor:DP-1
    workspace = special:scratchpad, on-created-empty:kitty
    ";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    let workspaces: Vec<_> = config
        .lines
        .iter()
        .filter(|l| l.key == "workspace")
        .collect();
    assert_eq!(workspaces.len(), 3);
    assert!(workspaces[0].value.raw.contains("monitor"));
    assert!(workspaces[2].value.raw.contains("special"));
}

#[test]
fn test_round_trip_preservation() {
    let input = r#"$mainMod = SUPER

monitor = eDP-2, 2560x1600@165.002, 0x0, 1.6
exec-once = nm-applet --indicator
gesture = 3, up, dispatcher, exec, garuda-rani
gesture = 3, down, dispatcher, exec, nwg-drawer -mb 10 -mr 10 -ml 10 -mt 10
gesture = 3, horizontal, workspace
windowrule = float, title:(^(kitty)$)
windowrule = float, title:(^(garuda-assistant)$)
bind = $mainMod SHIFT, R, exec, hyprctl reload
bind = $mainMod, 36, exec, footclient
bind = $mainMod, T, exec, footclient
bind = $mainMod, Q, killactive,
bindr = SUPER, SUPER_L, exec, pkill wofi || wofi --normal-window --show drun --allow-images
bindm = $mainMod, mouse:272, movewindow
exec-once = wpaperd
exec-once = waybar
layerrule = blur , waybar
layerrule = ignorezero , waybar
bind = ,122, exec, pamixer --decrease 5; notify-send " Volume: "$(pamixer --get-volume) -t 500
submap = resize
binde = ,right,resizeactive,50 0
binde = ,L,resizeactive,50 0
bind = ,escape,submap,reset
submap = reset
blurls = wofi
windowrule = opacity 0.85 override 0.85 override,title:(^(thunar)$)
windowrule = stayfocused, title:(^(wofi)$)
env = XDG_CURRENT_DESKTOP,Hyprland
env = XCURSOR_SIZE,24
binde = ALTSHIFT,TAB,cyclenext,prev

input {
    kb_layout = us
    kb_variant =
    numlock_by_default = true
    follow_mouse = 1
    sensitivity = 0
    touchpad {
        natural_scroll = true
        tap-to-click = true
        disable_while_typing = true
    }
}

general {
    gaps_in = 0
    gaps_out = 0
    border_size = 1
    col.active_border = 0xff1b9bc5
    col.inactive_border = rgba(595959aa)
    layout = dwindle
    resize_on_border = true
}

decoration {
    rounding = 10
    rounding_power = 2
    inactive_opacity = 0.85
    dim_inactive = true
    blur {
        enabled = true
        size = 5
        passes = 1
    }
    shadow {
        enabled = true
        range = 4
        render_power = 3
        color = rgba(1a1a1aee)
    }
}

animations {
    enabled = true
    bezier = myBezier, 0.05, 0.9, 0.1, 1.05
    animation = windows, 1, 7, myBezier
    animation = windowsOut, 1, 7, default, popin 80%
    animation = border, 1, 10, default
    animation = fade, 1, 7, default
    animation = workspaces, 1, 6, default
}

dwindle {
    pseudotile = true
    preserve_split = false
    smart_split = true
}

master {
    new_status = master
}

misc {
    disable_hyprland_logo = true
    vfr = true
    background_color = 0xff151313
}

xwayland {
    force_zero_scaling = true
}
"#;

    let mut lexer1 = Lexer::new(input);
    let tokens1 = lexer1.tokenize();
    let config1 = parse(tokens1, PathBuf::from("."), HashSet::new()).unwrap();

    assert_eq!(config1.variables.get("mainMod").unwrap().raw, "SUPER");
    assert!(config1.lines.iter().any(|l| l.key == "monitor"));
    assert!(config1.lines.iter().any(|l| l.key == "bind"));
    assert!(config1.lines.iter().any(|l| l.key == "bindr"));
    assert!(config1.lines.iter().any(|l| l.key == "bindm"));
    assert!(config1.lines.iter().any(|l| l.key == "binde"));
    assert!(config1.lines.iter().any(|l| l.key == "gesture"));
    assert!(config1.lines.iter().any(|l| l.key == "windowrule"));
    assert!(config1.lines.iter().any(|l| l.key == "exec-once"));
    assert!(config1.lines.iter().any(|l| l.key == "env"));
    assert!(config1.lines.iter().any(|l| l.key == "layerrule"));
    assert!(config1.lines.iter().any(|l| l.key == "blurls"));
    assert!(config1.lines.iter().any(|l| l.key == "submap"));

    assert!(config1.categories.iter().any(|c| c.name == "input"));
    assert!(config1.categories.iter().any(|c| c.name == "general"));
    assert!(config1.categories.iter().any(|c| c.name == "decoration"));
    assert!(config1.categories.iter().any(|c| c.name == "animations"));
    assert!(config1.categories.iter().any(|c| c.name == "dwindle"));
    assert!(config1.categories.iter().any(|c| c.name == "master"));
    assert!(config1.categories.iter().any(|c| c.name == "misc"));
    assert!(config1.categories.iter().any(|c| c.name == "xwayland"));

    let input_cat = config1
        .categories
        .iter()
        .find(|c| c.name == "input")
        .unwrap();
    assert!(input_cat.categories.iter().any(|c| c.name == "touchpad"));

    let output = config1.to_string();

    assert!(
        output.contains("$mainMod = SUPER"),
        "Variable not preserved"
    );
    assert!(
        output.contains("monitor =") && output.contains("eDP-2"),
        "Monitor not preserved"
    );
    assert!(
        output.contains("exec-once =") && output.contains("nm-applet"),
        "Exec-once not preserved"
    );
    assert!(
        output.contains("gesture =") && output.contains("garuda-rani"),
        "Gesture not preserved"
    );
    assert!(
        output.contains("windowrule =") && output.contains("float") && output.contains("kitty"),
        "Windowrule not preserved"
    );
    assert!(
        output.contains("bind =")
            && output.contains("$mainMod")
            && output.contains("hyprctl reload"),
        "Bind with variable not preserved"
    );
    assert!(
        output.contains("bindr =") && output.contains("SUPER_L"),
        "Bindr not preserved"
    );
    assert!(
        output.contains("bindm =") && output.contains("mouse:272"),
        "Bindm not preserved"
    );
    assert!(
        output.contains("binde =") && output.contains("resizeactive"),
        "Binde not preserved"
    );
    assert!(
        output.contains("layerrule =") && output.contains("blur") && output.contains("waybar"),
        "Layerrule not preserved"
    );
    assert!(
        output.contains("blurls =") && output.contains("wofi"),
        "Blurls not preserved"
    );
    assert!(
        output.contains("env =")
            && output.contains("XDG_CURRENT_DESKTOP")
            && output.contains("Hyprland"),
        "Env not preserved"
    );
    assert!(
        output.contains("submap =") && output.contains("resize"),
        "Submap not preserved"
    );

    assert!(output.contains("input {"), "Input category not preserved");
    assert!(
        output.contains("kb_layout =") && output.contains("us"),
        "kb_layout not preserved"
    );
    assert!(
        output.contains("touchpad {"),
        "Touchpad nested category not preserved"
    );
    assert!(
        output.contains("natural_scroll =") && output.contains("true"),
        "Nested option not preserved"
    );

    assert!(
        output.contains("general {"),
        "General category not preserved"
    );
    assert!(
        output.contains("border_size =") && output.contains("1"),
        "Border size not preserved"
    );
    assert!(
        output.contains("col.active_border =") && output.contains("0xff1b9bc5"),
        "Color not preserved"
    );

    assert!(
        output.contains("decoration {"),
        "Decoration category not preserved"
    );
    assert!(
        output.contains("blur {"),
        "Blur nested category not preserved"
    );
    assert!(
        output.contains("shadow {"),
        "Shadow nested category not preserved"
    );

    assert!(
        output.contains("animations {"),
        "Animations category not preserved"
    );
    assert!(
        output.contains("bezier =") && output.contains("myBezier"),
        "Bezier not preserved"
    );
    assert!(
        output.contains("animation =") && output.contains("windows"),
        "Animation not preserved"
    );

    let mut lexer2 = Lexer::new(&output);
    let tokens2 = lexer2.tokenize();
    let config2 = parse(tokens2, PathBuf::from("."), HashSet::new()).unwrap();

    assert_eq!(config2.variables.get("mainMod").unwrap().raw, "SUPER");
    assert_eq!(
        config1.lines.len(),
        config2.lines.len(),
        "Line count mismatch after round-trip"
    );
    assert_eq!(
        config1.categories.len(),
        config2.categories.len(),
        "Category count mismatch after round-trip"
    );

    let general1 = config1
        .categories
        .iter()
        .find(|c| c.name == "general")
        .unwrap();
    let general2 = config2
        .categories
        .iter()
        .find(|c| c.name == "general")
        .unwrap();
    let border1 = general1
        .lines
        .iter()
        .find(|l| l.key == "border_size")
        .unwrap();
    let border2 = general2
        .lines
        .iter()
        .find(|l| l.key == "border_size")
        .unwrap();
    assert_eq!(
        border1.value.raw, border2.value.raw,
        "Border size value mismatch after round-trip"
    );
}

#[test]
fn test_complex_bind_args() {
    let input = "bind = SUPER SHIFT, S, exec, bash -c 'grim -g \"$(slurp)\" - | tee >(wl-copy) | swappy -f -'";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let config = parse(tokens, PathBuf::from("."), HashSet::new()).unwrap();

    let bind = &config.lines[0];
    assert_eq!(bind.key, "bind");
    let expected =
        "SUPER SHIFT, S, exec, bash -c 'grim -g \"$(slurp)\" - | tee >(wl-copy) | swappy -f -'";
    assert_eq!(bind.value.raw.trim(), expected);
}
