//! structs for storing ngspice model definitions such as for nmos, pmos, or diode models.
//! not in use atm
struct NgModels {
    models: Vec<NgModel>,
}

impl NgModels {
    fn model_definitions(&self) -> String {
        let mut ret = String::new();
        for m in &self.models {
            ret.push_str(&m.model_line())
        }
        ret
    }
}

struct NgModel {
    name: String,
    definition: String,
}

impl NgModel {
    fn model_line(&self) -> String {
        format!(".model {} {}\n", self.name, self.definition)
    }
}