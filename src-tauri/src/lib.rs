mod cert;
pub mod db;
mod net;
mod server;
pub mod signaling;

use std::path::PathBuf;

use server::StudioConfig;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            use tauri::Manager;
            let data_dir = app.path().app_data_dir()?;
            let dist_dir = frontend_dist();
            tauri::async_runtime::spawn(async move {
                if let Err(err) = server::run_studio(StudioConfig { data_dir, dist_dir }).await {
                    tracing::error!("studio server failed: {err:#}");
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Link Studio");
}

pub fn run_headless() {
    init_tracing();
    let data_dir = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("data");
    let dist_dir = frontend_dist();
    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
    if let Err(err) = runtime.block_on(server::run_studio(StudioConfig { data_dir, dist_dir })) {
        eprintln!("studio server failed: {err:#}");
        std::process::exit(1);
    }
}

fn frontend_dist() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dist")
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).try_init();
}
