# Mjolnir

Mjolnir is a library for displaying graphics in thorlang.

## Installation 
---
You can either build from source: 
```bash
git clone https://github.com/Adotweb/mjolnir 
cd mjolnir 
chmod +x ./install.sh
./install.sh
cp ./reacthor.so /path/to/your/directory
```

or download the latest version from the github reposoitories `mjolnir.so` file (only on linux

## Usage

### The main event loop
Mjolnir is designed for ease of use, therefore it does not export e `main_loop` function or similar, the event loop is handled through thorlang native concepts. 

To start an event loop we create a window: 

```thor 
//import library 
let mjolnir = import_Lib("mjolnir.so");

let window = mjolnir.create_window();
```

This window will now instantly close, because opening a window does not keep thorlang from exiting, to prevent this, just make a loop:

```thor 
//import library 
let mjolnir = import_Lib("mjolnir.so");

let window = mjolnir.create_window();

//this while loop will keep the window from closing
while(true){

    //animation related things will happen here


    //we call this to actually "request" or "start" a new frame
    window.new_frame();


}
```

### Drawing 

To actually draw something to the screen use the provided methods on the window object (documented under *window object*)



### Window Object
The window object exports a couple of methods by default: 


> `window.sleep(milliseconds)` will halt execution for the amount of milliseconds specified.

> `window.get_delta_time() -> milliseconds` returns the amount of milliseconds that have passed since the last frame

> `window.draw_rect(point_array) | point_array = [[x1, x2], [y1, y2]]` 
will draw a rectangle starting at the first coordinate (top-left) to the second coordinate (bottom-right)

> `window.draw_line(point_array) | point_array = [[x1, x2], [y1, y2]]` 
draws a line from the first point to the secondd

> `window.get_screen_dimensions() -> [width, height]` returns the current size of the screen

> `window.set_color(rgb) | rgb = [r < 255, g < 255, b < 255]` sets the current color of the "pen" 
(is black by default so remember to set a color if you want to see someting)


> `window.new_frame()` and `window.flush()` requests a new frame, while flush also clears the currently drawn to screen. 

### Examples
An example of mjolnir usage can be found in the `example.thor` file.


