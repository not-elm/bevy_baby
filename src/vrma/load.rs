use crate::mascot::Mascot;
use crate::power_state::Loading;
use crate::settings::preferences::action::ActionPreferences;
use crate::settings::state::MascotAction;
use crate::vrm::spawn::HumanoidBoneNodes;
use crate::vrm::VrmExpression;
use crate::vrma::animation::{AnimationPlayerEntities, VrmAnimationGraph};
use crate::vrma::extensions::VrmaExtensions;
use crate::vrma::loader::VrmaAsset;
use crate::vrma::{RetargetTo, Vrma, VrmaDuration, VrmaHandle};
use bevy::animation::AnimationClip;
use bevy::app::{App, Plugin, PreStartup, Update};
use bevy::asset::{Assets, LoadedFolder};
use bevy::core::Name;
use bevy::gltf::GltfNode;
use bevy::log::{debug, error};
use bevy::prelude::{on_event, AnimationGraph, AssetServer, BuildChildren, Children, Commands, Component, Deref, DespawnRecursiveExt, Entity, Event, EventWriter, Handle, IntoSystemConfigs, Local, Or, ParallelCommands, Parent, Query, Reflect, Res, ResMut, With};
use bevy::scene::SceneRoot;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Event, Debug)]
pub struct RequestLoadVrma;

pub struct VrmaLoadPlugin;

impl Plugin for VrmaLoadPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<AnimationFolderHandle>()
            .add_event::<RequestLoadVrma>()
            .add_systems(PreStartup, start_load_folder)
            .add_systems(Update, (
                wait_load,
                spawn_vrma,
                (
                    remove_all_removed_vrma,
                    request_load_all_added_vrma,
                )
                    .chain()
                    .run_if(on_event::<RequestLoadVrma>),
            ));
    }
}

#[derive(Component, Deref, Reflect)]
pub struct VrmaExpressionNames(Vec<VrmExpression>);

impl VrmaExpressionNames {
    pub fn new(
        extensions: &VrmaExtensions,
    ) -> Self {
        let Some(expressions) = extensions.vrmc_vrm_animation.expressions.as_ref() else {
            return Self(Vec::default());
        };
        Self(expressions
            .preset
            .keys()
            .map(|expression| VrmExpression(expression.clone()))
            .collect())
    }
}

#[derive(Component, Reflect)]
struct AnimationFolderHandle(Handle<LoadedFolder>);

fn start_load_folder(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Loading,
        AnimationFolderHandle(asset_server.load_folder("animations")),
    ));
}

fn wait_load(
    mut commands: Commands,
    mut ew: EventWriter<RequestLoadVrma>,
    mut loaded: Local<bool>,
    folder_assets: Res<Assets<LoadedFolder>>,
    folder_handle: Query<(Entity, &AnimationFolderHandle)>,
) {
    if *loaded {
        return;
    }
    let (entity, handle) = folder_handle.single();
    if folder_assets.get(handle.0.id()).is_some() {
        *loaded = true;
        commands.entity(entity).remove::<Loading>();
        ew.send(RequestLoadVrma);
    }
}

fn request_load_all_added_vrma(
    commands: ParallelCommands,
    asset_server: Res<AssetServer>,
    folder_assets: Res<Assets<LoadedFolder>>,
    folder_handle: Query<&AnimationFolderHandle>,
    mascots: Query<(Entity, Option<&Children>), With<Mascot>>,
    vrma: Query<&MascotAction, Or<(With<Vrma>, With<VrmaHandle>)>>,
) {
    for (state, asset_path) in all_loaded_vrma_path(&folder_assets, &folder_handle) {
        mascots.par_iter().for_each(|(mascot_entity, children)| {
            if already_attached(children, &vrma, &state) {
                return;
            }
            debug!("Added {state} to {mascot_entity}");
            commands.command_scope(|mut commands| {
                commands.entity(mascot_entity).with_child((
                    state.clone(),
                    VrmaHandle(asset_server.load(asset_path.clone())),
                ));
            });
        });
    }
}

fn remove_all_removed_vrma(
    mut commands: Commands,
    folder_assets: Res<Assets<LoadedFolder>>,
    folder_handle: Query<&AnimationFolderHandle>,
    vrma: Query<(Entity, &MascotAction), Or<(With<Vrma>, With<VrmaHandle>)>>,
) {
    let current_exists = all_loaded_vrma_path(&folder_assets, &folder_handle)
        .into_iter()
        .map(|(state, _)| state)
        .collect::<Vec<_>>();
    for (remove_vrma_entity, state) in search_all_removed_vrma_path(&current_exists, vrma.iter()) {
        debug!("Remove {state} from {remove_vrma_entity}");
        commands.entity(remove_vrma_entity).despawn_recursive();
    }
}

fn all_loaded_vrma_path(
    folder_assets: &Res<Assets<LoadedFolder>>,
    folder_handle: &Query<&AnimationFolderHandle>,
) -> Vec<(MascotAction, PathBuf)> {
    let Some(folder) = folder_handle
        .get_single()
        .ok()
        .and_then(|handle| folder_assets.get(handle.0.id()))
    else {
        return Vec::with_capacity(0);
    };
    folder
        .handles
        .iter()
        .filter_map(|handle| {
            let path = handle.path()?.path();
            let state = MascotAction::new(path)?;
            Some((state, path.to_path_buf()))
        })
        .collect()
}

fn already_attached(
    children: Option<&Children>,
    vrma: &Query<&MascotAction, Or<(With<Vrma>, With<VrmaHandle>)>>,
    state: &MascotAction,
) -> bool {
    let Some(children) = children else {
        return false;
    };
    children
        .iter()
        .any(|entity| {
            vrma.get(*entity).is_ok_and(|vrma_state| vrma_state == state)
        })
}

fn spawn_vrma(
    mut commands: Commands,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
    actions: Res<ActionPreferences>,
    vrma_assets: Res<Assets<VrmaAsset>>,
    node_assets: Res<Assets<GltfNode>>,
    clip_assets: Res<Assets<AnimationClip>>,
    vrma_handles: Query<(Entity, &VrmaHandle, &MascotAction, &Parent)>,
) {
    for (handle_entity, handle, action, parent) in vrma_handles.iter() {
        let mascot_entity = parent.get();
        let Some(vrma) = vrma_assets.get(handle.0.id()) else {
            continue;
        };
        commands.entity(handle_entity).despawn_recursive();

        let Some(scene_root) = vrma.gltf.scenes.first().cloned() else {
            error!("[VRMA] Not found vrma scene in {action}");
            continue;
        };
        let extensions = match VrmaExtensions::from_gltf(&vrma.gltf) {
            Ok(extensions) => extensions,
            Err(e) => {
                error!("[VRMA] Not found vrma extensions in {action}:\n{e}");
                continue;
            }
        };

        // #[cfg(feature = "develop")]
        // create_vrma_json_for_debug(&extensions, action);

        commands.entity(mascot_entity).with_child((
            Vrma,
            Name::from(action.to_string()),
            RetargetTo(mascot_entity),
            SceneRoot(scene_root),
            actions.properties(action),
            action.clone(),
            AnimationPlayerEntities::default(),
            obtain_vrma_duration(&clip_assets, &vrma.gltf.animations),
            VrmAnimationGraph::new(vrma.gltf.animations.to_vec(), &mut animation_graphs),
            VrmaExpressionNames::new(&extensions),
            HumanoidBoneNodes::new(
                &extensions.vrmc_vrm_animation.humanoid.human_bones,
                &node_assets,
                &vrma.gltf.nodes,
            ),
        ));
    }
}

#[cfg(feature = "develop")]
fn create_vrma_json_for_debug(
    extensions: &VrmaExtensions,
    state: &MascotAction,
) {
    let dir = std::path::PathBuf::from("test").join("vrma");
    if !dir.exists() {
        std::fs::create_dir_all(&dir).unwrap();
    }
    let path = dir.join(format!("{state}.json"));
    std::fs::write(path, serde_json::to_string_pretty(&extensions).unwrap()).unwrap();
}

fn obtain_vrma_duration(
    assets: &Assets<AnimationClip>,
    handles: &[Handle<AnimationClip>],
) -> VrmaDuration {
    let duration = handles
        .iter()
        .filter_map(|handle| assets.get(handle))
        .map(|clip| clip.duration() as f64)
        .fold(0., |v1, v2| v2.max(v1));
    VrmaDuration(Duration::from_secs_f64(duration))
}

fn search_all_removed_vrma_path<'w>(
    current_exists: &[MascotAction],
    spawned_vrma: impl Iterator<Item=(Entity, &'w MascotAction)>,
) -> Vec<(Entity, &'w MascotAction)> {
    spawned_vrma
        .filter(|(_, state)| { !current_exists.contains(state) })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::settings::state::{ActionGroup, MascotAction};
    use crate::vrma::load::search_all_removed_vrma_path;
    use bevy::prelude::{default, Entity};

    #[test]
    fn test_search_removed_vrma() {
        let s1 = MascotAction::default();
        let s2 = MascotAction {
            group: ActionGroup::sit_down(),
            ..default()
        };
        let s3 = MascotAction {
            group: ActionGroup::drag(),
            ..default()
        };
        let exists = [s1.clone()];
        let spawned = [
            (Entity::from_raw(0), &s1),
            (Entity::from_raw(1), &s2),
            (Entity::from_raw(2), &s3),
        ];
        let removed = search_all_removed_vrma_path(&exists, spawned.iter().cloned());
        assert_eq!(removed, vec![
            (Entity::from_raw(1), &s2),
            (Entity::from_raw(2), &s3),
        ]);
    }
}