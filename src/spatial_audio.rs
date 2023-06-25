use bevy::asset::{Assets, Handle};
use bevy::ecs::component::Component;
use bevy::prelude::{
    Bundle, Commands, Entity, GlobalTransform, Query, Res, ResMut, Resource, With,
};
use bevy::transform::TransformBundle;
use bevy_kira_audio::{Audio, AudioInstance, AudioSource, AudioTween};

/* /// Component for audio emitters
///
/// Add [`Handle<AudioInstance>`]s to control their pan and volume based on emitter
/// and receiver positions.
#[derive(Component, Default)]
pub struct CustomAudioEmitter {
    /// Audio instances that are played by this emitter
    ///
    /// The same instance should only be on one emitter.
    pub instances: Vec<Handle<AudioInstance>>,
} */

#[derive(Bundle)]
pub(crate) struct DisposableAudioEmitterBundle {
    pub(crate) emitter_handle: AudioEmitterHandle,
    pub(crate) disposable_emitter: DisposableAudioEmitter,
    #[bundle]
    pub(crate) tranform_bundle: TransformBundle,
}

impl DisposableAudioEmitterBundle {
    pub(crate) fn new(instance: Handle<AudioInstance>, transform_bundle: TransformBundle) -> Self {
        Self {
            emitter_handle: AudioEmitterHandle {
                instance: Some(instance),
            },
            disposable_emitter: DisposableAudioEmitter::default(),
            tranform_bundle: transform_bundle,
        }
    }
}

/// Component for audio emitters
///
/// Add [`Handle<AudioInstance>`]s to control their pan and volume based on emitter
/// and receiver positions.
#[derive(Component, Default)]
pub struct AudioEmitterHandle {
    /// Audio instances that are played by this emitter
    ///
    /// The same instance should only be on one emitter.
    pub instance: Option<Handle<AudioInstance>>,
}

#[derive(Component, Default)]
pub struct DisposableAudioEmitter {}

#[derive(Bundle)]
pub(crate) struct LoopAudioEmitterBundle {
    pub(crate) emitter_handle: AudioEmitterHandle,
    pub(crate) loop_emitter: LoopAudioEmitter,
}

impl LoopAudioEmitterBundle {
    pub(crate) fn new(
        source_handle: Handle<AudioSource>,
        setup_fn: fn(Handle<AudioSource>, &Res<Audio>) -> Handle<AudioInstance>,
    ) -> Self {
        Self {
            emitter_handle: AudioEmitterHandle { instance: None },
            loop_emitter: LoopAudioEmitter {
                source_handle,
                setup_fn,
                pending_emitter_commands: Vec::<EmitterPlayerCommand>::new(),
                playback_state: None,
            },
        }
    }
}

#[derive(Component)]
pub struct LoopAudioEmitter {
    /// Audio instances that are played by this emitter
    ///
    /// The same instance should only be on one emitter.
    //pub instance: Option<Handle<AudioInstance>>,
    pub source_handle: Handle<AudioSource>,
    pub setup_fn: fn(Handle<AudioSource>, &Res<Audio>) -> Handle<AudioInstance>,
    pub pending_emitter_commands: Vec<EmitterPlayerCommand>,
    pub playback_state: Option<bevy_kira_audio::PlaybackState>,
}

impl LoopAudioEmitter {
    fn play_command(
        &self,
        tween: AudioTween,
        instance: &mut Option<Handle<AudioInstance>>,
        audio: &Res<Audio>,
        audio_instances: &mut ResMut<Assets<AudioInstance>>,
    ) {
        //play or create new instance of audio and then play it.
        let mut is_new_instance_required = false;
        if let Some(audio_handle) = instance {
            if let Some(audio_instance) = audio_instances.get_mut(&audio_handle) {
                audio_instance.resume(tween);
            } else {
                is_new_instance_required = true;
            };
        } else {
            is_new_instance_required = true;
        }

        if is_new_instance_required {
            *instance = Some(self.setup_fn(self.source_handle.clone(), audio));
        }
    }

    fn resume_command(
        &self,
        tween: AudioTween,
        instance: &Option<Handle<AudioInstance>>,
        audio_instances: &mut ResMut<Assets<AudioInstance>>,
    ) {
        //resume audio if it exists. otherwise do nothing.
        if let Some(audio_handle) = &instance {
            if let Some(audio_instance) = audio_instances.get_mut(audio_handle) {
                audio_instance.resume(tween);
            }
        }
    }

    fn pause_command(
        &self,
        tween: AudioTween,
        instance: &Option<Handle<AudioInstance>>,
        audio_instances: &mut ResMut<Assets<AudioInstance>>,
    ) {
        //pause audio if it exists. otherwise do nothing.
        if let Some(audio_handle) = &instance {
            if let Some(audio_instance) = audio_instances.get_mut(audio_handle) {
                audio_instance.pause(tween);
            }
        }
    }

    fn stop_command(
        &self,
        tween: AudioTween,
        instance: &Option<Handle<AudioInstance>>,
        audio_instances: &mut ResMut<Assets<AudioInstance>>,
    ) {
        //stop audio if it exists. otherwise do nothing.
        if let Some(audio_handle) = &instance {
            if let Some(audio_instance) = audio_instances.get_mut(audio_handle) {
                audio_instance.stop(tween);
            }
        }
    }

    fn setup_fn(
        &self,
        source_handle: Handle<AudioSource>,
        audio: &Res<Audio>,
    ) -> Handle<AudioInstance> {
        (self.setup_fn)(source_handle, audio)
    }

    fn process_commands(
        &mut self,
        instance: &mut Option<Handle<AudioInstance>>,
        audio: &Res<Audio>,
        audio_instances: &mut ResMut<Assets<AudioInstance>>,
    ) {
        self.pending_emitter_commands.reverse();
        let commands: Vec<EmitterPlayerCommand> = self.pending_emitter_commands.drain(..).collect();
        for command in commands {
            self.process_command(instance, command, audio, audio_instances);
        }
    }

    fn process_command(
        &mut self,
        instance: &mut Option<Handle<AudioInstance>>,
        command: EmitterPlayerCommand,
        audio: &Res<Audio>,
        audio_instances: &mut ResMut<Assets<AudioInstance>>,
    ) {
        match command {
            EmitterPlayerCommand::Play(tween) => {
                self.play_command(tween, instance, audio, audio_instances)
            }
            EmitterPlayerCommand::Resume(tween) => {
                self.resume_command(tween, instance, audio_instances)
            }
            EmitterPlayerCommand::Pause(tween) => {
                self.pause_command(tween, instance, audio_instances)
            }
            EmitterPlayerCommand::Stop(tween) => {
                self.stop_command(tween, instance, audio_instances)
            }
        }
    }

    pub(crate) fn play(&mut self, tween: AudioTween) {
        self.pending_emitter_commands
            .push(EmitterPlayerCommand::Play(tween));
    }

    pub(crate) fn resume(&mut self, tween: AudioTween) {
        self.pending_emitter_commands
            .push(EmitterPlayerCommand::Resume(tween));
    }

    pub(crate) fn pause(&mut self, tween: AudioTween) {
        self.pending_emitter_commands
            .push(EmitterPlayerCommand::Pause(tween));
    }

    pub(crate) fn stop(&mut self, tween: AudioTween) {
        self.pending_emitter_commands
            .push(EmitterPlayerCommand::Stop(tween));
    }
}

pub enum EmitterPlayerCommand {
    Play(AudioTween),
    Resume(AudioTween),
    Pause(AudioTween),
    Stop(AudioTween),
}

/// Component for the audio receiver
///
/// Most likely you will want to add this component to your player or you camera.
/// The entity needs a [`Transform`] and [`GlobalTransform`]. The view direction of the [`GlobalTransform`]
/// will
#[derive(Component)]
pub struct CustomAudioReceiver;

/// Configuration resource for spatial audio
///
/// If this resource is not added to the ECS, spatial audio is not applied.
#[derive(Resource)]
pub struct SpatialAudio {
    /// The volume will change from `1` at distance `0` to `0` at distance `max_distance`
    pub max_distance: f32,
}

impl SpatialAudio {
    pub(crate) fn update(
        &self,
        receiver_transform: &GlobalTransform,
        emitters: &Query<(&GlobalTransform, &AudioEmitterHandle)>,
        audio_instances: &mut Assets<AudioInstance>,
    ) {
        //this causes some odd behaviors because it only takes the right ear into account.
        for (emitter_transform, emitter) in emitters {
            let sound_path = emitter_transform.translation() - receiver_transform.translation();
            let volume = (1. - sound_path.length() / self.max_distance)
                .clamp(0., 1.)
                .powi(2);

            let right_ear_angle = receiver_transform.right().angle_between(sound_path);
            let panning = (right_ear_angle.cos() + 1.) / 2.;

            if let Some(instance) = &emitter.instance {
                if let Some(instance) = audio_instances.get_mut(instance) {
                    instance.set_volume(volume as f64, AudioTween::default());
                    instance.set_panning(panning as f64, AudioTween::default());
                }
            }
        }
    }
}

pub(crate) fn run_spatial_audio(
    spacial_audio: Res<SpatialAudio>,
    receiver: Query<&GlobalTransform, With<CustomAudioReceiver>>,
    emitters: Query<(&GlobalTransform, &AudioEmitterHandle)>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    if let Some(receiver_transform) = receiver.iter().next() {
        spacial_audio.update(receiver_transform, &emitters, &mut audio_instances);
    }
}

pub(crate) fn cleanup_spatial_audio(
    emitters: Query<(Entity, &AudioEmitterHandle), With<DisposableAudioEmitter>>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut commands: Commands,
) {
    for (entity, emitter) in &emitters {
        let mut should_despawn = false;
        if let Some(emitter_handle) = &emitter.instance {
            if let Some(instance) = audio_instances.get_mut(emitter_handle) {
                match instance.state() {
                    bevy_kira_audio::PlaybackState::Paused { position: _ } => (),
                    bevy_kira_audio::PlaybackState::Pausing { position: _ } => (),
                    bevy_kira_audio::PlaybackState::Playing { position: _ } => (),
                    bevy_kira_audio::PlaybackState::Queued => (),
                    bevy_kira_audio::PlaybackState::Stopped => {
                        should_despawn = true;
                    }
                    bevy_kira_audio::PlaybackState::Stopping { position: _ } => (),
                }
            } else {
                should_despawn = true;
            }
        } else {
            should_despawn = true;
        }

        if should_despawn {
            if let Some(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.despawn();
            }
        }
    }
}

pub(crate) fn handle_emitter_audio_commands(
    mut emitters: Query<(Entity, &mut AudioEmitterHandle, &mut LoopAudioEmitter)>,
    audio: Res<Audio>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    for (_entity, mut emitter_handle, mut emitter) in &mut emitters {
        emitter.process_commands(&mut emitter_handle.instance, &audio, &mut audio_instances);
        emitter.playback_state = if let Some(handle) = &emitter_handle.instance {
            if let Some(instance) = audio_instances.get_mut(handle) {
                Some(instance.state())
            } else {
                None
            }
        } else {
            None
        };
    }
}
