use crate::spatial_audio::{cleanup_spatial_audio, SpatialAudio, handle_emitter_audio_commands};
use crate::GameState;
use crate::{
    file_system_interaction::asset_loading::AudioAssets, spatial_audio::run_spatial_audio,
};
use bevy::prelude::*;
use bevy_kira_audio::prelude::{Audio, *};

/// Handles initialization of all sounds.
pub(crate) fn internal_audio_plugin(app: &mut App) {
    app.add_plugin(AudioPlugin)
        /* .insert_resource(AudioSettings {
            sound_capacity: 8192,
            command_capacity: 4096,
        }) */
        .add_system(init_audio.in_schedule(OnExit(GameState::Loading)))
        .insert_resource(SpatialAudio { max_distance: 25. })
        .add_system(
            run_spatial_audio
                .in_base_set(CoreSet::PostUpdate)
                .run_if(resource_exists::<SpatialAudio>()),
        )
        .add_system(
            handle_emitter_audio_commands
                .in_base_set(CoreSet::PostUpdate)
                .run_if(resource_exists::<SpatialAudio>()),
        )
        .add_system(
            cleanup_spatial_audio
                .in_base_set(CoreSet::PostUpdate)
                .run_if(resource_exists::<SpatialAudio>()),
        );
}

/* #[derive(Debug, Clone, Resource)]
pub(crate) struct AudioHandles {
    pub(crate) walking: Handle<AudioInstance>,
} */

fn init_audio(
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
    audio: Res<Audio>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    //audio.pause(); //I'm pretty sure this is what has been breaking the audio and making it behave so strangely.
/*     let walking_handle = audio
        .play(audio_assets.walking.clone())
        .looped()
        .with_volume(0.8)
        .handle();
    if let Some(walking_audio_instance) = audio_instances.get_mut(&walking_handle) {
        if let Some(error) = walking_audio_instance.pause(default()) {
            error!("pausing walking audio cause error: {:?}", error);
        }
        else {
            error!("pausing walking audio didn't cause a command error!");
        }
    }
    else {
        error!("could not find walking audio instance to pause!");
    }
    commands.insert_resource(AudioHandles {
        walking: walking_handle,
    }); */
}

pub(crate) fn create_walking_audio_handle(source_handle: Handle<AudioSource>, audio: &Res<Audio>) -> Handle<AudioInstance> {
    return audio
    .play(source_handle)
    .looped()
    .with_volume(0.8)
    .handle();
}