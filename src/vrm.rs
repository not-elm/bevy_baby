pub mod loader;
mod spawn;
pub mod extensions;
mod load;
mod spring_bone;
pub mod humanoid_bone;
pub mod expressions;

use crate::new_type;
use crate::vrm::expressions::VrmExpressionPlugin;
use crate::vrm::humanoid_bone::VrmHumanoidBonePlugin;
use crate::vrm::load::VrmLoadPlugin;
use crate::vrm::loader::{Vrm, VrmLoaderPlugin};
use crate::vrm::spawn::VrmSpawnPlugin;
use crate::vrm::spring_bone::VrmSpringBonePlugin;
use bevy::app::{App, Plugin};
use bevy::asset::AssetApp;
use bevy::prelude::{Component, Entity, GlobalTransform, Reflect, Transform};
use std::path::PathBuf;

new_type!(VrmBone, String);
new_type!(VrmExpression, String);

#[derive(Debug, Reflect, Clone, Component)]
pub struct VrmPath(pub PathBuf);

#[derive(Debug, Reflect, Copy, Clone, Component)]
pub struct BoneRestTransform(pub Transform);

#[derive(Debug, Reflect, Copy, Clone, Component)]
pub struct BoneRestGlobalTransform(pub GlobalTransform);

#[derive(Debug, Reflect, Copy, Clone, Component)]
pub struct VrmHipsBoneTo(pub Entity);

pub struct VrmPlugin;

impl Plugin for VrmPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_asset::<Vrm>()
            .register_type::<VrmPath>()
            .register_type::<BoneRestTransform>()
            .register_type::<BoneRestGlobalTransform>()
            .register_type::<VrmHipsBoneTo>()
            .register_type::<VrmBone>()
            .add_plugins((
                VrmLoadPlugin,
                VrmLoaderPlugin,
                VrmSpawnPlugin,
                VrmSpringBonePlugin,
                VrmHumanoidBonePlugin,
                VrmExpressionPlugin,
            ));
    }
}


