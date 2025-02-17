use std::{fs::{self, File}, io::Write};

use bevy::{log::{debug, error, warn}, tasks::IoTaskPool};

use crate::world::ChunkSave;

pub fn save_chunk(chunk: &ChunkSave) {
    let Ok(save_data) = ron::to_string(chunk) else {
        error!("could not save chunk!");
        return;
    };

    let file_path = format!("saves/chunk-{}.ron", chunk.idx);
    debug!("saving {}", file_path);

    #[cfg(not(target_arch = "wasm32"))]
    IoTaskPool::get()
        .spawn(async move {
            File::create(file_path)
                .and_then(|mut file| file.write(save_data.as_bytes()))
                .expect("Error while writing save file");
        })
        .detach();
}

pub fn try_load_chunk(chunk_idx: usize) -> Option<ChunkSave> {
    let file_path = format!("saves/chunk-{}.ron", chunk_idx);

    debug!("loading {}", file_path);

    #[cfg(not(target_arch = "wasm32"))]
    let Ok(contents) = fs::read_to_string(&file_path) else {
        return None;
    };

    let Ok(chunk) = ron::from_str::<ChunkSave>(&contents) else {
        warn!("Could not deserialize chunk save! corrupt? {}", file_path);
        return None;
    };

    Some(chunk)
}
