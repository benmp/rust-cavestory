extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;

use std::path::Path;
use std::time::{Duration, Instant};

const GAME_FRAMERATE: u32 = 60;
const NS_IN_SEC: u64 = 1_000_000_000;

pub fn main() {
    let sdl_context = match sdl2::init() {
        Ok(sdl_context) => sdl_context,
        Err(err) => panic!("failed to create sdl context: {}", err),
    };

    let video_subsystem = match sdl_context.video() {
        Ok(video_subsystem) => video_subsystem,
        Err(err) => panic!("failed to create video subsystem: {}", err),
    };

    let ttf_context = match sdl2::ttf::init() {
        Ok(ttf_context) => ttf_context,
        Err(err) => panic!("failed to create ttf context: {}", err),
    };

    let window = match video_subsystem.window("rust-sdl2 demo", 800, 600).build() {
        Ok(window) => window,
        Err(err) => panic!("failed to create window: {}", err),
    };

    let mut event_pump = match sdl_context.event_pump() {
        Ok(event_pump) => event_pump,
        Err(err) => panic!("failed to get event pump: {}", err),
    };

    let font_path: &Path = Path::new("ttf/DejaVuSansMono.ttf");
    let mut font = match ttf_context.load_font(font_path, 128) {
        Ok(font) => font,
        Err(err) => panic!("failed to load font: {}", err),
    };
    font.set_style(sdl2::ttf::STYLE_BOLD);

    let mut canvas = match window.into_canvas().build() {
        Ok(canvas) => canvas,
        Err(err) => panic!("failed to load canvas: {}", err),
    };
    let texture_creator = canvas.texture_creator();

    //set the background color to be drawn when .clear() is called
    canvas.set_draw_color(Color::RGBA(195, 217, 255, 255));
    //clear now so program first render has this bg color
    canvas.clear();

    //I think this is server "tick" rate if tied to web traffic
    let ns_per_physics_step = NS_IN_SEC as u64 / GAME_FRAMERATE as u64; // how many ns per physics frame
    let mut ns_total_physics_running_time = 0u64;

    let mut top_of_previous_game_loop_start_time = Instant::now();
    let mut ns_left_to_simulate = 0u64;

    let mut top_of_current_game_loop_start_time: Instant;
    let mut elapsed_time_since_previous_game_loop: Duration;

    let mut partial_progress_to_next_frame: f64;

    let mut s_since_last_render = Instant::now();

    let mut physic_loop_count = 0u64;
    let mut render_loop_count = 0u64;

    'running: loop {
        //new time at start of loop
        top_of_current_game_loop_start_time = Instant::now();

        //time since last start of loop (end of last physics cycle + render)
        elapsed_time_since_previous_game_loop = top_of_current_game_loop_start_time
            .duration_since(top_of_previous_game_loop_start_time);

        //prevent spiral of death, run physics at 10fps if necessary
        // if (frame_time.subsec_nanos * 1_000_000.00) > 100.0 {
        //     frame_time = 100.0;
        // }

        // store time for next loop
        top_of_previous_game_loop_start_time = top_of_current_game_loop_start_time;

        //accumulator represents how many ns have passed since last physics run
        ns_left_to_simulate += elapsed_time_since_previous_game_loop.as_secs() * NS_IN_SEC
            + elapsed_time_since_previous_game_loop.subsec_nanos() as u64;

        //Run game engine at specific ms steps
        while ns_left_to_simulate >= ns_per_physics_step {
            //we are now inside a single physics frame
            //drain events per physics frame
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    _ => {}
                }
            }

            //call game Systems here
            //let the systems know total running time, total physics time, and current ns per physics step
            //runsystems(total_physics_running_time_ns, ns_per_physics_step)

            //update total physics time and calculate how many physics steps are left
            ns_total_physics_running_time += ns_per_physics_step;
            ns_left_to_simulate -= ns_per_physics_step;

            physic_loop_count += 1;
        }

        //partial_progress_to_next_frame represents how close we are to the next complete frame (0% to 100%)
        //later used for interpolation of previous state with current state renderer
        partial_progress_to_next_frame = ns_left_to_simulate as f64 / ns_per_physics_step as f64;

        //todo accumulate and average theses calculations in an fps entity
        if Instant::now().duration_since(s_since_last_render).as_secs() > 0 {
            //todo this is technically fps based on previous frame, but whatever?
            let ns_previous_loop_duration = elapsed_time_since_previous_game_loop.as_secs()
                * NS_IN_SEC
                + elapsed_time_since_previous_game_loop.subsec_nanos() as u64;

            //Render
            let surface = font
                .render(&((NS_IN_SEC / ns_previous_loop_duration).to_string()))
                .blended(Color::RGBA(255, 0, 0, 255))
                .unwrap();
            let texture = texture_creator
                .create_texture_from_surface(&surface)
                .unwrap();

            //only need to clear once we add new textures
            canvas.clear();
            canvas.copy(&texture, None, None).unwrap();

            s_since_last_render = Instant::now();
        }

        //draw all copied textures
        canvas.present();
        render_loop_count += 1;
    }

    println!("physics count: {}", physic_loop_count);
    println!(
        "render count: {}, avg fps: {}",
        render_loop_count,
        render_loop_count / (ns_total_physics_running_time / NS_IN_SEC)
    );
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

    let cx = (800 as i32 - w) / 2;
    let cy = (600 as i32 - h) / 2;
    rect!(cx, cy, w, h)
}
