use crate::mascot::Mascot;
use crate::system_param::child_searcher::ChildSearcher;
use crate::vrm::extensions::{MorphTargetBind, VrmExtensions, VrmNode};
use crate::vrm::loader::{Vrm, VrmHandle};
use crate::vrm::{BonePgRestQuaternion, BoneRestTransform, VrmBone, VrmExpression, VrmHipsBoneTo, VrmPath};
use crate::vrma::load::RequestLoadVrma;
use bevy::app::{App, Update};
use bevy::asset::{Assets, Handle};
use bevy::core::Name;
use bevy::gltf::GltfNode;
use bevy::log::{error, info};
use bevy::math::Quat;
use bevy::prelude::{Added, Children, Commands, Component, Deref, DespawnRecursiveExt, Entity, EventWriter, Plugin, Query, Reflect, Res, Transform};
use bevy::render::view::RenderLayers;
use bevy::scene::SceneRoot;
use bevy::utils::HashMap;

pub struct VrmSpawnPlugin;

impl Plugin for VrmSpawnPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<VrmExpressions>()
            .register_type::<HumanoidBoneNodes>()
            .add_systems(Update, (
                spawn_vrm,
                attach_bones,
            ));
    }
}

#[derive(Reflect, Debug, Clone)]
pub struct ExpressionNode {
    pub name: Name,
    pub morph_target_index: usize,
}

#[derive(Component, Deref, Reflect)]
pub struct VrmExpressions(HashMap<VrmExpression, Vec<ExpressionNode>>);

impl VrmExpressions {
    pub fn new(
        extensions: &VrmExtensions,
        node_assets: &Assets<GltfNode>,
        nodes: &[Handle<GltfNode>],
    ) -> Self {
        let Some(expressions) = extensions
            .vrmc_vrm
            .expressions
            .as_ref()
        else {
            return Self(HashMap::default());
        };
        Self(expressions
            .preset
            .iter()
            .filter_map(|(preset_name, preset)| {
                let binds = preset.morph_target_binds.as_ref()?;
                let node = binds.iter()
                    .filter_map(|bind| convert_to_node(bind, node_assets, nodes))
                    .collect::<Vec<_>>();
                Some((VrmExpression(preset_name.clone()), node))
            })
            .collect()
        )
    }
}

fn convert_to_node(
    bind: &MorphTargetBind,
    node_assets: &Assets<GltfNode>,
    nodes: &[Handle<GltfNode>],
) -> Option<ExpressionNode> {
    let node_handle = nodes.get(bind.node)?;
    let node = node_assets.get(node_handle)?;
    Some(ExpressionNode {
        name: Name::new(node.name.clone()),
        morph_target_index: bind.index,
    })
}

#[derive(Component, Deref, Reflect)]
pub struct HumanoidBoneNodes(HashMap<VrmBone, Name>);

impl HumanoidBoneNodes {
    pub fn new(
        bones: &HashMap<String, VrmNode>,
        node_assets: &Assets<GltfNode>,
        nodes: &[Handle<GltfNode>],
    ) -> Self {
        Self(bones
            .iter()
            .filter_map(|(name, target_node)| {
                let node_handle = nodes.get(target_node.node as usize)?;
                let node = node_assets.get(node_handle)?;
                Some((VrmBone(name.clone()), Name::new(node.name.clone())))
            })
            .collect()
        )
    }
}

fn spawn_vrm(
    mut commands: Commands,
    mut ew: EventWriter<RequestLoadVrma>,
    node_assets: Res<Assets<GltfNode>>,
    vrm_assets: Res<Assets<Vrm>>,
    handles: Query<(Entity, &VrmHandle, &Transform, &VrmPath)>,
) {
    for (vrm_handle_entity, handle, tf, vrm_path) in handles.iter() {
        let Some(vrm) = vrm_assets.get(handle.0.id()) else {
            continue;
        };
        commands.entity(vrm_handle_entity).despawn_recursive();

        let Some(scene) = vrm.gltf.scenes.first() else {
            continue;
        };
        let extensions = match VrmExtensions::from_gltf(&vrm.gltf) {
            Ok(extensions) => extensions,
            Err(e) => {
                error!("[VRM] {e}");
                continue;
            }
        };

        // #[cfg(feature = "develop")]
        // create_vrm_json_for_debug(&vrm.gltf, &extensions);

        info!("Spawned mascot({:?}): {:?}", extensions.name(), vrm_path.0);
        commands.spawn((
            Mascot,
            RenderLayers::default(),
            SceneRoot(scene.clone()),
            vrm_path.clone(),
            *tf,
            VrmExpressions::new(&extensions, &node_assets, &vrm.gltf.nodes),
            HumanoidBoneNodes::new(
                &extensions.vrmc_vrm.humanoid.human_bones,
                &node_assets,
                &vrm.gltf.nodes,
            ),
            Name::new(extensions.name().unwrap_or_else(|| "VRM".to_string())),
        ));
        ew.send(RequestLoadVrma);
    }
}

fn attach_bones(
    mut commands: Commands,
    searcher: ChildSearcher,
    mascots: Query<(Entity, &HumanoidBoneNodes), Added<Children>>,
    transforms: Query<(&Transform, Option<&Children>)>,
) {
    for (mascot_entity, humanoid_bones) in mascots.iter() {
        for (bone, name) in humanoid_bones.0.iter() {
            let Some(bone_entity) = searcher.find_from_name(mascot_entity, name.as_str()) else {
                continue;
            };
            commands.entity(bone_entity).insert(bone.clone());
            // Use hips when sitting on window.
            if bone.0 == "hips" {
                commands.entity(mascot_entity).insert(VrmHipsBoneTo(bone_entity));
                recursive_attach_pg_resets(
                    &mut commands,
                    bone_entity,
                    Quat::IDENTITY,
                    &transforms,
                );
            }
        }
    }
}

fn recursive_attach_pg_resets(
    commands: &mut Commands,
    bone_entity: Entity,
    rest: Quat,
    transforms: &Query<(&Transform, Option<&Children>)>,
) {
    let Ok((tf, children)) = transforms.get(bone_entity) else {
        return;
    };
    let pg_rest = BonePgRestQuaternion(tf.rotation * rest);
    commands.entity(bone_entity).insert((
        BoneRestTransform(*tf),
        pg_rest,
    ));
    if let Some(children) = children {
        for child in children.iter() {
            recursive_attach_pg_resets(commands, *child, pg_rest.0, transforms);
        }
    }
}

#[cfg(feature = "develop")]
fn create_vrm_json_for_debug(
    gltf: &bevy::prelude::Gltf,
    extensions: &VrmExtensions,
) {
    let dir = std::path::PathBuf::from("test").join("vrm");
    if !dir.exists() {
        std::fs::create_dir_all(&dir).unwrap();
    }
    let path = dir.join(format!("{}.json", extensions.name().unwrap_or_default()));
    std::fs::write(path, serde_json::to_string_pretty(&crate::vrm::extensions::obtain_extensions(gltf).unwrap()).unwrap()).unwrap();
}
