#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup() {
        unsafe {
            // ngSpice_Init(printfcn, statfcn, ngexit, sdata, sinitdata, bgtrun, userData)
        }
    }
}