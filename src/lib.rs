use type_lib::{Value, ValueType};

use std::collections::HashMap;

use std::thread::{spawn, JoinHandle};

use std::sync::{mpsc::{Sender, Receiver, self}, Mutex, Arc, OnceLock};

use std::num::NonZeroU32;

static RENDER_THREAD: OnceLock<JoinHandle<()>> = OnceLock::new();
static RENDER_THREAD_SENDER: OnceLock<Sender<(String, Value)>> = OnceLock::new();



use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopBuilder},
    platform::wayland::EventLoopBuilderExtWayland,
    window::{Window, WindowLevel, WindowId}
};

use std::rc::Rc;

use softbuffer::{Context, Surface};

use bytemuck::{Pod, Zeroable};


#[derive(Default)]
struct App{
    window : Option<Rc<Window>>,
    context : Option<Context<Rc<Window>>>,
    surface : Option<Surface<Rc<Window>, Rc<Window>>>,
    receiver : Option<Receiver<(String, Value)>>,
  

    screen_size : [usize; 2],
    //buffer of pixels we draw to
    screen_buffer : Vec<u32>
}

impl App{
    fn from_receiver(rec : Receiver<(String, Value)>) -> Self{
        App{
            receiver : Some(rec), 
            ..Default::default()
        }
    }
}

impl ApplicationHandler for App{

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
   
        if self.window.is_none(){

            let window_attributes = Window::default_attributes().with_title("hello");
            let window = Rc::new(event_loop.create_window(window_attributes).unwrap());

            let context = Context::new(window.clone()).unwrap();
            let surface = Surface::new(&context, window.clone()).unwrap();

            let size = window.inner_size();
            self.screen_buffer.resize(size.width as usize * size.height as usize, 0);
            self.screen_size = [size.width as usize, size.height as usize];
            self.window = Some(window);
            self.context = Some(context);
            self.surface = Some(surface);
        }

    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {


        if let Some(receiver) = &self.receiver{
            if let Ok(rec) = receiver.try_recv(){
                match rec.0.as_str(){
                    "set_pixel" => {
                            let pixel_coordinates = rec.1;
                        
                            if let ValueType::Array(arr) = pixel_coordinates.value{
                           
                                let x = arr[0].clone().to_f64().unwrap() as usize;
                                let y = arr[1].clone().to_f64().unwrap() as usize;
                                let color = arr[2].clone().to_arr().unwrap();

                                let r = color[0].to_f64().unwrap() as u32;
                                let g = color[1].to_f64().unwrap() as u32;
                                let b = color[2].to_f64().unwrap() as u32;

                                self.screen_buffer[
                                    y * self.screen_size[0] + x
                                ]  = r >> 16 | g >> 8 | b;

                            } else {
                                println!("pixel coordinates need to be arrays")
                            }
                             

                    },
                    "draw_rect" => {
                        let pixel_coordinates = rec.1;
                        
                        let arr = pixel_coordinates.to_arr().unwrap();

                        let p1 = arr[0].clone().to_arr().unwrap();
                        let x1 = p1[0].clone().to_f64().unwrap() as usize;
                        let y1 = p1[1].clone().to_f64().unwrap() as usize;

                        let p2 = arr[0].clone().to_arr().unwrap();
                        let x2 = p2[0].clone().to_f64().unwrap() as usize;
                        let y2 = p2[1].clone().to_f64().unwrap() as usize;

                        for x in x1..x2{
                            for y in y1..y2{
                                self.screen_buffer[
                                    y * self.screen_size[0] + x
                                ] = (255 << 16) | (255 << 8) | 255
                            }
                        }

                        self.window.as_ref().unwrap().request_redraw();
                    },
                    _ => println!("some message!")
                }
            }
        }


        match event{
            WindowEvent::Resized(size) => {
                println!("resize to {:?}", size);
            },

            WindowEvent::CloseRequested => {
                 
                println!("the window was closed");
                self.window = None;
                event_loop.exit()
            },
            WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
                println!("{:?} clicked, registered as {:?}", event.physical_key, event.logical_key);
            },
            WindowEvent::RedrawRequested => {
                    if let Some(surface) = &mut self.surface {
                        let size = self.window.as_ref().unwrap().inner_size();
                        if let (Some(width), Some(height)) = (
                            NonZeroU32::new(size.width),
                            NonZeroU32::new(size.height),
                        ) {
                            surface.resize(width, height).unwrap();


                            let mut buffer = surface.buffer_mut().unwrap();
                            let buffer_width : usize = width.get() as usize;
                            let buffer_height : usize = height.get() as usize;
                           
                            self.screen_size[0] = buffer_width;
                            self.screen_size[1] = buffer_height;

                            let old_screen_buffer = self.screen_buffer.clone();

                            self.screen_buffer.resize(buffer_width * buffer_height, 0);
                        

                            for (index, pixel) in old_screen_buffer.iter().enumerate(){
                                    self.screen_buffer[index] = pixel.clone(); 
                            }


                            // Fill the buffer with a gradient
                            for y in 0..buffer_height {
                                for x in 0..buffer_width {
                                    let color = self.screen_buffer[y * buffer_width + x];

                                    if color != 0 {
                                        println!("some color at : {x},{y}")
                                    }
                                    buffer[y * buffer_width + x] = color;
                                }
                            }
                            buffer.present().unwrap();
                        }
                    }

                    self.window.as_ref().unwrap().request_redraw();
 
            },
            _ => ()
        }

    }
}


#[no_mangle]
pub extern "Rust" fn buf_append(values : HashMap<String, Value>) -> Value{

    println!("buf append");

    Value::nil()
}

#[no_mangle]
pub extern "Rust" fn set_pixel(values : HashMap<String, Value>) -> Value{

    let pixel_info = values.get("pixel_info").unwrap(); 

    let sender = RENDER_THREAD_SENDER.clone();

    let _ = sender.get().unwrap().send(
        ("set_pixel".to_string(), pixel_info.clone())
    ).unwrap();

    Value::nil()
}

#[no_mangle]
pub extern "Rust" fn draw_rect(values : HashMap<String, Value>) -> Value{

    let pixel_coords = values.get("pixel_coords").unwrap(); 

    let sender = RENDER_THREAD_SENDER.clone();

    let _ = sender.get().unwrap().send(
        ("draw_rect".to_string(), pixel_coords.clone())
    ).unwrap();

    Value::nil()
}


#[no_mangle]
pub extern "Rust" fn create_window(values : HashMap<String, Value>) -> Value{


    let (tx, rx) = mpsc::channel::<(String, Value)>();
   

    RENDER_THREAD_SENDER.get_or_init(|| tx.clone());

    RENDER_THREAD.get_or_init(move || spawn(|| {
        let event_loop = EventLoop::builder().with_any_thread(true).build().unwrap();

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        let mut app = App::from_receiver(rx);

        let _ = event_loop.run_app(&mut app);
    }));
    

    Value::nil()
}


#[no_mangle]
pub extern "Rust" fn value_map() -> HashMap<String, Value>{
    let mut map = HashMap::new();

    Value::lib_function("create_window", vec![], None, None).insert_to(&mut map);

    Value::lib_function("buf_append", vec![], None, None).insert_to(&mut map);

    Value::lib_function("set_pixel", vec!["pixel_info"], None, None).insert_to(&mut map);
    Value::lib_function("draw_rect", vec!["pixel_coords"], None, None).insert_to(&mut map);

    map
}
