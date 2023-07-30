//! net name
//! 
//! 
//! 

// every strongly separated net graph should be assigned one NetLabel on checking ... ?
// or: every net seg holds a Rc<NetLabel>, the same underlying NetLabel if connected

/// net label, which can be user set
pub struct NetLabel {
    /// autogenerated unique net label, always available
    autogen: String,
    /// optional overriding net label set by user
    custom: Option<String>,
}

impl NetLabel {
    /// return the user defined net name if it is set, otherwise return the autogenerated net label
    fn read(&self) -> &str {
        if let Some(ret) = &self.custom {
            return &ret;
        } else {
            return &self.autogen;
        }
    }

    /// set the user defiend net name
    fn set(&mut self, label: String) {
        self.custom = Some(label);
    }
}