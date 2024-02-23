use luminol_web::IdbQuerySource;
use wasm_bindgen::prelude::*;

/// Read data from storage.
pub async fn storage_get(key: &str) -> Result<String, web_sys::DomException> {
    luminol_web::idb(
        "eframe.storage",
        luminol_web::IdbTransactionMode::Readonly,
        |store| store.get_owned(key),
    )
    .await?
    .await?
    .ok_or_else(|| web_sys::DomException::new_with_message("Key not found in storage").unwrap())?
    .as_string()
    .ok_or_else(|| web_sys::DomException::new_with_message("Stored value is not a string").unwrap())
}

/// Write data to storage.
pub async fn storage_set(
    key: &str,
    value: impl Into<js_sys::JsString>,
) -> Result<(), web_sys::DomException> {
    luminol_web::idb(
        "eframe.storage",
        luminol_web::IdbTransactionMode::Readwrite,
        |store| store.put_key_val_owned(key, &value.into()),
    )
    .await
    .map(|_| ())
}

#[cfg(feature = "persistence")]
pub(crate) async fn load_memory(ctx: &egui::Context) {
    if let Ok(memory) = storage_get("egui_memory_ron").await {
        match ron::from_str(&memory) {
            Ok(memory) => {
                ctx.memory_mut(|m| *m = memory);
            }
            Err(err) => log::warn!("Failed to parse memory RON: {err}"),
        }
    }
}

#[cfg(not(feature = "persistence"))]
pub(crate) async fn load_memory(_: &egui::Context) {}

#[cfg(feature = "persistence")]
pub(crate) fn save_memory(ctx: &egui::Context, channels: &super::WorkerChannels) {
    match ctx.memory(ron::to_string) {
        Ok(ron) => {
            let (oneshot_tx, oneshot_rx) = oneshot::channel();
            channels.send(super::WebRunnerOutput::StorageSet(
                String::from("egui_memory_ron"),
                ron,
                oneshot_tx,
            ));
            let _ = oneshot_rx.recv();
        }
        Err(err) => {
            log::warn!("Failed to serialize memory as RON: {err}");
        }
    }
}

#[cfg(not(feature = "persistence"))]
pub(crate) fn save_memory(_: &egui::Context, _: &super::WorkerChannels) {}
