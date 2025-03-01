use crate::power_state::Loading;
use crate::settings::preferences::MascotPreferencesResource;
use crate::util::{create_dir_all_if_need, models_dir, remove_mystery_file_if_exists};
use crate::vrm::loader::VrmHandle;
use crate::vrm::VrmPath;
use bevy::app::{App, PreStartup, Update};
use bevy::asset::io::file::FileWatcher;
use bevy::asset::io::AssetSourceEvent;
use bevy::asset::{Assets, Handle, LoadedFolder};
use bevy::log::error;
use bevy::prelude::{on_event, AssetServer, Commands, Component, Entity, Event, EventWriter, IntoSystemConfigs, Local, Plugin, Query, Reflect, Res};
use crossbeam::channel::Receiver;
use std::path::PathBuf;
use std::time::Duration;

pub struct VrmLoadPlugin;

impl Plugin for VrmLoadPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<ModelsFolderHandle>()
            .add_event::<RequestLoadModels>()
            .add_systems(PreStartup, (
                start_load_models_folder,
                // start_watching,
            ).chain())
            .add_systems(Update, (
                prepare_initial_loading,
                load_models.run_if(on_event::<RequestLoadModels>),
                // receive_events,
            ));
    }
}

#[derive(Component)]
struct ModelFilesWatcher {
    _watcher: FileWatcher,
    receiver: Receiver<AssetSourceEvent>,
}

#[derive(Component, Reflect)]
struct ModelsFolderHandle(Handle<LoadedFolder>);

#[derive(Event)]
struct RequestLoadModels;

fn start_load_models_folder(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    remove_mystery_file_if_exists(&models_dir());
    commands.spawn((
        Loading,
        ModelsFolderHandle(asset_server.load_folder("models")),
    ));
}

fn prepare_initial_loading(
    mut commands: Commands,
    mut ew: EventWriter<RequestLoadModels>,
    mut loaded: Local<bool>,
    folders: Res<Assets<LoadedFolder>>,
    handle: Query<(Entity, &ModelsFolderHandle)>,
) {
    if *loaded {
        return;
    }
    let (entity, handle) = handle.single();
    if folders.contains(handle.0.id()) {
        *loaded = true;
        commands.entity(entity).remove::<Loading>();
        ew.send(RequestLoadModels);
    }
}

fn load_models(
    mut commands: Commands,
    mascot_preferences: Res<MascotPreferencesResource>,
    folders: Res<Assets<LoadedFolder>>,
    handle: Query<&ModelsFolderHandle>,
    asset_server: Res<AssetServer>,
    mascots: Query<&VrmPath>,
) {
    let Some(folder) = folders.get(handle.single().0.id()) else {
        return;
    };
    let exists_mascots = mascots
        .iter()
        .map(|p| p.0.as_path())
        .collect::<Vec<_>>();
    for asset_path in folder
        .handles
        .iter()
        .flat_map(|handle| handle.path())
        .filter(|path| !exists_mascots.contains(&path.path()))
    {
        commands.spawn((
            mascot_preferences.transform(asset_path.path()),
            VrmPath(asset_path.path().to_path_buf()),
            VrmHandle(asset_server.load(asset_path)),
        ));
    }
}

fn start_watching(
    mut commands: Commands,
) {
    let (sender, receiver) = crossbeam::channel::unbounded();
    let models_folder = models_dir();
    create_dir_all_if_need(&models_folder);

    match FileWatcher::new(
        models_folder,
        sender,
        Duration::from_secs(1),
    ) {
        Ok(watcher) => {
            commands.spawn(ModelFilesWatcher {
                _watcher: watcher,
                receiver,
            });
        }
        Err(e) => {
            error!("[FileWatcher] Failed to start watching files:\n{e}");
        }
    }
}

fn receive_events(
    mut commands: Commands,
    mascot_preferences: Res<MascotPreferencesResource>,
    asset_server: Res<AssetServer>,
    watchers: Query<&ModelFilesWatcher>,
) {
    while let Ok(event) = watchers.single().receiver.try_recv() {
        if let AssetSourceEvent::AddedAsset(path) = event {
            let relative_path = PathBuf::from("models").join(&path);
            commands.spawn((
                mascot_preferences.transform(&relative_path),
                VrmHandle(asset_server.load(models_dir().join(path))),
                VrmPath(relative_path),
            ));
        }
    }
}

