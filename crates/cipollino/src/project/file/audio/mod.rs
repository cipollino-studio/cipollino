
use std::{path::PathBuf, sync::Arc, collections::HashSet};

use crate::{audio::generate::MAX_AUDIO_CHANNELS, project::{saveload::load::LoadingMetadata, AssetPtr, Project}};

use self::reader::read_samples;

use super::{FileList, FilePtr, FileType};

pub const SAMPLES_PER_VOLUME_SUM: usize = 100;
mod reader;

#[derive(Clone)]
pub struct AudioFile {
    pub samples: Arc<Vec<[f32; MAX_AUDIO_CHANNELS]>>,
    pub volumes: Vec<f32>
}

impl FileType for AudioFile {

    fn load(project: &Project, path: PathBuf) -> Result<Self, String> {
        match path.extension().ok_or("No extension")?.to_str().unwrap() {
            "mp3" => {
                Ok(Self::new(read_samples(path, project.sample_rate as u32)?))
            },
            _ => Err("Invalid extension.".to_owned()) 
        }
    }

    fn get_list(project: &Project) -> &FileList<Self> {
        &project.audio_files 
    }

    fn get_list_mut(project: &mut Project) -> &mut FileList<Self> {
        &mut project.audio_files 
    }

    fn list_in_folder(folder: &crate::project::folder::Folder) -> &Vec<super::FilePtr<Self>> {
        &folder.audios
    }

    fn list_in_folder_mut(folder: &mut crate::project::folder::Folder) -> &mut Vec<super::FilePtr<Self>> {
        &mut folder.audios
    }

    fn list_in_loading_metadata(metadata: &LoadingMetadata) -> &HashSet<FilePtr<Self>> {
        &metadata.audio_file_ptrs
    }

    fn list_in_loading_metadata_mut(metadata: &mut LoadingMetadata) -> &mut HashSet<FilePtr<Self>> { 
        &mut metadata.audio_file_ptrs
    }

    fn make_asset_ptr(ptr: &FilePtr<Self>) -> AssetPtr {
        AssetPtr::Audio(ptr.clone())
    }

    fn icon() -> &'static str {
        egui_phosphor::regular::SPEAKER_HIGH
    }

}

impl AudioFile {

    pub fn new(samples: Vec<[f32; 2]>) -> Self {
        let mut volumes = Vec::new();
        for i in 0..samples.len() / SAMPLES_PER_VOLUME_SUM {
            let mut volume = 0.0;
            for j in 0..SAMPLES_PER_VOLUME_SUM {
                for c in 0..MAX_AUDIO_CHANNELS {
                    volume += samples[i * SAMPLES_PER_VOLUME_SUM + j][c].abs();
                }
            }
            volume /= (MAX_AUDIO_CHANNELS * SAMPLES_PER_VOLUME_SUM) as f32;
            volumes.push(volume);
        }

        Self {
            samples: Arc::new(samples),
            volumes
        }
    }


}
