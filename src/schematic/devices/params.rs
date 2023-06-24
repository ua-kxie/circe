//! device parameter types
//! multiple devices can use the same parameter specifier. e.g. all devices can use the `Raw` paramter specifier, R L C can use `SingleValue`, etc.
//! a device should be able to choose between all compatible parameter specifier

/// this struct to edit device parameters by specifying the spice netlist line (after port connects) directly
#[derive(Debug)]
pub struct Raw {
    pub raw: String,
}
impl Raw {
    pub fn new(raw: String) -> Self {
        Raw { raw }
    }
    pub fn set(&mut self, new: String) {
        self.raw = new;
    }
}

/// this struct to edit device paramters by specying a single characteristic value (resistance, capacitance, inductance)
#[derive(Debug)]
pub struct SingleValue {
    pub value: f32,
}
impl SingleValue {
    fn new(value: f32) -> Self {
        SingleValue { value }
    }
}
