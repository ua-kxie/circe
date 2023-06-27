use super::super::params;
use super::Graphics;
use iced::Element;
use lazy_static::lazy_static;

pub const ID_PREFIX: &str = "V";

lazy_static! {
    static ref DEFAULT_GRAPHICS: Graphics =
        serde_json::from_slice(&std::fs::read("src/schematic/devices/devicetype/v.json").unwrap())
            .unwrap();
}

#[derive(Debug, Clone)]
pub enum ParamVMsg {
    RawPEMsg(params::RawPEMsg)
}

#[derive(Debug)]
pub enum ParamV {
    Raw(params::Raw),
}
impl Default for ParamV {
    fn default() -> Self {
        ParamV::Raw(params::Raw::new(String::from("3.3")))
    }
}
impl ParamV {
    pub fn summary(&self) -> String {
        match self {
            ParamV::Raw(s) => s.raw.clone(),
        }
    }
    pub fn view(&self) -> Element<ParamVMsg> {
        match self {
            ParamV::Raw(raw) => {
                raw.view().map(ParamVMsg::RawPEMsg)
            },
        }
    }
    pub fn update(&mut self, msg: ParamVMsg) {
        match msg {
            ParamVMsg::RawPEMsg(msg) => {
                let ParamV::Raw(raw) = self;
                raw.update(msg)
            },
        }
    }
}

#[derive(Debug)]
pub struct V {
    pub params: ParamV,
    pub graphics: &'static Graphics,
}
impl V {
    pub fn new() -> V {
        V {
            params: ParamV::default(),
            graphics: &DEFAULT_GRAPHICS,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json;
    #[test]
    fn it_works() {
        let out = serde_json::json!(*super::DEFAULT_GRAPHICS);
        std::fs::write(
            "src/schematic/devices/devicetype/v.json",
            serde_json::to_string_pretty(&out).unwrap().as_bytes(),
        )
        .expect("Unable to write file");
    }

    fn parse() {
        let a = std::fs::read("src/schematic/devices/devicetype/v.json").unwrap();
        let b: super::Graphics = serde_json::from_slice(&a).unwrap();
    }
}
