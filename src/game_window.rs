use piston_window::{
    PistonWindow,
    EventLoop,
    Size,
};
use sdl2_window::Sdl2Window;

use crate::direction::Direction;

pub struct GameWindow {
    window: piston_window::PistonWindow<Sdl2Window>,
    full_screen: bool,
    vm_rect_size: Size,
    rotation: Direction,
    pixel_scale: f64,
    margin: f64,
    f_count: i32,
}

#[allow(dead_code)]
impl GameWindow {
    pub fn new<F1: Into<f64>, F2: Into<f64>, F3: Into<f64>>(
        video_subsystem: sdl2::VideoSubsystem,
        full_screen: bool,
        vm_rect_size: (F1, F1),
        rotation: Direction,
        pixel_scale: F2,
        margin: F3,
    ) -> Self {
        let vm_rect_size = Size { width: vm_rect_size.0.into(), height: vm_rect_size.1.into() };
        let pixel_scale = pixel_scale.into();
        let margin = margin.into();
        let view_rect = {
            let (width, height) = (vm_rect_size.width * pixel_scale, vm_rect_size.height * pixel_scale);
            match rotation {
                Direction::Up    | Direction::Down => (width, height),
                Direction::Right | Direction::Left => (height, width),
            }
        };
        let mut window: PistonWindow<Sdl2Window> = {
            const OPENGL_VER: piston_window::OpenGL = piston_window::OpenGL::V3_2;
            let window_title = format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            let window_rect_size = if full_screen {
                [8192.0, 8192.0]
            } else {
                [view_rect.0 + margin * 2.0, view_rect.1 + margin * 2.0]
            };
            let window_setting = piston_window::WindowSettings::new(&window_title, window_rect_size)
                .samples(0)
                .fullscreen(full_screen)
                .exit_on_esc(true)
                .graphics_api(OPENGL_VER)
                .vsync(true)
                .resizable(false)
                .decorated(true)
                .controllers(true)
            ;
            let sdl2_window = Sdl2Window::with_subsystem(
                video_subsystem,
                &window_setting,
            ).unwrap();
            PistonWindow::new(OPENGL_VER, 0, sdl2_window)
        };
        window.set_max_fps(120);
        window.set_ups(60);
        window.set_ups_reset(0);
        window.set_swap_buffers(true);
        window.set_bench_mode(false);
        window.set_lazy(false);
        GameWindow {
            window,
            full_screen,
            vm_rect_size,
            rotation,
            pixel_scale,
            margin,
            f_count: 0,
        }
    }

    pub fn window(&mut self) -> &piston_window::PistonWindow<Sdl2Window> {
        &self.window
    }

    pub fn mut_window(&mut self) -> &mut piston_window::PistonWindow<Sdl2Window> {
        &mut self.window
    }

    pub fn full_screen(&self) -> bool {
        self.full_screen
    }

    pub fn vm_rect_size(&self) -> Size {
        self.vm_rect_size
    }

    pub fn rotation(&self) -> Direction {
        self.rotation
    }

    pub fn set_rotation(&mut self, rotation: Direction) {
        self.rotation = rotation;
    }

    pub fn turn_left(&mut self) {
        self.rotation = self.rotation.turn_left();
        self.f_count = 0;
    }

    pub fn turn_right(&mut self) {
        self.rotation = self.rotation.turn_right();
        self.f_count = 0;
    }

    pub fn pixel_scale(&self) -> f64 {
        self.pixel_scale
    }

    pub fn margin(&self) -> f64 {
        self.margin
    }

    pub fn f_count(&self) -> i32 {
        self.f_count
    }

    pub fn set_f_count(&mut self, n: i32) {
        self.f_count = n;
    }

    pub fn inc_f_count(&mut self) {
        self.f_count += 1;
    }
}
