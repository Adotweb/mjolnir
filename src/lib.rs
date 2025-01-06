use type_lib::{Value, ValueType};

use std::collections::HashMap;

use std::thread::{spawn, JoinHandle};

use std::sync::{mpsc::{Sender, Receiver, self}, Mutex, Arc, OnceLock};


static RENDER_THREAD: OnceLock<JoinHandle<()>> = OnceLock::new();
static RENDER_THREAD_SENDER: OnceLock<Sender<(String, Value)>> = OnceLock::new();


use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopBuilder},
    platform::wayland::EventLoopBuilderExtWayland,
    window::{Window, WindowLevel, WindowId}
};

use bytemuck::{Pod, Zeroable};


#[derive(Default)]
struct App{
    window : Option<Window>,
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


            self.window = Some(event_loop.create_window(window_attributes).unwrap())
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

        println!("new thing event");

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
