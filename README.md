Circe: Prototyping for 2D CAD drawn with Iced canvas

### Preview:
Simple op-amp with generic devices
![Screenshot 2023-08-14 205401](https://github.com/ua-kxie/circe/assets/56177821/24db33c0-69f5-4187-8e41-38a495a6aecc)

### Setup:
1. Clone the repository
2. Run `git submodule init`
2. Run `git submodule update`
3. Run `cargo run`

Alternatively, clone the repository with the following:

`git clone --recurse-submodules https://github.com/ua-kxie/circe.git`

followed by `cargo run`

### Controls: 
* left click wires or device to select  
* mouse wheel to zoom and pan  
* F key to fit viewport to geometry
* right click drag to zoom to area  
* left click drag to select area
* left click drag on selected device to drag selected
* select single device to edit parameter (wonky) - if you have ideas for implementing a properties editor, get in touch ~
  
#### Hotkeys:

##### Schematic Controls:

Ctrl-C/left-click - copy/paste

Shift-C - cycle tentative selection

Del - delete selected

M - move selected

X/Y - flip horizontal/vertical during move
##### Circuit Schematic:

Shift-L - net label (does no thing for now)

W - draw wire

R - resistor (during move/copy, rotates selected, ctrl-R to counter rotate)

L - inductor

C - capacitor

G - ground

V - voltage source

N - nmos device

P - pmos device

Space - run dc op simulation

ctrl-space - run ac simulation

Shift T - run transient simulation

##### Designer Schematic
-for now, intended for dev use only-

W - draw a line

A - draw an arc/circle

P - place port

B - define device boundary

##### Plot/chart view
(shift) X - horizontal zoom

(shift) Y - vertical zoom 

### Goals
Target application is EDA schematic capture

### Contribute:
Consider using `cargo fmt` & `cargo fix`.

Looking for experienced help with architecture and GUI.
