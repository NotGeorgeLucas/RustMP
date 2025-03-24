mod client;
mod server;
mod message;
mod game_main;

use eframe::egui;
use client::Client;
use server::Server;
use game_main::GameHandle;
use std::sync::{Arc,Mutex};

const COMMS_PORT:u16 =13882; 
struct LauncherApp {
    text: String,
    client: Option<Arc<Mutex<Client>>>,
    server: Option<Arc<Mutex<Server>>>,
    game_main: Option<Arc<Mutex<GameHandle>>>,
}

impl Default for LauncherApp {
    fn default() -> Self {
        Self {
            text: String::new(),
            client: None,
            server: None,
            game_main: None,
        }
    }
}

impl LauncherApp {
    
    fn launch_game(&mut self) -> Result<(),std::io::Error>{
        self.game_main = Some(Arc::new(Mutex::new(game_main::start())));
        Ok(())
    }


    fn launch_server(&mut self) -> Result<(),std::io::Error>{
        self.server = Some(Arc::new(Mutex::new(Server::new().unwrap())));

        if let Some(server) = self.server.take() {
            let server_clone = Arc::clone(&server);
            server.lock().unwrap().start(server_clone);
        }
        Ok(())
    }


    fn launch_client(&mut self,server_ip: String) -> Result<(), String> {
        if server_ip.is_empty() {
            return Err("Cannot divide by zero".to_string());
        }

        self.client = Some(Arc::new(Mutex::new(Client::new(server_ip).unwrap())));

        if let Some(client) = self.client.take() {
            let client_clone = Arc::clone(&client);
            client.lock().unwrap().start(client_clone);
        }
        Ok(())
    }
}

impl eframe::App for LauncherApp {
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
                    match self.launch_server(){
                        Ok(()) => {println!("Server started"); self.launch_game().expect("Game Launch Failed");},
                        Err(e) => println!("Error: {}",e),
                    }
                    
                }

                ui.add(egui::Separator::default().spacing(10.0));

                ui.add(egui::TextEdit::singleline(&mut self.text)
                    .desired_width(f32::INFINITY)
                    .hint_text("Enter lobby IP"));
                if ui.add_sized([ui.available_width(), 30.0], egui::Button::new("Join")).clicked() {
                    match self.launch_client(self.text.clone()){
                        Ok(())=>{println!("Client started"); self.launch_game().expect("Game Launch Failed");},
                        Err(e) => println!("Error: {}",e),
                    }
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
        Box::new(|_cc| Ok(Box::new(LauncherApp::default()))),
    )
}
