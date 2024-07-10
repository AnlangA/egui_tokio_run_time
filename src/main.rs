use eframe::egui;
use tokio::sync::mpsc;
use std::sync::Arc;
use parking_lot::Mutex;
use egui::*;
struct AppState {
    input: String,
    tx: mpsc::Sender<String>,
}

fn main() -> eframe::Result<()> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (tx, mut rx) = mpsc::channel(32);
    
    // 启动Tokio任务来接收和打印消息
    rt.spawn(async move {
        while let Some(message) = rx.recv().await {
            println!("Received: {}", message);
        }
    });

    let state = Arc::new(Mutex::new(AppState {
        input: String::new(),
        tx,
    }));

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "egui 向 Tokio 发送数据示例",
        options,
        Box::new(|_cc| Box::new(MyApp::new(state, rt, _cc))),
    )
}

struct MyApp {
    state: Arc<Mutex<AppState>>,
    rt: tokio::runtime::Runtime,
}

impl MyApp {
    fn new(state: Arc<Mutex<AppState>>, rt: tokio::runtime::Runtime, cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        Self { state, rt }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("egui 向 Tokio 发送数据示例");
            
            let mut state = self.state.lock();
            
            ui.horizontal(|ui| {
                ui.label("输入消息：");
                ui.text_edit_singleline(&mut state.input);
            });

            if ui.button("发送").clicked() {
                if !state.input.is_empty() {
                    let tx = state.tx.clone();
                    let message = state.input.clone();
                    
                    // 在Tokio运行时中发送消息
                    self.rt.spawn(async move {
                        if let Err(e) = tx.send(message).await {
                            eprintln!("发送错误: {}", e);
                        }
                    });
                }
            }
        });
    }
}
fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "Song".to_owned(),
        egui::FontData::from_static(include_bytes!("./font/STSong.ttf")),
    );
    fonts.families.insert(
        FontFamily::Name("Song".into()),
        vec!["Song".to_owned()],
    );
    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "Song".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("Song".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}