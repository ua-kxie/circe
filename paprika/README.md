## Overview
NgSpice is an open-sourced electronics circuit simulation engine based on the venerable SPICE simulator developed and maintained at UC Berkeley during the 80s and 90s. It is the industry standard: to this day, SPICE simulation models are what vendors provide along their discreet device offerings. 

This is a work in progress. Use at your own risk. 

Known memory safety issues:

Type implementing PkSpiceManager moving in memory after calling init will crash once a callback is called.

Commanding `bg_run` creates a new thread


The following functions exposed by the API still need to be implemented


~~ngSpice_Init~~

ngSpice_Init_Sync

~~ngSpice_Command~~

~~ngGet_Vec_Info~~

ngCM_Input_Path

ngGet_Evt_NodeInfo

ngSpice_AllEvtNodes

ngSpice_Init_Evt

ngSpice_Circ

~~ngSpice_CurPlot~~

~~ngSpice_AllPlots~~

~~ngSpice_AllVecs~~

~~ngSpice_running~~

ngSpice_SetBkpt

## Installation
Obtain the appropriate `sharedspice` lib from [here](https://ngspice.sourceforge.io/shared.html). `Sharedspice.dll` for windows can be downloaded directly from the webpage. It is also available through [homebrew](https://formulae.brew.sh/formula/libngspice). Linux binding is not tested.

The example code in `main.rs` and `tests/lib.rs` show how to specify the `sharedspice` path. 

## Explanation
Compiling `main.rs` produces a simple command line program which passes messages between the user and NgSpice's `command` call. <span style="color:green">stdout</span>, <span style="color:red">stderr</span>, and <span style="color:blue">stats</span> are color coded. If you see <span style="color:magenta">~~something like this~~</span> please open an issue detailing how to reproduce it.

Both `main.rs` and `tests/lib.rs` contain simple examples of how a manager may be implemented. 

## Contribute
Any contribution is welcome. 

`sharedspice` source code specifies that its memory are not to be freed by the calling program. 

Callbacks may be called from a different thread and thus need to consider thread safety.

## Resources
[NgSpice Beginner Tutorial](https://ngspice.sourceforge.io/ngspice-tutorial.html)

[NgSpice Manual](https://ngspice.sourceforge.io/docs/ngspice-39-manual.pdf)

## Other Bindings
### Rust
[NyanCAD NgSpice bindings](https://github.com/NyanCAD/rust-ngspice)  (Incomplete, no documentations nor code comments. ngspice-sys on crates.io)

[elektron_ngspice](https://github.com/spielhuus/elektron_ngspice/blob/main/src/lib.rs) (Incomplete, no documentations nor code comments. elektron_ngspice on crates.io. Almost identical to NyanCAD above)

### Others
[KiCad NgSpice Binding (cpp)](https://gitlab.com/kicad/code/kicad/-/blob/master/eeschema/sim/ngspice.cpp)

[PySpice NgSpice Binding (Python)](https://github.com/PySpice-org/PySpice/blob/master/PySpice/Spice/NgSpice/Shared.py)

Some existing NgSpice based simulators can be found [here](https://ngspice.sourceforge.io/resources.html)
