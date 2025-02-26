mod setup;
mod hit_test;
#[cfg(target_os = "windows")]
mod windows;

use crate::application_windows::hit_test::ApplicationWindowsHitTestPlugin;
use crate::application_windows::setup::ApplicationWindowsSetupPlugin;
use bevy::app::App;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::render::view::{NoFrustumCulling, RenderLayers};

#[derive(Debug, Component, Eq, PartialEq, Copy, Clone, Reflect)]
#[reflect(Component)]
pub struct TargetMonitor(pub Entity);


pub struct ApplicationWindowsPlugin;

impl Plugin for ApplicationWindowsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                ApplicationWindowsSetupPlugin,
                ApplicationWindowsHitTestPlugin,
                #[cfg(target_os = "windows")]
                windows::ApplicationWindowOnWindowsPlugin,
            ));

        app
            .world_mut()
            .register_component_hooks::<Mesh3d>()
            .on_add(|mut world: DeferredWorld, entity: Entity, _| {
                world.commands().entity(entity).insert((
                    //FIXME: When mascot moves to the top of the screen while sitting, the face is not drawn, 
                    // so it is inserted as a temporary measure
                    NoFrustumCulling,
                    RenderLayers::default(),
                ));
            });
    }
}
