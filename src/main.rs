/*
 * Project: MultiSerial
 * Version: 0.1.0
 * Author: kong
 * Description: Main application entry point and UI logic for MultiSerial.
 */

mod serial_manager;

use eframe::egui;
use serial_manager::{Charset, LineEnding, LogEntry, PortConfig, SerialManager};
use std::collections::HashMap;
use std::time::Duration;

fn main() -> eframe::Result {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([960.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "MultiSerial",
        options,
        Box::new(|cc| {
            setup_custom_style(&cc.egui_ctx);
            Ok(Box::<MultiSerialApp>::default())
        }),
    )
}

// ── Colors (Catppuccin‑Mocha inspired, eye‑friendly) ─────────────
struct C;
impl C {
    const BG_BASE:    egui::Color32 = egui::Color32::from_rgb(30, 30, 46);   // #1E1E2E
    const BG_SURFACE: egui::Color32 = egui::Color32::from_rgb(36, 36, 54);   // #242436
    const BG_OVERLAY: egui::Color32 = egui::Color32::from_rgb(49, 50, 68);   // #313244
    const TEXT:       egui::Color32 = egui::Color32::from_rgb(205, 214, 244); // #CDD6F4
    const TEXT_DIM:   egui::Color32 = egui::Color32::from_rgb(147, 153, 178); // #9399B2
    const ACCENT:     egui::Color32 = egui::Color32::from_rgb(137, 180, 250); // #89B4FA
    const GREEN:      egui::Color32 = egui::Color32::from_rgb(166, 218, 149); // #A6DA95
    const RED:        egui::Color32 = egui::Color32::from_rgb(243, 139, 168); // #F38BA8
    const YELLOW:     egui::Color32 = egui::Color32::from_rgb(249, 226, 175); // #F9E2AF
    const LAVENDER:   egui::Color32 = egui::Color32::from_rgb(180, 190, 254); // #B4BEFE
    const TEAL:       egui::Color32 = egui::Color32::from_rgb(148, 226, 213); // #94E2D5
    const PEACH:      egui::Color32 = egui::Color32::from_rgb(250, 179, 135); // #FAB387
    const MUTED_BG:   egui::Color32 = egui::Color32::from_rgb(24, 24, 37);   // darker bg
    const SIDEBAR_BG: egui::Color32 = egui::Color32::from_rgb(24, 24, 37);
    const BTN_ACTIVE: egui::Color32 = egui::Color32::from_rgb(116, 199, 236);// #74C7EC
}

const FONT_NAMES: &[&str] = &["Default", "Consolas", "Courier New", "Cascadia Mono", "Lucida Console", "Microsoft YaHei", "SimHei", "SimSun"];
const FONT_SIZES: &[f32] = &[10.0, 11.0, 12.0, 13.0, 14.0, 16.0, 18.0, 20.0, 24.0];

fn apply_font_settings(ctx: &egui::Context, font_name: &str, font_size: f32) {
    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(egui::TextStyle::Body,       egui::FontId::new(font_size, egui::FontFamily::Proportional));
    style.text_styles.insert(egui::TextStyle::Monospace,  egui::FontId::new(font_size, egui::FontFamily::Monospace));
    style.text_styles.insert(egui::TextStyle::Button,     egui::FontId::new(font_size, egui::FontFamily::Proportional));
    style.text_styles.insert(egui::TextStyle::Small,      egui::FontId::new(font_size * 0.85, egui::FontFamily::Proportional));
    style.text_styles.insert(egui::TextStyle::Heading,    egui::FontId::new(font_size * 1.5, egui::FontFamily::Proportional));
    ctx.set_style(style);

    // Try to load system font if not Default
    if font_name != "Default" {
        let mut fonts = egui::FontDefinitions::default();
        if let Ok(font_data) = load_system_font(font_name) {
            fonts.font_data.insert("custom_font".to_owned(), egui::FontData::from_owned(font_data));
            fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "custom_font".to_owned());
            fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().insert(0, "custom_font".to_owned());
            ctx.set_fonts(fonts);
        }
    }
}

fn load_system_font(name: &str) -> Result<Vec<u8>, std::io::Error> {
    let win_fonts = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
    let font_dir = std::path::Path::new(&win_fonts).join("Fonts");
    // Map common names to file names
    let filename = match name {
        "Consolas"        => "consola.ttf",
        "Courier New"     => "cour.ttf",
        "Cascadia Mono"   => "CascadiaMono.ttf",
        "Lucida Console"  => "lucon.ttf",
        "Microsoft YaHei" => "msyh.ttc",
        "SimHei"          => "simhei.ttf",
        "SimSun"          => "simsun.ttc",
        _                 => "consola.ttf",
    };
    std::fs::read(font_dir.join(filename))
}

fn setup_custom_style(ctx: &egui::Context) {
    let mut vis = egui::Visuals::dark();
    vis.panel_fill = C::BG_BASE;
    vis.window_fill = C::BG_SURFACE;
    vis.window_rounding = 10.0.into();
    vis.widgets.noninteractive.bg_fill = C::BG_SURFACE;
    vis.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, C::TEXT);
    vis.widgets.inactive.bg_fill = C::BG_OVERLAY;
    vis.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, C::TEXT);
    vis.widgets.hovered.bg_fill = egui::Color32::from_rgb(69, 71, 90);
    vis.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, C::ACCENT);
    vis.widgets.active.bg_fill = C::BG_OVERLAY;
    vis.widgets.active.fg_stroke = egui::Stroke::new(1.5, C::BTN_ACTIVE);
    vis.widgets.active.rounding  = 6.0.into();
    vis.widgets.hovered.rounding = 6.0.into();
    vis.widgets.inactive.rounding = 6.0.into();
    vis.selection.bg_fill = egui::Color32::from_rgba_premultiplied(137, 180, 250, 40);
    vis.selection.stroke = egui::Stroke::new(1.0, C::ACCENT);
    vis.extreme_bg_color = C::MUTED_BG;
    vis.faint_bg_color = C::BG_OVERLAY;
    ctx.set_visuals(vis);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 4.0);
    ctx.set_style(style);
}

// ── Application State ────────────────────────────────────────────
#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct AppSettings {
    charset: Charset,
    line_ending: LineEnding,
    show_hex: bool,
    show_timestamp: bool,
    auto_scroll: bool,
    format_json: bool,
    filter_ansi: bool,
    font_name: String,
    font_size: f32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            charset: Charset::Utf8,
            line_ending: LineEnding::CrLf,
            show_hex: false,
            show_timestamp: true,
            auto_scroll: true,
            format_json: false,
            filter_ansi: true,
            font_name: "Default".to_string(),
            font_size: 14.0,
        }
    }
}

struct MultiSerialApp {
    // Port management
    available_ports: Vec<String>,
    port_checked: HashMap<String, bool>,
    port_configs: HashMap<String, PortConfig>,
    open_ports: HashMap<String, PortInstance>,
    active_tab: String,

    // Settings dialog
    settings_port: Option<String>,

    // Global prefs
    charset: Charset,
    line_ending: LineEnding,
    show_hex: bool,
    show_timestamp: bool,
    auto_scroll: bool,
    format_json: bool,
    filter_ansi: bool,

    // Font settings
    font_name: String,
    font_size: f32,
    font_changed: bool,

    // Search
    search_text: String,
    search_visible: bool,
    search_matches: Vec<usize>,
    search_current: usize,

    // Send
    send_buffer: String,
    send_hex: bool,
    send_history: Vec<String>,

    // Misc
    status_msg: String,
}

struct PortInstance {
    manager: SerialManager,
    config: PortConfig,
}

impl Default for MultiSerialApp {
    fn default() -> Self {
        let ports = SerialManager::list_ports();
        let settings = Self::load_settings();
        
        let mut app = Self {
            available_ports: ports.clone(),
            port_checked: HashMap::new(),
            port_configs: HashMap::new(),
            open_ports: HashMap::new(),
            active_tab: String::new(),
            settings_port: None,
            charset: settings.charset,
            line_ending: settings.line_ending,
            show_hex: settings.show_hex,
            show_timestamp: settings.show_timestamp,
            auto_scroll: settings.auto_scroll,
            format_json: settings.format_json,
            filter_ansi: settings.filter_ansi,
            font_name: settings.font_name,
            font_size: settings.font_size,
            font_changed: true,
            search_text: String::new(),
            search_visible: false,
            search_matches: Vec::new(),
            search_current: 0,
            send_buffer: String::new(),
            send_hex: false,
            send_history: Vec::new(),
            status_msg: "Ready".to_string(),
        };
        for p in &ports {
            app.port_checked.insert(p.clone(), false);
            app.port_configs.insert(p.clone(), PortConfig { name: p.clone(), ..Default::default() });
        }
        app
    }
}

impl MultiSerialApp {
    fn load_settings() -> AppSettings {
        let path = std::path::Path::new("config.json");
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(settings) = serde_json::from_str(&content) {
                return settings;
            }
        }
        AppSettings::default()
    }

    fn save_settings(&self) {
        let settings = AppSettings {
            charset: self.charset,
            line_ending: self.line_ending,
            show_hex: self.show_hex,
            show_timestamp: self.show_timestamp,
            auto_scroll: self.auto_scroll,
            format_json: self.format_json,
            filter_ansi: self.filter_ansi,
            font_name: self.font_name.clone(),
            font_size: self.font_size,
        };
        if let Ok(json) = serde_json::to_string_pretty(&settings) {
            let _ = std::fs::write("config.json", json);
        }
    }

    fn refresh_ports(&mut self) {
        let ports = SerialManager::list_ports();
        for p in &ports {
            self.port_checked.entry(p.clone()).or_insert(false);
            self.port_configs.entry(p.clone()).or_insert(PortConfig { name: p.clone(), ..Default::default() });
        }
        // Remove vanished ports (only if not open)
        self.port_checked.retain(|k, _| ports.contains(k) || self.open_ports.contains_key(k));
        self.available_ports = ports;
    }

    fn open_checked_ports(&mut self) {
        let names_to_open: Vec<String> = self.port_checked.iter()
            .filter(|(k, v)| **v && !self.open_ports.contains_key(*k))
            .map(|(k, _)| k.clone())
            .collect();
        for name in names_to_open {
            if let Some(cfg) = self.port_configs.get(&name) {
                let cfg = cfg.clone();
                let mut mgr = SerialManager::new();
                match mgr.connect(&cfg, self.charset) {
                    Ok(()) => {
                        self.open_ports.insert(name.clone(), PortInstance { manager: mgr, config: cfg });
                        if self.active_tab.is_empty() {
                            self.active_tab = name.clone();
                        }
                        self.status_msg = format!("Opened {}", name);
                    }
                    Err(e) => {
                        self.status_msg = format!("Failed to open {}: {}", name, e);
                    }
                }
            }
        }
    }

    fn close_checked_ports(&mut self) {
        let names_to_close: Vec<String> = self.port_checked.iter()
            .filter(|(k, v)| **v && self.open_ports.contains_key(*k))
            .map(|(k, _)| k.clone())
            .collect();
        for name in names_to_close {
            if let Some(mut inst) = self.open_ports.remove(&name) {
                inst.manager.disconnect();
                self.status_msg = format!("Closed {}", name);
            }
        }
        if !self.open_ports.contains_key(&self.active_tab) {
            self.active_tab = self.open_ports.keys().next().cloned().unwrap_or_default();
        }
    }

    fn format_log_text(&self, entry: &LogEntry) -> String {
        if self.show_hex {
            return entry.raw.iter().map(|b| format!("{:02X} ", b)).collect::<String>();
        }
        let mut text = entry.text.clone();
        if self.filter_ansi {
            text = Charset::strip_ansi(&text);
        }
        if self.format_json {
            text = try_format_json(&text);
        }
        text
    }
}

fn try_format_json(text: &str) -> String {
    let mut starts = Vec::new();
    for (i, c) in text.char_indices() {
        if c == '{' || c == '[' { starts.push(i); }
    }
    let mut ends = Vec::new();
    for (i, c) in text.char_indices() {
        if c == '}' || c == ']' { ends.push(i); }
    }
    // Try the largest potential blocks first
    starts.sort(); 
    ends.sort_by(|a, b| b.cmp(a));
    
    for &s in &starts {
        for &e in &ends {
            if e > s {
                let json_part = &text[s..=e];
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_part) {
                    if let Ok(pretty) = serde_json::to_string_pretty(&val) {
                        return format!("{}{}{}", &text[..s], pretty, &text[e+1..]);
                    }
                }
            }
        }
    }
    text.to_string()
}

// ── UI ───────────────────────────────────────────────────────────
impl eframe::App for MultiSerialApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // High-contrast selection colors
        let mut visuals = egui::Visuals::dark();
        visuals.selection.bg_fill = egui::Color32::from_rgb(0, 120, 215); // Bright Blue
        visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
        ctx.set_visuals(visuals);

        // ── Menu Bar ─────────────────────────────────────────────
        egui::TopBottomPanel::top("menu_bar")
            .frame(egui::Frame::none()
                .fill(egui::Color32::from_rgb(40, 42, 58))
                .inner_margin(egui::Margin { left: 8.0, right: 8.0, top: 4.0, bottom: 4.0 }))
            .show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                let menu_text = |s: &str| egui::RichText::new(s).color(egui::Color32::WHITE).size(14.0).strong();

                ui.menu_button(menu_text(" File "), |ui| {
                    if ui.button("Save All Logs…").clicked() {
                        if let Some(dir) = rfd::FileDialog::new()
                            .set_title("Select folder to save logs")
                            .pick_folder()
                        {
                            for (port_name, inst) in &self.open_ports {
                                if let Ok(logs) = inst.manager.logs.lock() {
                                    let content: String = logs.iter().map(|e| {
                                        format!("[{}] {}\n", e.timestamp, e.text)
                                    }).collect();
                                    let safe_name = port_name.replace("\\", "_").replace("/", "_");
                                    let ts = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
                                    let filename = format!("{}_{}.log", safe_name, ts);
                                    let path = dir.join(&filename);
                                    let _ = std::fs::write(&path, &content);
                                }
                            }
                            self.status_msg = format!("Logs saved to {}", dir.display());
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button(menu_text(" View "), |ui| {
                    if ui.checkbox(&mut self.show_hex,       "Hex View").changed() { self.save_settings(); }
                    if ui.checkbox(&mut self.show_timestamp, "Show Timestamps").changed() { self.save_settings(); }
                    if ui.checkbox(&mut self.format_json,    "Format JSON").changed() { self.save_settings(); }
                    if ui.checkbox(&mut self.auto_scroll,    "Auto Scroll").changed() { self.save_settings(); }
                });

                ui.menu_button(menu_text(" Settings "), |ui| {
                    ui.label(egui::RichText::new("Charset").color(C::ACCENT));
                    for &cs in Charset::ALL {
                        if ui.radio_value(&mut self.charset, cs, cs.label()).clicked() {
                            self.save_settings();
                            ui.close_menu();
                        }
                    }
                    ui.separator();
                    ui.label(egui::RichText::new("Line Ending").color(C::ACCENT));
                    for &le in LineEnding::ALL {
                        if ui.radio_value(&mut self.line_ending, le, le.label()).clicked() {
                            self.save_settings();
                            ui.close_menu();
                        }
                    }
                    ui.separator();
                    ui.label(egui::RichText::new("Font").color(C::ACCENT));
                    for &name in FONT_NAMES {
                        if ui.radio_value(&mut self.font_name, name.to_string(), name).clicked() {
                            self.font_changed = true;
                            self.save_settings();
                        }
                    }
                    ui.separator();
                    ui.label(egui::RichText::new("Font Size").color(C::ACCENT));
                    for &sz in FONT_SIZES {
                        let label = format!("{:.0}px", sz);
                        if ui.radio_value(&mut self.font_size, sz, label).clicked() {
                            self.font_changed = true;
                            self.save_settings();
                        }
                    }
                });

                ui.menu_button(menu_text(" Help "), |ui| {
                    ui.label("MultiSerial v0.2");
                    ui.label("Multi-port serial monitor");
                    ui.hyperlink_to("Reference: SuperCom", "https://github.com/SuperStudio/SuperCom");
                    ui.close_menu();
                });

                // Right-aligned status info
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let open_cnt = self.open_ports.len();
                    ui.label(egui::RichText::new(format!("  {} port(s) open  | {}", open_cnt, self.charset.label())).color(C::TEXT_DIM).small());
                });
            });
        });

        // ── Status Bar ──────────────────────────────────────────
        egui::TopBottomPanel::bottom("status_bar").exact_height(24.0).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&self.status_msg).color(C::TEXT_DIM).small());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(inst) = self.open_ports.get(&self.active_tab) {
                        let rx = *inst.manager.rx_count.lock().unwrap();
                        let tx = *inst.manager.tx_count.lock().unwrap();
                        ui.label(egui::RichText::new(format!("RX: {} B   TX: {} B", rx, tx)).color(C::TEAL).small());
                    }
                });
            });
        });

        // ── Left Panel (Port List) ──────────────────────────────
        egui::SidePanel::left("port_panel")
            .default_width(260.0)
            .resizable(false)
            .frame(egui::Frame::none().fill(C::SIDEBAR_BG).inner_margin(egui::Margin::same(10.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    draw_serial_icon(ui, 20.0, C::ACCENT);
                    ui.label(egui::RichText::new("Ports").strong().size(16.0).color(C::ACCENT));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(
                            egui::Button::new(egui::RichText::new("⟳").size(14.0).color(egui::Color32::WHITE))
                                .fill(egui::Color32::from_rgb(60, 65, 80))
                                .rounding(4.0)
                        ).on_hover_text("Refresh ports").clicked() {
                            self.refresh_ports();
                        }
                    });
                });
                ui.add_space(4.0);

                // Action buttons
                ui.horizontal(|ui| {
                    if ui.add_sized([ui.available_width() / 2.0 - 4.0, 28.0],
                        egui::Button::new(egui::RichText::new("▶ Open").color(egui::Color32::WHITE))
                            .fill(egui::Color32::from_rgb(64, 160, 100))
                            .rounding(6.0))
                        .on_hover_text("Open all checked ports")
                        .clicked()
                    {
                        self.open_checked_ports();
                    }
                    if ui.add_sized([ui.available_width(), 28.0],
                        egui::Button::new(egui::RichText::new("■ Close").color(egui::Color32::WHITE))
                            .fill(egui::Color32::from_rgb(180, 70, 70))
                            .rounding(6.0))
                        .on_hover_text("Close all checked ports")
                        .clicked()
                    {
                        self.close_checked_ports();
                    }
                });
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(4.0);

                // Port list
                let port_names: Vec<String> = self.available_ports.clone();
                let open_keys: Vec<String> = self.open_ports.keys().cloned().collect();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for port_name in &port_names {
                        let is_open = open_keys.contains(port_name);
                        let checked = self.port_checked.entry(port_name.clone()).or_insert(false);

                        ui.horizontal(|ui| {
                            // Status dot
                            let dot_color = if is_open { C::GREEN } else { C::TEXT_DIM };
                            let (dot_rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                            ui.painter().circle_filled(dot_rect.center(), 4.0, dot_color);

                            // Checkbox
                            ui.checkbox(checked, "");

                            // Port name — custom styled for high contrast
                            let is_selected = self.active_tab == *port_name;
                            let (port_text_color, port_bg) = if is_selected {
                                // Active port: bright white text on solid blue bg
                                (egui::Color32::WHITE, egui::Color32::from_rgb(50, 90, 160))
                            } else if is_open {
                                (C::GREEN, egui::Color32::TRANSPARENT)
                            } else {
                                (C::TEXT, egui::Color32::TRANSPARENT)
                            };
                            let resp = ui.add(
                                egui::Button::new(
                                    egui::RichText::new(port_name).color(port_text_color).monospace().strong()
                                )
                                .fill(port_bg)
                                .rounding(4.0)
                                .min_size(egui::vec2(100.0, 22.0))
                            );
                            if resp.clicked() && is_open {
                                self.active_tab = port_name.clone();
                            }

                            // Settings button — explicit fill for visibility
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.add(
                                    egui::Button::new(egui::RichText::new("⚙").color(egui::Color32::WHITE).size(14.0))
                                        .fill(egui::Color32::from_rgb(60, 65, 80))
                                        .rounding(4.0)
                                )
                                    .on_hover_text("Port settings")
                                    .clicked()
                                {
                                    self.settings_port = Some(port_name.clone());
                                }
                            });
                        });
                    }
                });
            });

        // ── Settings Dialog (egui Window) ────────────────────────
        if let Some(port_name) = self.settings_port.clone() {
            let mut open = true;
            egui::Window::new(format!("⚙ Settings: {}", port_name))
                .open(&mut open)
                .resizable(false)
                .default_width(320.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    if let Some(cfg) = self.port_configs.get_mut(&port_name) {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Baud Rate:").color(C::TEXT));
                            egui::ComboBox::from_id_salt("baud_setting")
                                .selected_text(cfg.baud_rate.to_string())
                                .show_ui(ui, |ui| {
                                    for b in [9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600] {
                                        ui.selectable_value(&mut cfg.baud_rate, b, b.to_string());
                                    }
                                });
                        });
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Data Bits:").color(C::TEXT));
                            egui::ComboBox::from_id_salt("data_bits_setting")
                                .selected_text(cfg.data_bits.to_string())
                                .show_ui(ui, |ui| {
                                    for d in [5u8, 6, 7, 8] {
                                        ui.selectable_value(&mut cfg.data_bits, d, d.to_string());
                                    }
                                });
                        });
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Parity:").color(C::TEXT));
                            egui::ComboBox::from_id_salt("parity_setting")
                                .selected_text(cfg.parity_label())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut cfg.parity, 0, "None");
                                    ui.selectable_value(&mut cfg.parity, 1, "Odd");
                                    ui.selectable_value(&mut cfg.parity, 2, "Even");
                                });
                        });
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Stop Bits:").color(C::TEXT));
                            egui::ComboBox::from_id_salt("stop_bits_setting")
                                .selected_text(cfg.stop_bits_label())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut cfg.stop_bits, 1, "1");
                                    ui.selectable_value(&mut cfg.stop_bits, 2, "2");
                                });
                        });
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Flow Ctrl:").color(C::TEXT));
                            egui::ComboBox::from_id_salt("flow_setting")
                                .selected_text(cfg.flow_label())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut cfg.flow_control, 0, "None");
                                    ui.selectable_value(&mut cfg.flow_control, 1, "Hardware");
                                    ui.selectable_value(&mut cfg.flow_control, 2, "Software");
                                });
                        });
                    }
                });
            if !open {
                self.settings_port = None;
            }
        }

        // ── Central Panel ────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(C::BG_BASE).inner_margin(egui::Margin::same(0.0)))
            .show(ctx, |ui| {

            // Tab bar
            if !self.open_ports.is_empty() {
                let mut new_tab = self.active_tab.clone();
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    for name in self.open_ports.keys() {
                        let is_active = *name == self.active_tab;
                        let tab_text_color = if is_active { egui::Color32::WHITE } else { C::TEXT_DIM };
                        let bg = if is_active { egui::Color32::from_rgb(60, 90, 150) } else { C::BG_BASE };
                        let resp = ui.add(
                            egui::Button::new(egui::RichText::new(name).color(tab_text_color).monospace().strong())
                                .fill(bg)
                                .rounding(egui::Rounding { nw: 6.0, ne: 6.0, sw: 0.0, se: 0.0 })
                        );
                        if resp.clicked() {
                            new_tab = name.clone();
                        }
                    }
                });
                self.active_tab = new_tab;

                // Toolbar
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    if ui.add(
                        egui::Button::new(egui::RichText::new("🗑 Clear").color(egui::Color32::WHITE))
                            .fill(egui::Color32::from_rgb(140, 60, 60))
                            .rounding(4.0)
                    ).clicked() {
                        if let Some(inst) = self.open_ports.get(&self.active_tab) {
                            inst.manager.logs.lock().unwrap().clear();
                        }
                    }
                    ui.separator();
                    if ui.checkbox(&mut self.show_timestamp, egui::RichText::new("Time").color(C::TEXT).small()).changed() { self.save_settings(); }
                    if ui.checkbox(&mut self.show_hex,       egui::RichText::new("HEX").color(C::TEXT).small()).changed() { self.save_settings(); }
                    if ui.checkbox(&mut self.format_json,    egui::RichText::new("JSON").color(C::TEXT).small()).changed() { self.save_settings(); }
                    if ui.checkbox(&mut self.auto_scroll,    egui::RichText::new("Auto↓").color(C::TEXT).small()).changed() { self.save_settings(); }
                    if ui.checkbox(&mut self.filter_ansi,    egui::RichText::new("ANSI").color(C::TEXT).small()).changed() { self.save_settings(); }
                    ui.separator();
                    if ui.add(
                        egui::Button::new(egui::RichText::new("🔍 Search").color(C::TEXT))
                            .fill(if self.search_visible { C::BG_OVERLAY } else { egui::Color32::TRANSPARENT })
                            .rounding(4.0)
                    ).clicked() {
                        self.search_visible = !self.search_visible;
                    }
                });

                // Ctrl+F shortcut
                if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::F)) {
                    self.search_visible = !self.search_visible;
                }

                // Search bar
                if self.search_visible {
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("🔍").color(C::ACCENT).size(16.0));
                        
                        // Force a high-contrast dark background for the search input
                        let mut search_resp = None;
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(30, 30, 45))
                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 100)))
                            .inner_margin(egui::Margin::symmetric(4.0, 2.0))
                            .rounding(4.0)
                            .show(ui, |ui| {
                                let resp = ui.add(
                                    egui::TextEdit::singleline(&mut self.search_text)
                                        .hint_text("Search…")
                                        .desired_width(250.0)
                                        .font(egui::TextStyle::Monospace)
                                        .text_color(egui::Color32::WHITE)
                                        .frame(false) // Disable inner frame to use our own
                                );
                                search_resp = Some(resp);
                            });
                        let search_resp = search_resp.unwrap();
                        
                        // Recalculate matches when text changes
                        if search_resp.changed() {
                            self.search_matches.clear();
                            self.search_current = 0;
                            if !self.search_text.is_empty() {
                                if let Some(inst) = self.open_ports.get(&self.active_tab) {
                                    if let Ok(logs) = inst.manager.logs.lock() {
                                        let query = self.search_text.to_lowercase();
                                        for (i, entry) in logs.iter().enumerate() {
                                            if entry.text.to_lowercase().contains(&query) {
                                                self.search_matches.push(i);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        let total = self.search_matches.len();
                        let cur_display = if total > 0 { self.search_current + 1 } else { 0 };
                        ui.label(egui::RichText::new(format!(" {}/{} ", cur_display, total))
                            .color(egui::Color32::WHITE).monospace().strong());

                        if ui.add(
                            egui::Button::new(egui::RichText::new("▲ Prev").color(egui::Color32::WHITE))
                                .fill(egui::Color32::from_rgb(60, 100, 120))
                                .rounding(4.0)
                        ).clicked() && total > 0 {
                            if self.search_current == 0 {
                                self.search_current = total - 1;
                            } else {
                                self.search_current -= 1;
                            }
                        }
                        if ui.add(
                            egui::Button::new(egui::RichText::new("▼ Next").color(egui::Color32::WHITE))
                                .fill(egui::Color32::from_rgb(60, 100, 120))
                                .rounding(4.0)
                        ).clicked() && total > 0 {
                            self.search_current = (self.search_current + 1) % total;
                        }
                        // Enter = next, Shift+Enter = prev
                        if search_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            if total > 0 {
                                if ui.input(|i| i.modifiers.shift) {
                                    if self.search_current == 0 { self.search_current = total - 1; } else { self.search_current -= 1; }
                                } else {
                                    self.search_current = (self.search_current + 1) % total;
                                }
                            }
                            search_resp.request_focus();
                        }
                        if ui.add(
                            egui::Button::new(egui::RichText::new(" ✕ ").color(egui::Color32::WHITE))
                                .fill(egui::Color32::from_rgb(160, 60, 60))
                                .rounding(4.0)
                        ).clicked() {
                            self.search_visible = false;
                            self.search_text.clear();
                            self.search_matches.clear();
                        }
                    });
                }
                ui.separator();

                // Log area
                if let Some(inst) = self.open_ports.get(&self.active_tab) {
                    let entries: Vec<LogEntry> = inst.manager.logs.lock().unwrap().clone();

                    let available = ui.available_height() - 44.0;
                    let search_highlight = if !self.search_text.is_empty() && self.search_visible {
                        Some(self.search_text.clone())
                    } else { None };
                    let highlight_idx = if !self.search_matches.is_empty() {
                        Some(self.search_matches.get(self.search_current).copied().unwrap_or(0))
                    } else { None };

                    let scroll = egui::ScrollArea::vertical()
                        .stick_to_bottom(self.auto_scroll && highlight_idx.is_none())
                        .max_height(available);

                    scroll.show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            for (idx, entry) in entries.iter().enumerate() {
                                let text = self.format_log_text(entry);
                                let mut color = log_color(&entry.text);

                                // Highlight search match
                                let is_current_match = highlight_idx == Some(idx);
                                let is_any_match = search_highlight.as_ref()
                                    .map(|q| entry.text.to_lowercase().contains(&q.to_lowercase()))
                                    .unwrap_or(false);

                                // Use solid opaque backgrounds for strong contrast
                                let (bg, text_color) = if is_current_match {
                                    (Some(egui::Color32::from_rgb(35, 60, 110)), egui::Color32::WHITE)
                                } else if is_any_match {
                                    (Some(egui::Color32::from_rgb(80, 65, 20)), egui::Color32::from_rgb(255, 240, 200))
                                } else {
                                    (None, color)
                                };

                                let frame = if let Some(bg_color) = bg {
                                    egui::Frame::none().fill(bg_color).inner_margin(egui::Margin { left: 4.0, right: 4.0, top: 1.0, bottom: 1.0 })
                                } else {
                                    egui::Frame::none()
                                };

                                frame.show(ui, |ui| {
                                    if self.show_timestamp {
                                        ui.horizontal_wrapped(|ui| {
                                            ui.label(egui::RichText::new(&entry.timestamp).monospace().color(if bg.is_some() { egui::Color32::from_rgb(180, 190, 210) } else { C::TEXT_DIM }).small());
                                            ui.label(egui::RichText::new(&text).monospace().color(text_color));
                                        });
                                    } else {
                                        ui.add(egui::Label::new(
                                            egui::RichText::new(&text).monospace().color(text_color)
                                        ).wrap());
                                    }
                                });

                                // Scroll to current match
                                if is_current_match {
                                    ui.scroll_to_cursor(Some(egui::Align::Center));
                                }
                            }
                        });
                }

                ui.separator();

                // Send area
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    ui.checkbox(&mut self.send_hex, egui::RichText::new("HEX").color(C::TEXT_DIM).small());

                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut self.send_buffer)
                            .hint_text("Type command and press Enter…")
                            .desired_width(ui.available_width() - 80.0)
                            .font(egui::TextStyle::Monospace)
                    );

                    let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    let send_clicked = ui.add(
                        egui::Button::new(egui::RichText::new("Send ▶").color(egui::Color32::WHITE))
                            .fill(C::ACCENT)
                            .rounding(6.0)
                    ).clicked();

                    if (enter || send_clicked) && !self.send_buffer.is_empty() {
                        let data = if self.send_hex {
                            hex_to_bytes(&self.send_buffer)
                        } else {
                            let ending = self.line_ending.as_str();
                            format!("{}{}", self.send_buffer, ending).into_bytes()
                        };
                        if let Some(inst) = self.open_ports.get_mut(&self.active_tab) {
                            match inst.manager.send(&data) {
                                Ok(()) => {
                                    if !self.send_history.contains(&self.send_buffer) {
                                        self.send_history.push(self.send_buffer.clone());
                                        if self.send_history.len() > 20 {
                                            self.send_history.remove(0);
                                        }
                                    }
                                    self.status_msg = format!("Sent {} bytes", data.len());
                                }
                                Err(e) => { self.status_msg = format!("Send error: {}", e); }
                            }
                        }
                        self.send_buffer.clear();
                        resp.request_focus();
                    }
                });
            } else {
                // Empty state
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(ui.available_height() / 3.0);
                        ui.horizontal(|ui| {
                            ui.add_space(ui.available_width() / 2.0 - 24.0);
                            draw_serial_icon(ui, 48.0, C::ACCENT);
                        });
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("MultiSerial").size(28.0).color(egui::Color32::WHITE).strong());
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("Select ports in the sidebar and click Open to start monitoring")
                            .size(14.0).color(C::TEXT_DIM));
                    });
                });
            }
        });

        // Apply font settings if changed
        if self.font_changed {
            apply_font_settings(ctx, &self.font_name, self.font_size);
            self.font_changed = false;
        }

        ctx.request_repaint_after(Duration::from_millis(16));
    }
}

// ── Helpers ──────────────────────────────────────────────────────
fn draw_serial_icon(ui: &mut egui::Ui, size: f32, color: egui::Color32) {
    let (rect, _) = ui.allocate_at_least(egui::vec2(size, size), egui::Sense::hover());
    let painter = ui.painter();
    let center = rect.center();
    let r = size * 0.45;
    
    // Abstract "Flow" icon - extremely simple and modern
    let p1 = center + egui::vec2(-r, 0.0);
    let p2 = center + egui::vec2(r, 0.0);
    
    // Two circles connected by a thin pulse line
    painter.circle_stroke(center, r * 0.8, egui::Stroke::new(1.2, color.gamma_multiply(0.5)));
    
    let pulse = vec![
        p1,
        center + egui::vec2(-r * 0.2, 0.0),
        center + egui::vec2(0.0, -r * 0.5),
        center + egui::vec2(r * 0.2, 0.0),
        p2,
    ];
    painter.add(egui::Shape::line(pulse, egui::Stroke::new(1.8, color)));
    painter.circle_filled(p1, 2.5, color);
    painter.circle_filled(p2, 2.5, color);
}

fn log_color(text: &str) -> egui::Color32 {
    let t = text.to_uppercase();
    if t.contains("ERROR") || t.contains("FAIL") || t.contains("PANIC") {
        C::RED
    } else if t.contains("WARN") {
        C::YELLOW
    } else if t.contains("OK") || t.contains("DONE") || t.contains("SUCCESS") {
        C::GREEN
    } else if t.contains("INFO") {
        C::LAVENDER
    } else {
        C::TEXT
    }
}

fn hex_to_bytes(hex: &str) -> Vec<u8> {
    hex.split_whitespace()
        .filter_map(|s| u8::from_str_radix(s, 16).ok())
        .collect()
}
