use crate::mascot::Mascot;
use bevy::app::{App, Plugin};
use bevy::prelude::{AppExtStates, OnEnter, Query, States, Visibility, With};

#[derive(Debug, Default, States, Eq, PartialEq, Hash, Copy, Clone)]
pub enum MascotVisibilityState {
    #[default]
    Visible,
    Invisible,
}

pub struct MascotVisibilityPlugin;

impl Plugin for MascotVisibilityPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<MascotVisibilityState>();
        // .add_systems(OnEnter(MascotVisibilityState::Visible), set_visible::<true>)
        // .add_systems(OnEnter(MascotVisibilityState::Invisible), set_visible::<false>);
    }
}

fn set_visible<const VISIBLE: bool>(
    mut mascots: Query<&mut Visibility, With<Mascot>>,
) {
    for mut mascot in mascots.iter_mut() {
        *mascot = if VISIBLE { Visibility::Visible } else { Visibility::Hidden };
    }
}