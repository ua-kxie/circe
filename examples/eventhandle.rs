fn main() {
    let (event: i16, vstate: i16, sstate: i16) = (0, 0, 0);  // event, viewport state, schematic state
    match vstate {
        panning => {
            match event {}
        },
        newview => {
            match event {}
        }
        none => {
            match sstate {
                wiring => {},
                device_placing => {},
            }
        }
    }
}
// how to enter a vstate?