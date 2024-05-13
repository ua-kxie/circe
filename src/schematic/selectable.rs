/// functionality to implement selectability for click drag and such
///
/// copy: adds copy of all selected entities, remove selected marker on old copies, 
/// add marker on new copies, preview and place with transform
/// 
/// move: preview and place with transform
/// 
/// 
use bevy::prelude::*;

use super::NewCurposI;

// componenent marks entity as a schematic element - can be selected, marked as tentative. 
// Entities with this compoenent should be checked for collision against cursor

#[derive(Component)]
struct SchematicElement;

#[derive(Component)]
struct Selected;

#[derive(Component)]
struct Tentative;

fn main(
    es: Query<Entity, With<SchematicElement>>,
    mut e_new_curpos: EventReader<NewCurposI>,
) {
    if let Some(curpos) = e_new_curpos.read().last() {
        // update with tentative marker
    }

}