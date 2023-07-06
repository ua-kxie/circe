Circe: Prototyping for 2D CAD drawn with Iced canvas

### Preview:
![Capture](https://github.com/ua-kxie/circe/assets/56177821/467531f5-45cc-4690-8f6d-2a49444faafe)

### Setup:
1. Clone the repository
2. Run `git submodule init`
2. Run `git submodule update`
3. Run `cago run`

Alternatively, clone the repository with the following:
> `git clone --recurse-submodules https://github.com/ua-kxie/circe.git`

followed by 
> `cargo run`

### Controls: 
* click wires or device to select  
* mouse wheel to zoom and pan  
* right click drag to zoom to area  
* left click drag to select area
* left click drag on selected device to drag selected
* select single device to edit parameter (wonky)  
  
#### Hotkeys:

W - draw wire

F - fit viewport to geometry

C - cycle tentative selection

Del - delete selected

R - resistor, rotate selected during move, placement

G - ground

V - voltage source

M - move selected

Space - run dc op simulation  


Target application is EDA schematic capture

### Contribute:
use `cargo fmt`.
