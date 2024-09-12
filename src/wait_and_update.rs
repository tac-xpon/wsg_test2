use crate::{
    direction::*,
    input_role::*,
    GameWindow,
};
use bgsp_lib2::{
    bg_plane::*,
    sp_resources::*
};
use piston_window::*;

pub fn doing(
    game_window: &mut GameWindow,
    spr: &mut SpResources,
    bg: &mut (BgPlane, BgPlane),
    keyboard_map: &mut InputRoleMap<Key>,
    button_map: &mut InputRoleMap<ControllerButton>,
    hat_map: &mut InputRoleMap<ControllerHat>,
    mousebutton_map: &mut InputRoleMap<MouseButton>,
    pointer_pos: &mut (f64, f64),
) -> bool {
    let mut texture_context = game_window.mut_window().create_texture_context();
    let texture_settings = TextureSettings::new();
    bg.0.rendering();
    let bg0_whole = Texture::from_image(
        &mut texture_context,
        bg.0.whole_image(),
        &texture_settings,
    ).unwrap();
    bg.1.rendering();
    let bg1_whole = Texture::from_image(
        &mut texture_context,
        bg.1.whole_image(),
        &texture_settings,
    ).unwrap();
    // Sprites
    let sp_drawn = Texture::from_image(
        &mut texture_context,
        &spr.rendering(game_window.vm_rect_size().width as i32, game_window.vm_rect_size().height as i32),
        &texture_settings,
    ).unwrap();

    while let Some(event) = game_window.mut_window().next() {
        if let Some(pos) = event.mouse_cursor_args() {
            pointer_pos.0 = pos[0];
            pointer_pos.1 = pos[1];
        }
        if let Some(input) = event.button_args() {
            match input.button {
                Button::Keyboard(k) => {
                    keyboard_map.update_state(k, input.state == ButtonState::Press);
                }
                Button::Mouse(b) => {
                    mousebutton_map.update_state(b, input.state == ButtonState::Press);
                }
                Button::Controller(b) => {
                    button_map.update_state_exclusive(b, input.state == ButtonState::Press);
                }
                Button::Hat(h) => {
                    hat_map.update_state_exclusive(h, true);
                }
            }
        }
        if let Event::Loop(Loop::Render(_)) = event {
            let vm_rect_size = game_window.vm_rect_size();
            let window_size = game_window.window().size();
            let rotation = game_window.rotation();
            let pixel_scale = game_window.pixel_scale();
            let margin_2x = game_window.margin() * 2.0;
            let f_count = game_window.f_count();
            game_window.mut_window().draw_2d(&event, |context, graphics, _device| {
                let transform = {
                    let (zoom, h_offset, v_offset) = {
                        let view_rect = {
                            let (width, height) = (vm_rect_size.width * pixel_scale, vm_rect_size.height * pixel_scale);
                            match rotation {
                                Direction::Up    | Direction::Down => (width, height),
                                Direction::Right | Direction::Left => (height, width),
                            }
                        };
                        let h_zoom = window_size.width / (view_rect.0 + margin_2x);
                        let v_zoom = window_size.height / (view_rect.1  + margin_2x);
                        let zoom = h_zoom.min(v_zoom);
                        let h_offset = (window_size.width - view_rect.0 * zoom) / 2.0;
                        let v_offset = (window_size.height - view_rect.1 * zoom) / 2.0;
                        (zoom, h_offset, v_offset)
                    };
                    let base_transform = context.transform.zoom(zoom).trans(h_offset / zoom, v_offset / zoom);
                    match rotation {
                        Direction::Up    => base_transform.rot_deg(  0.0).trans(0.0, 0.0),
                        Direction::Right => base_transform.rot_deg( 90.0).trans(0.0, -(vm_rect_size.height * pixel_scale)),
                        Direction::Down  => base_transform.rot_deg(180.0).trans(-(vm_rect_size.width * pixel_scale), -(vm_rect_size.height * pixel_scale)),
                        Direction::Left  => base_transform.rot_deg(270.0).trans(-(vm_rect_size.width * pixel_scale), 0.0),
                    }
                };
                let draw_inside = draw_state::DrawState::new_inside();
                if f_count < 4 {
                    // Initialize
                    graphics.clear_color([0.0, 0.0, 0.0, 1.0]);
                    graphics.clear_stencil(0);
                    rectangle::Rectangle::new([1.0; 4]).draw(
                        [0.0, 0.0, vm_rect_size.width * pixel_scale, vm_rect_size.height * pixel_scale],
                        &draw_state::DrawState::new_clip(),
                        transform,
                        graphics,
                    );
                } else {
                    // Clear
                    rectangle::Rectangle::new([0.0, 0.0, 0.0, 1.0]).draw(
                        [0.0, 0.0, vm_rect_size.width * pixel_scale, vm_rect_size.height * pixel_scale],
                        &draw_inside,
                        transform,
                        graphics,
                    );
                }
                // BG1
                image::draw_many(
                    bg.1.draw_rects(), [1.0, 1.0, 1.0, 1.0],
                    &bg1_whole,
                    &draw_inside,
                    transform,
                    graphics,
                );
                // Sprites
                image::Image::new().draw(
                    &sp_drawn,
                    &draw_inside,
                    transform,
                    graphics,
                );
                // BG0
                image::draw_many(
                    bg.0.draw_rects(), [1.0, 1.0, 1.0, 1.0],
                    &bg0_whole,
                    &draw_inside,
                    transform,
                    graphics,
                );
            });
            game_window.inc_f_count();
            return false;
        }
    }
    true
}
