use eframe::egui;
use std::process::{Command, Stdio};
use std::thread;


struct MyApp {
    text: String,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            text: String::new(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                    let _server_thread = thread::spawn(|| {
                        println!("Starting server...");
                        let status = Command::new("cargo")
                            .arg("run")
                            .arg("--bin")
                            .arg("server")
                            .stdout(Stdio::inherit())
                            .stderr(Stdio::inherit())
                            .status()
                            .expect("Failed to start server");
                
                        if !status.success() {
                            eprintln!("Server failed to start");
                        }
                    });
                }
                ui.add(egui::Separator::default()
            .spacing(10.0));
                ui.add(egui::TextEdit::singleline(&mut self.text)
                    .desired_width(f32::INFINITY)
                    .hint_text("Enter lobby ID"));
                if ui.add_sized([ui.available_width(), 30.0], egui::Button::new("Join")).clicked() {
                    let _client_thread = thread::spawn(|| {
                        println!("Starting client...");
                        let status = Command::new("cargo")
                            .arg("run")
                            .arg("--bin")
                            .arg("client")
                            .stdout(Stdio::inherit())
                            .stderr(Stdio::inherit())
                            .status()
                            .expect("Failed to start client");
                
                        if !status.success() {
                            eprintln!("Client failed to start");
                        }
                    });
                }
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
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}
