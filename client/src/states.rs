use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum AppState {
    #[default]
    Loading,
    Setup,
    Playing,
}
