
pub struct Wire;
use bevy::prelude::*;

impl Plugin for Wire {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, main);
    }
}

fn setup() {}

fn main() {}