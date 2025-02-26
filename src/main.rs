#![allow(
    clippy::type_complexity,
    clippy::too_many_arguments,
)]

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
                    level: bevy::log::Level::INFO,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: None,
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