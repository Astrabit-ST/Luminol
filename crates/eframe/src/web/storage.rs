pub(super) fn local_storage() -> Option<web_sys::Storage> {
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
pub(crate) async fn load_memory(ctx: &egui::Context, channels: &super::WorkerChannels) {
    if channels.output_tx.is_some() {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        channels.send(super::WebRunnerOutputInner::StorageGet(
            String::from("egui_memory_ron"),
            oneshot_tx,
        ));
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
pub(crate) async fn load_memory(_: &egui::Context, _: &super::WorkerChannels) {}

#[cfg(feature = "persistence")]
pub(crate) fn save_memory(ctx: &egui::Context, channels: &super::WorkerChannels) {
    match ctx.memory(|mem| ron::to_string(mem)) {
        Ok(ron) => {
            let (oneshot_tx, oneshot_rx) = oneshot::channel();
            channels.send(super::WebRunnerOutputInner::StorageSet(
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
