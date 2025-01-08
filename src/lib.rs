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
    receiver : Option<Receiver<(String, Value)>>
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

            WindowEvent::RedrawRequested => {
                    if let Some(surface) = &mut self.surface {
                        let size = self.window.as_ref().unwrap().inner_size();
                        if let (Some(width), Some(height)) = (
                            NonZeroU32::new(size.width),
                            NonZeroU32::new(size.height),
                        ) {
                            surface.resize(width, height).unwrap();
                            let mut buffer = surface.buffer_mut().unwrap();
                            let buffer_width = width.get() as usize;
                            let buffer_height = height.get() as usize;

                            // Fill the buffer with a gradient
                            for y in 0..buffer_height {
                                for x in 0..buffer_width {
                                    let red = (x as usize * 255 / buffer_width) as u32;
                                    let green = (y as usize * 255 / buffer_height) as u32;
                                    let blue = 128;
                                    buffer[y * buffer_width + x] =
                                        (red << 16) | (green << 8) | blue;
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
pub extern "Rust" fn create_window(values : HashMap<String, Value>) -> Value{


    let (tx, rx) = mpsc::channel::<(String, Value)>();
   

    RENDER_THREAD_SENDER.get_or_init(|| tx.clone());

    RENDER_THREAD.get_or_init(move || spawn(|| {
        let event_loop = EventLoop::builder().with_any_thread(true).build().unwrap();

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        let mut app = App::from_receiver(rx);

        let _ = event_loop.run_app(&mut app);
    }));
    
    println!("the thing runs now!");

    Value::nil()
}


#[no_mangle]
pub extern "Rust" fn value_map() -> HashMap<String, Value>{
    let mut map = HashMap::new();

    Value::lib_function("create_window", vec![], None, None).insert_to(&mut map);

    map
}
