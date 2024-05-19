use bevy::prelude::*;

/// wire: 
/// point,
/// point,
/// seg,

/// point entity, 2 per line segment
/// optionally render solder dot
/// always render mesh at (0, 0), use spatial transform for location
#[derive(Component)]
struct WireSegEnd;

/// line seg entity, 1 per line segment
/// have two WireSegEnd as children in ecs
/// identity transform, create mesh at WireSegEnd locations
#[derive(Component)]
struct WireSeg;

/// system to update wireseg mesh according to children's position
fn wire_system() {}