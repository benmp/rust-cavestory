extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;
use sdl2::pixels::Color;

use std::time::{Duration, Instant};
use std::path::Path;

const GAME_FRAMERATE: u32 = 60;

pub fn main() {
    let sdl_context = match sdl2::init() {
        Ok(sdl_context) => sdl_context,
        Err(err) => panic!("failed to create sdl context: {}", err)
    };

    let video_subsystem = match sdl_context.video() {
        Ok(video_subsystem) => video_subsystem,
        Err(err) => panic!("failed to create video subsystem: {}", err)
    };

    let ttf_context = match sdl2::ttf::init() {
        Ok(ttf_context) => ttf_context,
        Err(err) => panic!("failed to create ttf context: {}", err)
    };

    let window = match video_subsystem.window("rust-sdl2 demo", 800, 600).build() {
        Ok(window) => window,
        Err(err) => panic!("failed to create window: {}", err)
    };

    let mut canvas = match window.into_canvas().build() {
        Ok(canvas) => canvas,
        Err(err) => panic!("failed to load canvas: {}", err)
    };
    let texture_creator = canvas.texture_creator();

    let mut event_pump = match sdl_context.event_pump() {
        Ok(event_pump) => event_pump,
        Err(err) => panic!("failed to get event pump: {}", err)
    };

    let font_path: &Path = Path::new("ttf/Hack-Regular.ttf");
    let mut font = match ttf_context.load_font(font_path, 128) {
        Ok(font) => font,
        Err(err) => panic!("failed to load font: {}", err)
    };

    let surface = font.render("HELLO").blended(Color::RGBA(255, 0, 0, 255)).unwrap();
    let teture = texture_creator.create_texture_from_surface(&surface).unwrap();

    canvas.set_draw_color(Color::RGBA(195, 217, 255, 255));
    canvas.clear();

    let mut total_running_time_ms = 0;
    let ms_per_physics_step = 1_000 / GAME_FRAMERATE as u32; // how many ms per physic frame

    let mut current_time = Instant::now();
    let mut ms_left_to_simulate = 0;

    let mut new_time: Instant;
    let mut time_since_last_frame: Duration;

    let mut partial_progress_to_next_frame;

    let mut i = 0;
    'running: loop {

        //new time at start of loop
        new_time = Instant::now();

        //time since last start of loop (end of last physics cycle + render)
        time_since_last_frame = new_time.duration_since(current_time);

        //prevent spiral of death, run physics at 10fps if necessary
        //if (frame_time.subsec_nanos * 1_000_000.00) > 100.0 {
        // frame_time = 100.0;
        // }

        // store current_time for next loop
        current_time = new_time;

        //accumulator represents how many ms have passed since last physics run
        ms_left_to_simulate += time_since_last_frame.subsec_nanos() * 1_000_000;

        //Run game engine at specific ms steps
        while ms_left_to_simulate >= ms_per_physics_step {
            //we are now inside a single physics frame
            //drain events per physics frame
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                        Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                            break 'running
                        },
                    _ => {}
                }
            }

            //call game Systems here
            //let the system know how long its been running and how long the current step is in ms
            //runsystems(t, dt)

            total_running_time_ms += ms_per_physics_step;
            ms_left_to_simulate -= ms_per_physics_step;
        }

        //partial_progress_to_next_frame represents how close we are to the next complete frame (0% to 100%)
        partial_progress_to_next_frame = ms_left_to_simulate as f32 / ms_per_physics_step as f32;

        //Render
        i = (i as f32 + (1.0 * partial_progress_to_next_frame)) as u8 % 255;
        canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        canvas.clear();
        canvas.present();
    }
}

// handle the annoying Rect i32
macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
        )
    );

// Scale fonts to a reasonable size when they're too big (though they might look less smooth)
fn get_centered_rect(rect_width: u32, rect_height: u32, cons_width: u32, cons_height: u32) -> Rect {
    let wr = rect_width as f32 / cons_width as f32;
    let hr = rect_height as f32 / cons_height as f32;

    let (w, h) = if wr > 1f32 || hr > 1f32 {
        if wr > hr {
            println!("Scaling down! The text will look worse!");
            let h = (rect_height as f32 / wr) as i32;
            (cons_width as i32, h)
        } else {
            println!("Scaling down! The text will look worse!");
            let w = (rect_width as f32 / hr) as i32;
            (w, cons_height as i32)
        }
    } else {
        (rect_width as i32, rect_height as i32)
    };

    let cx = (SCREEN_WIDTH as i32 - w) / 2;
    let cy = (SCREEN_HEIGHT as i32 - h) / 2;
    rect!(cx, cy, w, h)
}

