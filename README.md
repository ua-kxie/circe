Circe: Prototyping for 2D CAD drawn with Iced canvas

### Preview:
![Capture](https://github.com/ua-kxie/circe/assets/56177821/467531f5-45cc-4690-8f6d-2a49444faafe)
![Capture1](https://github.com/ua-kxie/circe/assets/56177821/2f6394cb-83af-496b-bf8f-24039f3558fc)


### Setup:
1. Clone the repository
2. Run `git submodule init`
2. Run `git submodule update`
3. Run `cargo run`

Alternatively, clone the repository with the following:

`git clone --recurse-submodules https://github.com/ua-kxie/circe.git`

followed by `cargo run`

### Controls: 
* click wires or device to select  
* mouse wheel to zoom and pan  
* right click drag to zoom to area  
* left click drag to select area
* left click drag on selected device to drag selected
* select single device to edit parameter (wonky) - if you have ideas for implementing a properties editor, get in touch ~
  
#### Hotkeys:
##### Circuit Schematic:
L - net label (does no thing for now)

W - draw wire

F - fit viewport to geometry

C - cycle tentative selection

Del - delete selected

R - resistor (during move/copy, rotates selected, ctrl-R to counter rotate)

G - ground

V - voltage source

N - nmos device

P - pmos device

D - new devices hotkeys

M - move selected

Space - run dc op simulation

Shift T - run transient simulation (10u 1m)

##### Plot/chart view
(shift) X - horizontal zoom

(shift) Y - vertical zoom 

F - fit viewport to geometry

##### device designer view
-for now, intended for dev use only-

W - draw a line

A - draw an arc/circle

P - place port

B - define device boundary

### Goals
Target application is EDA schematic capture

### Contribute:
Consider using `cargo fmt` & `cargo fix`.

Looking for experienced help with architecture and GUI.
