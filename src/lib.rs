use type_lib::{Value, ValueType};

use std::collections::HashMap;

use std::thread::{spawn, JoinHandle};

use std::time::{Duration, Instant};

use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc, Mutex, OnceLock,
    RwLock
};

use std::num::NonZeroU32;

static RENDER_THREAD: OnceLock<JoinHandle<()>> = OnceLock::new();
static RENDER_THREAD_SENDER: OnceLock<Sender<(String, Value)>> = OnceLock::new();


static DELTA_TIME : OnceLock<RwLock<f64>> = OnceLock::new();


static SCREEN_DIMENSIONS : OnceLock<RwLock<[f64;2]>> = OnceLock::new();



use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopBuilder},
    platform::wayland::EventLoopBuilderExtWayland,
    window::{Window, WindowId, WindowLevel},
};

use std::rc::Rc;

use softbuffer::{Context, Surface};

use bytemuck::{Pod, Zeroable};

#[derive(Default)]
struct App {
    window: Option<Rc<Window>>,
    context: Option<Context<Rc<Window>>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    receiver: Option<Receiver<(String, Value)>>,

    screen_size: [usize; 2],
    //buffer of pixels we draw to
    screen_buffer: Vec<u32>,

    command_queue: Vec<(String, Value)>,


    last_frame_time : Option<Instant>
}

impl App {
    fn from_receiver(rec: Receiver<(String, Value)>) -> Self {
        App {
            receiver: Some(rec),
            last_frame_time : Some(Instant::now()),
            ..Default::default()
        }
    }

    fn apply_queue(&mut self) {
        for rec in self.command_queue.iter() {
            match rec.0.as_str() {
                "flush" => {
                    self.screen_buffer.fill(0);

                }
                "set_pixel" => {
                    let pixel_coordinates = rec.1.clone();

                    if let ValueType::Array(arr) = pixel_coordinates.value {
                        let x = arr[0].clone().to_f64().unwrap() as usize;
                        let y = arr[1].clone().to_f64().unwrap() as usize;
                        let color = arr[2].clone().to_arr().unwrap();

                        let r = color[0].to_f64().unwrap() as u32;
                        let g = color[1].to_f64().unwrap() as u32;
                        let b = color[2].to_f64().unwrap() as u32;

                        self.screen_buffer[y * self.screen_size[0] + x] = r >> 16 | g >> 8 | b;
                    } else {
                        println!("pixel coordinates need to be arrays")
                    }
                }
                "draw_rect" => {
                    let pixel_coordinates = rec.1.clone();

                    let arr = pixel_coordinates.to_arr().unwrap();

                    let p1 = arr[0].clone().to_arr().unwrap();
                    let x1 = p1[0].clone().to_f64().unwrap() as usize;
                    let y1 = p1[1].clone().to_f64().unwrap() as usize;

                    let p2 = arr[1].clone().to_arr().unwrap();
                    let x2 = p2[0].clone().to_f64().unwrap() as usize;
                    let y2 = p2[1].clone().to_f64().unwrap() as usize;


                    for x in x1..x2 {
                        for y in y1..y2 {
                            if let Some(target_index) =
                                self.screen_buffer.get_mut(y * self.screen_size[0] + x)
                            {
                                *target_index = (255 << 16) | (255 << 8) | 255;
                            }
                        }
                    }
                }
                _ => println!("some message!"),
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes().with_title("hello");
            let window = Rc::new(event_loop.create_window(window_attributes).unwrap());

            let context = Context::new(window.clone()).unwrap();
            let surface = Surface::new(&context, window.clone()).unwrap();

            let size = window.inner_size();
            self.screen_buffer
                .resize(size.width as usize * size.height as usize, 0);
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
        if let Some(receiver) = &self.receiver {
            if let Ok(rec) = receiver.try_recv() {

                if rec.0.as_str() == "flush" {
                    let delta = Instant::now() - self.last_frame_time.unwrap();

                    let lock = DELTA_TIME.get().unwrap(); 

                    let mut guard = lock.write().unwrap();
                  
                    *guard = delta.as_secs_f64();
                    
                 
                    self.apply_queue();
                    self.command_queue = vec![("flush".to_string(), Value::nil())];
                    self.window.as_ref().unwrap().request_redraw();

                    self.last_frame_time = Some(Instant::now())
                }

                self.command_queue.push(rec.clone());
            }
        }

        match event {
            WindowEvent::Resized(size) => {
                let buf_w = size.width as usize;
                let buf_h = size.height as usize;
                self.screen_buffer.resize(buf_w * buf_h, 0);

                self.screen_size = [buf_w, buf_h];

                let screen_size_lock = SCREEN_DIMENSIONS.get().unwrap();

                let mut screen_size_guard = screen_size_lock.write().unwrap();


                *screen_size_guard = [buf_w as f64, buf_h as f64];

                if let Some(surface) = &mut self.surface {
                    let _ = surface.resize(
                        NonZeroU32::new(buf_w as u32).unwrap(),
                        NonZeroU32::new(buf_h as u32).unwrap(),
                    );
                    let _ = surface.buffer_mut().unwrap().present();
                }
            }

            WindowEvent::CloseRequested => {
                self.window = None;
                event_loop.exit()
            }
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {}
            WindowEvent::RedrawRequested => {
                if let Some(surface) = &mut self.surface {
                    let size = self.window.as_ref().unwrap().inner_size();
                    if let (Some(width), Some(height)) =
                        (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                    {

                        let mut buffer = surface.buffer_mut().unwrap();


                        let buffer_width = self.screen_size[0];
                        let buffer_height = self.screen_size[1];
                        self.screen_buffer.resize(buffer_width * buffer_height, 0);

                        // Fill the buffer with a gradient
                        for y in 0..buffer_height {
                            for x in 0..buffer_width {
                                let color = self.screen_buffer[y * buffer_width + x];

                                buffer[y * buffer_width + x] = color;
                            }
                        }
                        buffer.present().unwrap();
                    }
                }

                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}



#[no_mangle]
pub extern "Rust" fn get_screen_dimensions(values: HashMap<String, Value>) -> Value {
    

    let screen_size_lock = SCREEN_DIMENSIONS.get().unwrap(); 
    
    let screen_size_guard = screen_size_lock.read().unwrap();

    let screen_size = (*screen_size_guard).clone();

    
    Value::array(screen_size.iter().map(|x| Value::number(*x)).collect())
}


#[no_mangle]
pub extern "Rust" fn get_delta_time(values: HashMap<String, Value>) -> Value{
       

    let delta_lock = DELTA_TIME.get().unwrap();


    let delta = delta_lock.read().unwrap().clone();
    

     
    Value::number(delta)
}

#[no_mangle]
pub extern "Rust" fn sleep(values: HashMap<String, Value>) -> Value{
       
    let sleep_duration = values.get("sleep_duration").unwrap().to_f64().expect("sleep duration must be a number in ms"); 

    std::thread::sleep(Duration::from_millis(sleep_duration as u64));
     
    Value::nil()
}

#[no_mangle]
pub extern "Rust" fn set_pixel(values: HashMap<String, Value>) -> Value {
    let pixel_info = values.get("pixel_info").unwrap();

    let sender = RENDER_THREAD_SENDER.clone();

    let _ = sender
        .get()
        .unwrap()
        .send(("set_pixel".to_string(), pixel_info.clone()))
        .unwrap();

    Value::nil()
}

#[no_mangle]
pub extern "Rust" fn draw_rect(values: HashMap<String, Value>) -> Value {
    let pixel_coords = values.get("pixel_coords").unwrap();

    let sender = RENDER_THREAD_SENDER.clone();

    let _ = sender
        .get()
        .unwrap()
        .send(("draw_rect".to_string(), pixel_coords.clone()))
        .unwrap();

    Value::nil()
}

#[no_mangle]
pub extern "Rust" fn flush(values: HashMap<String, Value>) -> Value {
    let sender = RENDER_THREAD_SENDER.clone();

    let _ = sender
        .get()
        .unwrap()
        .send(("flush".to_string(), Value::nil()))
        .unwrap();

    Value::nil()
}

#[no_mangle]
pub extern "Rust" fn create_window(values: HashMap<String, Value>) -> Value {
    let (tx, rx) = mpsc::channel::<(String, Value)>();

    RENDER_THREAD_SENDER.get_or_init(|| tx.clone());

    DELTA_TIME.get_or_init(|| RwLock::new(0.0));
    

    SCREEN_DIMENSIONS.get_or_init(|| RwLock::new([0.0, 0.0]));

    RENDER_THREAD.get_or_init(move || {
        spawn(|| {
            let event_loop = EventLoop::builder().with_any_thread(true).build().unwrap();

            event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
            let mut app = App::from_receiver(rx);

            let _ = event_loop.run_app(&mut app);
        })
    });

    Value::nil()
}

#[no_mangle]
pub extern "Rust" fn value_map() -> HashMap<String, Value> {
    let mut map = HashMap::new();

    Value::lib_function("create_window", vec![], None, None).insert_to(&mut map);

    Value::lib_function("buf_append", vec![], None, None).insert_to(&mut map);

    Value::lib_function("set_pixel", vec!["pixel_info"], None, None).insert_to(&mut map);
    Value::lib_function("draw_rect", vec!["pixel_coords"], None, None).insert_to(&mut map);
    Value::lib_function("flush", vec![], None, None).insert_to(&mut map);
    Value::lib_function("get_delta_time", vec![], None, None).insert_to(&mut map);
    Value::lib_function("sleep", vec!["sleep_duration"], None, None).insert_to(&mut map);

    Value::lib_function("get_screen_dimensions", vec![], None, None).insert_to(&mut map);

    map
}
