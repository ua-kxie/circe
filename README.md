Circe: Prototyping for 2D CAD drawn with Iced canvas

Soliciting experienced help and/or advice with architecture and gui

### Preview:
Simple op-amp with generic devices
![Screenshot 2023-08-14 205401](https://github.com/ua-kxie/circe/assets/56177821/24db33c0-69f5-4187-8e41-38a495a6aecc)

### Setup:
`cargo run`

The binary executable is not working at the moment due to problems finding the sharedspice library.

### Controls: 
* left click wires or device to select  
* mouse wheel to zoom and pan  
* F key to fit viewport to geometry
* right click drag to zoom to area  
* left click drag to select area
* left click drag on selected device to drag selected
* select single device to edit parameter
  
#### Hotkeys:

##### Schematic Controls (circuit schematic and symbol designer):

Ctrl-C/left-click - copy/paste

Shift-C - cycle tentative selection

Del - delete selected

M - move selected

X/Y - flip horizontal/vertical during move
##### Circuit Schematic:

Shift-L - net label (has no effect on net connections atm, is just a comment)

W - draw wire

R - resistor (during move/copy, rotates selected, ctrl-R to counter rotate)

L - inductor

C - capacitor

G - ground

V - voltage source

N - nmos device

P - pmos device

Space - run dc op simulation

Ctrl-space - run ac simulation

Shift-T - run transient simulation

##### Symbol Designer
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

#### Currently Working On:
* improved wiring pathfinding (petgraph)
* device/wire drag/grab keeping net connections
* connect overlapping ports with wiring
* net labels

### Contribute:
Consider using `cargo fmt` & `cargo fix`.

Looking for experienced help with architecture and GUI.
