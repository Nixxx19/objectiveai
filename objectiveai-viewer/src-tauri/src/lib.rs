pub mod args;

#[tauri::command]
fn get_request(state: tauri::State<'_, args::Request>) -> &args::Request {
    state.inner()
}

pub fn run(args: args::Args) {
    let request = args.parse_request().expect("invalid request JSON");

    tauri::Builder::default()
        .manage(request)
        .invoke_handler(tauri::generate_handler![get_request])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
