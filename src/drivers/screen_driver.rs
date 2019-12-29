use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::pixels::Color;
use sdl2::Sdl;
use sdl2::rect::Rect;

pub const CHIP8_WIDTH:usize = 64;
pub const CHIP8_HEIGHT:usize = 32;
const SCALE_FACTOR: u32 = 20;
const SCREEN_WIDTH: u32 = (CHIP8_WIDTH as u32) * SCALE_FACTOR;
const SCREEN_HEIGHT: u32 = (CHIP8_HEIGHT as u32) * SCALE_FACTOR;

const OFF_COLOUR:(u8, u8, u8) = (255, 255, 255);
const ON_COLOUR: (u8, u8, u8) = (0, 0, 0);

pub struct ScreenDriver{
    canvas: Canvas<Window>
}

impl ScreenDriver{
    pub fn new(sdl_context: &Sdl) -> ScreenDriver{
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem.window("Chip8 Emulator by Braydn Moore",
                                            SCREEN_WIDTH,
                                            SCREEN_HEIGHT)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        ScreenDriver{
            canvas
        }
    }

    pub fn draw(&mut self, screen: &[[u8; CHIP8_WIDTH]; CHIP8_HEIGHT]){
        // for every pixel in the screen array
        for (y, row) in screen.iter().enumerate() {
            for (x, &col) in row.iter().enumerate() {
                // get the x y coordinates on the scaled up screen
                let x = (x as u32) * SCALE_FACTOR;
                let y = (y as u32) * SCALE_FACTOR;

                // set the color to draw based on if the pixel is on or off
                self.canvas.set_draw_color(if col == 0 {Color::RGB(OFF_COLOUR.0, OFF_COLOUR.1, OFF_COLOUR.2)}
                                                else {Color::RGB(ON_COLOUR.0, ON_COLOUR.1, ON_COLOUR.2)});
                // draw the rectangle to the canvas
                let _ = self.canvas
                            .fill_rect(Rect::new(x as i32, y as i32, SCALE_FACTOR, SCALE_FACTOR));
            }
        }
        // show the window
        self.canvas.present();
    }
}