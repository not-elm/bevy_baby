#![allow(
    clippy::type_complexity,
    clippy::too_many_arguments,
)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod application_windows;
mod mascot;
mod global_window;
mod system_param;
mod power_state;
mod global_mouse;
mod menu;
mod settings;
mod error;
mod macros;
mod vrm;
mod vrma;
mod util;

use crate::application_windows::ApplicationWindowsPlugin;
use crate::global_mouse::GlobalMousePlugin;
use crate::mascot::DesktopMascotPlugin;
use crate::menu::MenuPlugin;
use crate::power_state::PowerStatePlugin;
use crate::settings::AppSettingsPlugin;
use crate::vrm::VrmPlugin;
use crate::vrma::VrmaPlugin;
use bevy::app::{App, PluginGroup};
use bevy::color::Color;
use bevy::log::LogPlugin;
use bevy::prelude::{default, AmbientLight, ClearColor, MeshPickingPlugin};
use bevy::render::settings::{RenderCreation, WgpuSettings};
use bevy::render::RenderPlugin;
use bevy::window::WindowPlugin;
use bevy::DefaultPlugins;
use bevy_webview_wry::api::{AllLogPlugins, AppExitApiPlugin};
use bevy_webview_wry::prelude::AllDialogPlugins;
use bevy_webview_wry::WebviewWryPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(LogPlugin {
                    #[cfg(debug_assertions)]
                    level: bevy::log::Level::DEBUG,
                    #[cfg(target_os = "windows")]
                    filter: "wgpu_hal=off".to_string(),
                    ..default()
                })
                .set(WindowPlugin {
                    // Windows won't start without PrimaryWindow for some reason.
                    #[cfg(not(target_os = "windows"))]
                    primary_window: None,
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        // On Windows, neither on `VULKAN` and `DX12` don't work transparency.
                        // Ensure not certainty that this will work correctly on all devices.
                        #[cfg(target_os = "windows")]
                        backends: Some(bevy::render::settings::Backends::GL),
                        ..default()
                    }),
                    ..default()
                }),
            WebviewWryPlugin {
                local_root: std::path::PathBuf::from("ui")
            },
            MeshPickingPlugin,
            #[cfg(feature = "develop")]
            bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
        ))
        .add_plugins((
            AllDialogPlugins,
            AllLogPlugins,
            AppExitApiPlugin,
        ))
        .add_plugins((
            MenuPlugin,
            DesktopMascotPlugin,
            PowerStatePlugin,
            VrmPlugin,
            VrmaPlugin,
            ApplicationWindowsPlugin,
            GlobalMousePlugin,
            AppSettingsPlugin,
        ))
        .insert_resource(AmbientLight {
            brightness: 3000.0,
            ..default()
        })
        .insert_resource(ClearColor(Color::NONE))
        .run();
}

#[cfg(test)]
pub(crate) mod tests {
    use bevy::MinimalPlugins;

    pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

    #[macro_export]
    macro_rules! success {
        () => {
            std::result::Result::Ok(())
        };
    }

    pub fn test_app() -> bevy::app::App {
        let mut app = bevy::app::App::new();
        app.add_plugins(MinimalPlugins);
        app
    }
}