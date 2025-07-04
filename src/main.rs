use eframe::egui;
use std::{net::IpAddr, process::Command};
use rust_mp::SERVER_PORT;

struct LauncherApp {
    text: String,
    pending_launch: bool,
    is_server: Option<bool>,
    selected_character: rust_mp::player::CharacterType,
}

impl Default for LauncherApp {
    fn default() -> Self {
        Self {
            text: String::new(),
            pending_launch: false,
            is_server: None,
            selected_character: rust_mp::player::CharacterType::Witcher,
        }
    }
}

impl LauncherApp {
    fn initiate_game_launch(&mut self, is_server: bool){
        self.is_server = Some(is_server);
        self.pending_launch = true;
    }


    fn launch_game_after_closure(&mut self,is_server: bool, ip_string: Option<String>) -> Result<(),std::io::Error>{
        Command::new("target/debug/game_main")
            .args(format!("{} {}:{} {}", is_server, ip_string.unwrap(), SERVER_PORT, self.selected_character.to_string()).split_whitespace())
            .spawn()?;
        Ok(())
    }

}

impl eframe::App for LauncherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.pending_launch {
            self.pending_launch = false;
            let is_server = self.is_server.unwrap_or(false);
            if (!String::is_empty(&self.text) && self.text.parse::<IpAddr>().is_ok()) || is_server{
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                self.launch_game_after_closure(is_server, Some(self.text.clone())).expect("Failed to launch game main");
            }
        }
        ctx.set_visuals(egui::Visuals {
            dark_mode: true,
            override_text_color: Some(egui::Color32::WHITE),
            window_fill: egui::Color32::from_gray(40),
            panel_fill: egui::Color32::from_gray(50),
            widgets: egui::style::Widgets {
                inactive: egui::style::WidgetVisuals {
                    bg_fill: egui::Color32::from_gray(60),
                    weak_bg_fill: egui::Color32::from_gray(60),
                    bg_stroke: egui::Stroke::NONE,
                    corner_radius: egui::CornerRadius { nw: (30), ne: (30), sw: (30), se: (30) },
                    fg_stroke: egui::Stroke::new(1.0, egui::Color32::WHITE),
                    expansion: 0.0,
                },
                hovered: egui::style::WidgetVisuals {
                    bg_fill: egui::Color32::from_gray(80),
                    weak_bg_fill: egui::Color32::from_gray(80),
                    bg_stroke: egui::Stroke::NONE,
                    corner_radius: egui::CornerRadius { nw: (30), ne: (30), sw: (30), se: (30) },
                    fg_stroke: egui::Stroke::new(1.0, egui::Color32::LIGHT_GRAY),
                    expansion: 0.0,
                },
                active: egui::style::WidgetVisuals {
                    bg_fill: egui::Color32::from_gray(100),
                    weak_bg_fill: egui::Color32::from_gray(100),
                    bg_stroke: egui::Stroke::NONE,
                    corner_radius: egui::CornerRadius { nw: (30), ne: (30), sw: (30), se: (30) },
                    fg_stroke: egui::Stroke::new(1.0, egui::Color32::WHITE),
                    expansion: 0.0,
                },
                open: egui::style::WidgetVisuals {
                    bg_fill: egui::Color32::from_gray(70),
                    weak_bg_fill: egui::Color32::from_gray(70),
                    bg_stroke: egui::Stroke::NONE,
                    corner_radius: egui::CornerRadius { nw: (30), ne: (30), sw: (30), se: (30) },
                    fg_stroke: egui::Stroke::new(1.0, egui::Color32::WHITE),
                    expansion: 0.0,
                },
                noninteractive: egui::style::WidgetVisuals {
                    bg_fill: egui::Color32::from_gray(70),
                    weak_bg_fill: egui::Color32::from_gray(70),
                    bg_stroke: egui::Stroke::new(1.0, egui::Color32::from_gray(120)),
                    corner_radius: egui::CornerRadius { nw: (30), ne: (30), sw: (30), se: (30) },
                    fg_stroke: egui::Stroke::new(1.0, egui::Color32::WHITE),
                    expansion: 0.0,
                },
            },
            ..Default::default()
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Game Launcher");

                if ui.add_sized([ui.available_width(), 30.0], egui::Button::new("Host")).clicked() {
                    self.initiate_game_launch(true);                    
                }

                ui.add(egui::Separator::default().spacing(10.0));

                ui.add(egui::TextEdit::singleline(&mut self.text)
                    .desired_width(f32::INFINITY)
                    .hint_text("Enter lobby IP"));
                if ui.add_sized([ui.available_width(), 30.0], egui::Button::new("Join")).clicked() {
                    self.initiate_game_launch(false);
                }

                ui.horizontal(|ui| {
                    ui.label("Select Character:");
                    egui::ComboBox::from_id_salt("character_select")
                        .selected_text(match self.selected_character {
                            rust_mp::player::CharacterType::Witcher => "Witcher",
                            rust_mp::player::CharacterType::Witch => "Witch",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.selected_character, rust_mp::player::CharacterType::Witcher, "Witcher");
                            ui.selectable_value(&mut self.selected_character, rust_mp::player::CharacterType::Witch, "Witch");
                        });
                });
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([300.0, 200.0])
            .with_resizable(false),
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "Game Launcher",
        options,
        Box::new(|_cc| Ok(Box::new(LauncherApp::default()))),
    )
}
