mod app;
mod fetch;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "gravitas",
        options,
        Box::new(|cc| Ok(Box::new(app::GravitasApp::new(cc)))),
    )
    .unwrap();
}

#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();

    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "gravitas-canvas",
                web_options,
                Box::new(|cc| Ok(Box::new(app::GravitasApp::new(cc)))),
            )
            .await
            .expect("failed to start eframe");
    });
}
