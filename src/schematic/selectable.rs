/// functionality to implement selectability for click drag and such
///
/// copy: adds copy of all selected entities, remove selected marker on old copies,
/// add marker on new copies, preview and place with transform
///
/// move: preview and place with transform
///
///
use bevy::prelude::*;
use euclid::default;

use super::NewCurposI;

/// componenent marks entity as a schematic element - can be selected, marked as tentative.
/// Entities with this compoenent should be checked for collision against cursor

#[derive(Component)]
struct SchematicElement;

#[derive(Component)]
struct Selected;

#[derive(Component)]
struct Tentative;

// events

/// event fires when elements marked tentatives should be come selected
#[derive(Event)]
struct TentativesToSelected;

/// event fires when elements marked selected should be cleared
#[derive(Event)]
struct ClearSelected;

/// event fires when tentative collider changes (mouse moves, or area selection is dropped)
#[derive(Event)]
struct NewTentativeCollider;

/// state
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum SelectSt {
    #[default]
    Point, // select elements from a point / by intersection with ray
    Aabb, // select elements from an aabb area / by intersection with frustrum (cuboid in orthographic projection)
}


/// this function marks schematic elements with the [`Tentative`] marker if they intersect the selection collider
fn mark_tentatives(
    schematic_elements: Query<Entity, With<SchematicElement>>, 
    e_tentatives: Query<Entity, With<Tentative>>, 
    mut enew_tentative_collider: EventReader<NewTentativeCollider>,
    mut commands: Commands,
    selst: Res<State<SelectSt>>,
) {
    if let Some(curpos) = enew_tentative_collider.read().last() {
        // clear tentative markers
        for e in &e_tentatives {
            commands.entity(e).remove::<Tentative>();
        }
        // add tentative markers based on new information
        let st = selst.get();
        for e in schematic_elements.iter() {
            if false {  // todo: if entity collides with point/ray or area/cuboid based on selection state
                commands.entity(e).insert(Tentative);
            }
        }
    }
}


/// this system simply adds the [`Selected`] marker to all entities already marked with [`Tentative`], 
/// conditioned on the event [`TentativesToSelected`]
fn select (
    mut e: EventReader<TentativesToSelected>,
    es: Query<Entity, With<Tentative>>, 
    mut commands: Commands,
) {
    if e.read().last().is_some() {
        for element in es.iter() {
            commands.entity(element).insert(Selected);
        }
    }
}

/// this system simply removes the [`Selected`] marker to all entities already marked, 
/// conditioned on the event [`ClearSelected`]
fn clr_selected (
    mut e: EventReader<ClearSelected>,
    es: Query<Entity, With<Selected>>, 
    mut commands: Commands,
) {
    if e.read().last().is_some() {
        for element in es.iter() {
            commands.entity(element).remove::<Selected>();
        }
    }
}