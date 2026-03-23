pub mod args;
pub mod chunk;
pub mod functions;

use std::io::BufRead;
use tauri::Emitter;

#[tauri::command]
fn get_request(state: tauri::State<'_, args::Request>) -> &args::Request {
    state.inner()
}

pub fn run(args: args::Args) {
    let request = args.parse_request().expect("invalid request JSON");

    tauri::Builder::default()
        .manage(request)
        .setup(|app| {
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let stdin = std::io::stdin();
                for line in stdin.lock().lines() {
                    let line = match line {
                        Ok(line) => line,
                        Err(_) => break,
                    };
                    let chunk: chunk::Chunk = match serde_json::from_str(&line) {
                        Ok(chunk) => chunk,
                        Err(err) => chunk::Chunk::Error(objectiveai::error::ResponseError {
                            code: 0,
                            message: serde_json::Value::String(err.to_string()),
                        }),
                    };
                    handle.emit("chunk", &chunk).ok();
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_request])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
