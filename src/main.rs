use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use eframe::egui;
use tokio::io::{self, AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() {
    // 创建一个异步通道
    let (tx, mut rx) = mpsc::channel(32);

    // 启动一个 tokio 任务来读取终端输入
    tokio::spawn(async move {
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin).lines();
        while let Some(line) = reader.next_line().await.unwrap() {
            let a = line.clone() + "aaa";
            //print!("{}",a);
            if tx.send(line).await.is_err() {
                break;
            }
        }
    });

    // 共享接收器
    let rx = Arc::new(Mutex::new(rx));

    // 运行 egui 应用
    let app = MyApp {
        rx: Arc::clone(&rx),
        messages: vec![],
    };
    let native_options = eframe::NativeOptions::default();
    match eframe::run_native(
        "test",
        native_options,
        Box::new(|_cc| Ok(Box::new(app))),
    ) {
        Ok(_) => {}
        Err(e) => {
            println!("{:?}", e)
        }
    }
}

struct MyApp {
    rx: Arc<Mutex<mpsc::Receiver<String>>>,
    messages: Vec<String>,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 检查是否有新消息
        let mut rx = self.rx.lock().unwrap();
        while let Ok(msg) = rx.try_recv() {
            self.messages.push(msg);
        }
        drop(rx); // 释放锁

        // 处理消息以删除倒数第二个换行符及其之前的内容
        let combined_messages: String = self.messages.join("\n");
        let processed_message = remove_second_last_newline(&combined_messages);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Messages from Terminal:");
            ui.label(processed_message);
        });
    }
}

fn remove_second_last_newline(text: &str) -> String {
    let mut parts: Vec<&str> = text.split('\n').collect();
    if parts.len() > 1 {
        parts.remove(parts.len() - 2);
    }
    parts.join("\n")
}
