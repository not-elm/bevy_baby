use crate::application_windows::TargetMonitor;
use crate::global_mouse::cursor::GlobalMouseCursor;
use crate::system_param::monitors::monitor_rect;
use bevy::app::{App, Startup};
use bevy::core::Name;
use bevy::log::debug;
use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::prelude::{Commands, Entity, Plugin, Query};
use bevy::render::camera::{RenderTarget, ScalingMode};
use bevy::render::view::RenderLayers;
use bevy::window::{CursorOptions, Monitor, PrimaryMonitor, PrimaryWindow, WindowLevel, WindowMode, WindowRef, WindowResolution};

pub struct ApplicationWindowsSetupPlugin;

impl Plugin for ApplicationWindowsSetupPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<TargetMonitor>()
            .add_systems(Startup, (
                // remove_primary,
                setup_windows,
            ).chain());
    }
}

#[derive(Component)]
pub struct A;

fn remove_primary(
    mut commands: Commands,
    window: Query<Entity, With<PrimaryWindow>>,
) {
    commands.entity(window.single()).despawn();
}

fn setup_windows(
    mut commands: Commands,
    global_cursor: Res<GlobalMouseCursor>,
    monitors: Query<(Entity, &Monitor, Option<&PrimaryMonitor>)>,
) {
    let cursor_pos = global_cursor.global_cursor_pos();
    let Some(scale_factor) = monitors
        .iter()
        .find_map(|(_, monitor, _)| {
            monitor_rect(monitor).contains(cursor_pos).then_some(monitor.scale_factor as f32)
        })
    else {
        return;
    };

    for (layer, (monitor_entity, monitor, primary)) in monitors.iter().enumerate() {
        let mut window = create_window(monitor.physical_size().as_vec2());

        debug!("Monitor({:?}) {:?}", monitor.physical_position, monitor.physical_size());
        window.position.set((monitor.physical_position.as_vec2() * scale_factor).as_ivec2());
        let size = monitor.physical_size();
        window.resolution.set_scale_factor(monitor.scale_factor as f32);

        let window_entity = commands.spawn((
            Name::new(format!("Window({:?})", monitor.physical_position)),
            TargetMonitor(monitor_entity),
            RenderLayers::layer(layer),
           // window,
        )).id();
        commands.entity(monitor_entity).insert(RenderLayers::layer(layer));
        spawn_camera(&mut commands, window_entity, layer);

        // if primary.is_some() {
        //     commands.entity(window_entity).insert(PrimaryWindow);
        // }
    }
}

fn spawn_camera(
    commands: &mut Commands,
    window_entity: Entity,
    camera_layer: usize,
) {
    commands.spawn((
        Name::new(format!("Camera({camera_layer})")),
        RenderLayers::layer(camera_layer),
        Camera3d::default(),
        Camera {
           // target: RenderTarget::Window(WindowRef::Entity(window_entity)),
            hdr: true,
            ..default()
        },
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 3.,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(0., 0.0, 4.5),
    ));
}

fn create_window(size: Vec2) -> Window {
    Window {
        transparent: true,
        has_shadow: false,
        #[cfg(target_os = "macos")]
        composite_alpha_mode: bevy::window::CompositeAlphaMode::PostMultiplied,
        resizable: true,
        decorations: false,
        window_level: WindowLevel::Normal,
        resolution: WindowResolution::new(800., 800.),
        titlebar_shown: true,
        mode: WindowMode::Windowed,
        cursor_options: CursorOptions {
            hit_test: false,
            ..default()
        },
        ..default()
    }
}