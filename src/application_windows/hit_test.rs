use crate::system_param::cameras::Cameras;
use crate::system_param::mouse_position::MousePosition;
use bevy::app::{App, Plugin};
use bevy::prelude::{Entity, MeshRayCast, Query, RayCastSettings};
use bevy::render::view::RenderLayers;
use bevy::window::Window;

pub struct ApplicationWindowsHitTestPlugin;

impl Plugin for ApplicationWindowsHitTestPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(PreUpdate, update_hit_test.run_if(resource_exists_and_changed::<GlobalMouseCursor>));
    }
}

fn update_hit_test(
    mut mesh_ray_cast: MeshRayCast,
    mut windows: Query<(Entity, &mut Window, &RenderLayers)>,
    mouse_position: MousePosition,
    cameras: Cameras,
) {
    for (window_entity, mut window, layers) in windows.iter_mut() {
        let Some((camera, tf, _)) = cameras.find_camera(window_entity) else {
            window.cursor_options.hit_test = false;
            continue;
        };
        let Some(local_cursor_pos) = mouse_position.local(layers) else {
            window.cursor_options.hit_test = false;
            continue;
        };
        let Ok(ray) = camera.viewport_to_world(tf, local_cursor_pos) else {
            window.cursor_options.hit_test = false;
            continue;
        };
        let hitting_anyone = !mesh_ray_cast
            .cast_ray(ray, &RayCastSettings::default())
            .is_empty();
        match (hitting_anyone, window.cursor_options.hit_test) {
            (true, false) => window.cursor_options.hit_test = true,
            #[cfg(not(feature = "develop"))]
            (false, true) => window.cursor_options.hit_test = false,
            _ => (),
        }
    }
}


