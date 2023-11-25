fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

/// Read data from local storage.
pub fn local_storage_get(key: &str) -> Option<String> {
    local_storage().map(|storage| storage.get_item(key).ok())??
}

/// Write data to local storage.
pub fn local_storage_set(key: &str, value: &str) {
    local_storage().map(|storage| storage.set_item(key, value));
}

#[cfg(feature = "persistence")]
pub(crate) async fn load_memory(ctx: &egui::Context, worker_options: &super::WorkerOptions) {
    if let Some(output_tx) = &worker_options.channels.output_tx {
        let app_id = worker_options.app_id.clone();
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        output_tx
            .send(super::WebRunnerOutput(
                super::WebRunnerOutputInner::StorageGet(app_id, oneshot_tx),
            ))
            .unwrap();
        if let Some(memory) = oneshot_rx.await.ok().flatten() {
            match ron::from_str(&memory) {
                Ok(memory) => {
                    ctx.memory_mut(|m| *m = memory);
                }
                Err(err) => log::warn!("Failed to parse memory RON: {err}"),
            }
        }
    }
}

#[cfg(not(feature = "persistence"))]
pub(crate) async fn load_memory(_: &egui::Context, _: &super::WorkerOptions) {}

#[cfg(feature = "persistence")]
pub(crate) fn save_memory(ctx: &egui::Context) {
    match ctx.memory(|mem| ron::to_string(mem)) {
        Ok(ron) => {
            local_storage_set("egui_memory_ron", &ron);
        }
        Err(err) => {
            log::warn!("Failed to serialize memory as RON: {err}");
        }
    }
}

#[cfg(not(feature = "persistence"))]
pub(crate) fn save_memory(_: &egui::Context) {}
