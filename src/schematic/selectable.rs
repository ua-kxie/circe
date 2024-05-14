/// functionality to implement selectability for click drag and such
///
/// copy: adds copy of all selected entities, remove selected marker on old copies,
/// add marker on new copies, preview and place with transform
///
/// move: preview and place with transform
///
///
use bevy::prelude::*;
use bevy::math::bounding::{Aabb3d, AabbCast3d, IntersectsVolume, RayCast3d};
/// componenent marks entity as a schematic element - can be selected, marked as tentative.
/// Entities with this compoenent should be checked for collision against cursor

#[derive(Component)]
pub struct SchematicElement{
    pub aabb: Aabb3d,
}

// /// component is added automatically to SchematicElements without it. 
// /// Should be deleted on elements which changes position/shape.
// #[derive(Component)]
// pub struct Collider(Aabb3d);

#[derive(Component)]
pub struct Selected;

#[derive(Component)]
pub struct Tentative;

// events

/// event fires when elements marked tentatives should be come selected
#[derive(Event)]
struct TentativesToSelected;

/// event fires when elements marked selected should be cleared
#[derive(Event)]
struct ClearSelected;

/// event fires when tentative collider changes (mouse moves, or area selection is dropped)
#[derive(Event)]
pub enum NewTentativeCollider{
    Ray(RayCast3d),
    Volume(AabbCast3d),
}

/// this function marks schematic elements with the [`Tentative`] marker if they intersect the selection collider
fn mark_tentatives(
    schematic_elements: Query<(Entity, &SchematicElement)>, 
    e_tentatives: Query<Entity, With<Tentative>>, 
    mut enew_tentative_collider: EventReader<NewTentativeCollider>,
    mut commands: Commands,
) {
    // mark as tentative if tentative collider changes
    if let Some(collider) = enew_tentative_collider.read().last() {
        // clear tentative markers
        for e in &e_tentatives {
            commands.entity(e).remove::<Tentative>();
        }
        dbg!("A");
        // add tentative markers based on new information
        for e in schematic_elements.iter() {
            if match collider {
                NewTentativeCollider::Ray(cast) => {
                    cast.intersects(&e.1.aabb)
                },
                NewTentativeCollider::Volume(cast) => {
                    cast.intersects(&e.1.aabb)
                },
            } {
                dbg!("any");
                commands.entity(e.0).insert(Tentative);
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

// fn update_element_collider(
//     schematic_elements: Query<Entity, With<SchematicElement>, Without<Collider>>, 
// ) {
    
// }

pub struct SelectablePlugin;

impl Plugin for SelectablePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                select,
                clr_selected,
                mark_tentatives,
            ),
        );
        app.add_event::<TentativesToSelected>();
        app.add_event::<ClearSelected>();
        app.add_event::<NewTentativeCollider>();
    }
}