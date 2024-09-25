mod bgsp_data;
use bgsp_data::*;

mod direction;
use direction::*;

mod input_role;
use input_role::*;

mod game_window;
use game_window::*;

mod wait_and_update;

use bgsp_lib2::{bg_plane::*, bgsp_common::*, sp_resources::*};

use audio_lib3::*;

mod sound_generator;
use sound_generator::*;

mod sound_manager;
use sound_manager::*;

use piston_window::{ControllerButton, ControllerHat, HatState, Key, MouseButton};

const FULL_SCREEN: bool = false;
const VM_RECT_SIZE: (i32, i32) = (40, 30);
const VM_RECT_PIXEL_SIZE: (i32, i32) = (
    VM_RECT_SIZE.0 * PATTERN_SIZE as i32,
    VM_RECT_SIZE.1 * PATTERN_SIZE as i32,
);
const ROTATION: Direction = Direction::Up;
const PIXEL_SCALE: i32 = 2;
const WINDOW_MARGIN: i32 = 2;
const BG0_RECT_SIZE: (i32, i32) = (100, 60);
const BG1_RECT_SIZE: (i32, i32) = (160, 160);
const MAX_SPRITES: usize = 512;

const SAMPLING_FREQ: i32 = 48000;
const SAMPLES_PER_FRAME: usize = SAMPLING_FREQ as usize / 60;
const SOUND_BUF_SIZE: usize = 4096;
const NUM_OF_AUDIO_CHANNELS: usize = 8;
const FREQ_ADJ_RATIO: f64 = 44.1;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let mut game_window = GameWindow::new(
        video_subsystem,
        FULL_SCREEN,
        VM_RECT_PIXEL_SIZE,
        ROTATION,
        PIXEL_SCALE,
        WINDOW_MARGIN,
    );

    let mut keyboard_map = InputRoleMap::<Key>::new();
    {
        let set_list = [
            (Key::Space, InputRole::Start),
            (Key::D4,    InputRole::Pause),
            (Key::Return,InputRole::Ok),
            (Key::X,     InputRole::Cancel),
            (Key::W,     InputRole::Up),
            (Key::D,     InputRole::Right),
            (Key::S,     InputRole::Down),
            (Key::A,     InputRole::Left),
            (Key::Up,    InputRole::Up),
            (Key::Right, InputRole::Right),
            (Key::Down,  InputRole::Down),
            (Key::Left,  InputRole::Left),
        ];
        keyboard_map.assign(&set_list);
    }
    let mut button_map = InputRoleMap::<ControllerButton>::new();
    {
        let set_list = [
            (ControllerButton { id: 0, button: 0 }, InputRole::Start),
            (ControllerButton { id: 0, button: 1 }, InputRole::Pause),
            (ControllerButton { id: 0, button: 2 }, InputRole::Ok),
            (ControllerButton { id: 0, button: 3 }, InputRole::Cancel),
            (ControllerButton { id: 0, button: 3 }, InputRole::Cancel),
        ];
        button_map.assign(&set_list);
    }
    let mut hat_map = InputRoleMap::<ControllerHat>::new();
    {
        let set_list = [
            (ControllerHat { id: 0, which: 0, state: HatState::Centered  }, InputRole::None),
            (ControllerHat { id: 0, which: 0, state: HatState::Up        }, InputRole::Up),
            (ControllerHat { id: 0, which: 0, state: HatState::Down      }, InputRole::Down),
            (ControllerHat { id: 0, which: 0, state: HatState::Right     }, InputRole::Right),
            (ControllerHat { id: 0, which: 0, state: HatState::Left      }, InputRole::Left),
            (ControllerHat { id: 0, which: 0, state: HatState::RightUp   }, InputRole::Right),
            (ControllerHat { id: 0, which: 0, state: HatState::RightUp   }, InputRole::Up),
            (ControllerHat { id: 0, which: 0, state: HatState::RightDown }, InputRole::Right),
            (ControllerHat { id: 0, which: 0, state: HatState::RightDown }, InputRole::Down),
            (ControllerHat { id: 0, which: 0, state: HatState::LeftUp    }, InputRole::Left),
            (ControllerHat { id: 0, which: 0, state: HatState::LeftUp    }, InputRole::Up),
            (ControllerHat { id: 0, which: 0, state: HatState::LeftDown  }, InputRole::Left),
            (ControllerHat { id: 0, which: 0, state: HatState::LeftDown  }, InputRole::Down),
        ];
        hat_map.assign(&set_list);
    }
    let mut mousebutton_map = InputRoleMap::<MouseButton>::new();
    {
        let set_list = [
            (MouseButton::Left,  InputRole::LeftButton),
            (MouseButton::Right, InputRole::RightButton),
        ];
        mousebutton_map.assign(&set_list);
    }
    let mut input_role_state = InputRoleState::default();

    let mut bg_texture_bank = BgTextureBank::new(
        &bgchar_data::BG_PATTERN_TBL,
        &bgpal_data::COLOR_TBL,
        game_window.pixel_scale() as i32,
    );
    let rc_bg_texture_bank = Rc::new(RefCell::new(&mut bg_texture_bank));
    let mut bg = {
        let bg0 = BgPlane::new(
            BG0_RECT_SIZE,
            VM_RECT_PIXEL_SIZE,
            rc_bg_texture_bank.clone(),
        );
        let bg1 = BgPlane::new(
            BG1_RECT_SIZE,
            VM_RECT_PIXEL_SIZE,
            rc_bg_texture_bank.clone(),
        );
        (bg0, bg1)
    };

    let mut sp_texture_bank = SpTextureBank::new(
        &spchar_data::SP_PATTERN_TBL,
        &sppal_data::COLOR_TBL,
        game_window.pixel_scale() as i32,
    );
    let rc_sp_texture_bank = Rc::new(RefCell::new(&mut sp_texture_bank));
    let mut spr = SpResources::new(MAX_SPRITES, rc_sp_texture_bank.clone());

    let mut audio_context = AudioContext::with_subsystem(audio_subsystem);
    audio_context.set_freq(Some(SAMPLING_FREQ));
    audio_context.set_channels(Some(1));
    audio_context.set_samples(Some(256));
    let mut audio_device_a = audio_context.open_device(SOUND_BUF_SIZE).unwrap();

    let mut sound_manager = SoundManager::default();

    if game_window.full_screen() {
        sdl_context.mouse().show_cursor(false);
    }
    sdl_context.mouse().show_cursor(false);

    bg.1.fill_palette(1);
    {
        let s = "Test for WSG Play".to_string();
        let x = (VM_RECT_SIZE.0 - s.len() as i32) / 2;
        bg.1.set_cur_pos(x,  0).put_string(&s,    Some(&CharAttributes::new(3, BgSymmetry::Normal)));
        bg.1.set_cur_pos(3,  4).put_achar(&AChar::new(0x80 as u32, 4, BgSymmetry::Normal));
        for i in 0..8 {
            let y = 6 + i as i32 * 2;
            let achar = AChar::new('0' as u32 + i, 4, BgSymmetry::Normal);
            bg.1.set_cur_pos( 3, y).put_achar(&achar).put_achar(&AChar::new(':', 1, BgSymmetry::Normal));
            bg.1.set_cur_pos(16, y).put_palette_n(3, 18).put_palette(2);
        }
        bg.1.set_cur_pos(8, 3).put_string(&"LastFrame", None);
        bg.1.set_achar_at(12, 4, &AChar::new(0x7fu32, 4, BgSymmetry::Normal));
        bg.1.set_cur_pos(18, 3).put_string(&"PlaySpeed", None);
        bg.1.set_achar_at(20, 4, &AChar::new('-', 4, BgSymmetry::Normal));
        bg.1.set_achar_at(25, 4, &AChar::new('+', 4, BgSymmetry::Normal));
        bg.1.set_cur_pos(30, 3).put_string(&"Sound:", None);
        bg.1.set_cur_pos(29, 4).put_string(&"Volume:", None);
    }
    spr.sp[0].code(0).palette(1).symmetry(SpSymmetry::Normal);

    let mut t_count = 0;
    let mut pointer_pos = (0.0, 0.0);
    let mut master_volume = 4;
    let mut music_select = 0;
    let mut music_playing = None;
    let mut play_step = 1;
    let mut suppress_last = false;
    let mut sound_generator = SoundGenerator::new(SAMPLING_FREQ, Some(SOUND_BUF_SIZE), Some(FREQ_ADJ_RATIO));
    audio_device_a.set_volume(master_volume);
    audio_device_a.resume();

    input_role_state.clear_all();
    let mut offset = 0;
    'main_loop: loop {
        let mut pos = (t_count * SAMPLES_PER_FRAME as i32 + offset) as usize;
        let audio_device_current = audio_device_a.current();
        while audio_device_current > pos {
            pos += SAMPLES_PER_FRAME;
            offset += SAMPLES_PER_FRAME as i32;
        }
        while pos - audio_device_current > SAMPLES_PER_FRAME * 4 {
            pos -= SAMPLES_PER_FRAME;
            offset -= SAMPLES_PER_FRAME as i32;
        }

        if t_count % play_step == 0 {
            sound_manager.run();
        }
            let sound_data = sound_manager.get_ch_registers();
        if t_count % play_step == play_step - 1 {
            sound_manager.clear_ch_registers();
        }
        sound_generator.generate(&sound_data);
        audio_device_a.set_data(pos, sound_generator.mixed_buffer());

        let (m_pos_spx, m_pos_spy) = (pointer_pos.0 as i32 / PIXEL_SCALE, pointer_pos.1 as i32 / PIXEL_SCALE);
        let (m_pos_bgx, m_pos_bgy) = (m_pos_spx / PATTERN_SIZE as i32, m_pos_spy / PATTERN_SIZE as i32);
        bg.0.set_cur_pos(0, 0).put_string(&format!("{:4}", t_count), Some(&CharAttributes::new(1, BgSymmetry::Normal)));
        bg.0.set_cur_pos(0, 1).put_string(&format!("{:3} {:3}", m_pos_spx, m_pos_spy), Some(&CharAttributes::new(1, BgSymmetry::Normal)));
        bg.0.set_cur_pos(0, 2).put_string(&format!("{:3} {:3}", m_pos_bgx, m_pos_bgy), Some(&CharAttributes::new(1, BgSymmetry::Normal)));
        spr.sp[0].pos(Pos::new(m_pos_spx, m_pos_spy)).visible(true);

        let mut selected = None;
        if input_role_state.get(InputRole::LeftButton).1 & 0b1111 == 0b0011 {
            bg.1.set_cur_pos(m_pos_bgx, m_pos_bgy);
            let achar = bg.1.read_achar();
            if achar.palette == 4 || achar.palette == 5 {
                if ('0' as u32..='7' as u32).contains(&achar.code) {
                    let ch = achar.code as usize - '0' as usize;
                    sound_generator.mute[ch] = !sound_generator.mute[ch];
                }
                if achar.code == 0x80 as u32 {
                    for ch in 0..8 {
                        sound_generator.mute[ch] = !sound_generator.mute[ch];
                    }
                }
                if achar.code == '_' as u32 || achar.code == '|' as u32 {
                    let music_no = m_pos_bgx as usize - 4;
                    if (0..0x20).contains(&music_no) {
                        selected = Some(music_no);
                    }
                }
                if achar.code == '+' as u32 {
                    if play_step > 1 {
                        play_step -= 1;
                    }
                }
                if achar.code == '-' as u32 {
                    if play_step < 4 {
                        play_step += 1;
                    }
                }
                if achar.code == 0x7f {
                    suppress_last = !suppress_last;
                    sound_manager.suppress_last_silence = suppress_last;
                }
            }
        }

        if input_role_state.get(InputRole::Left).1 & 0b1111 == 0b0011 {
            music_select = if music_select == 0 { 31 } else { music_select -1 };
        }
        if input_role_state.get(InputRole::Right).1 & 0b1111 == 0b0011 {
            music_select = if music_select == 31 { 0 } else { music_select + 1 };
        }
        if input_role_state.get(InputRole::Up).1 & 0b1111 == 0b0011 {
            if master_volume < 7 {
                master_volume += 1;
                audio_device_a.set_volume(master_volume);
            }
        }
        if input_role_state.get(InputRole::Down).1 & 0b1111 == 0b0011 {
            if master_volume > 0 {
                master_volume -= 1;
                audio_device_a.set_volume(master_volume);
            }
        }
        if input_role_state.get(InputRole::Start).1 & 0b1111 == 0b0011 {
            if let Some(music_no) = music_playing {
                if music_no != music_select {
                    sound_manager.play_request[music_no] = 0;
                }
            }
            selected = Some(music_select);
            music_playing = Some(music_select);
        }
        if input_role_state.get(InputRole::Ok).1 & 0b1111 == 0b0011 {
            selected = Some(music_select);
        }
        if input_role_state.get(InputRole::Cancel).1 & 0b1111 == 0b0011 {
            for i in 0..0x20 {
                sound_manager.play_request[i] = 0;
            }
        }

        if let Some(music_no) = selected {
            if music_no == 0x1f {
                sound_manager.play_request[music_no] += 1;
            } else {
                sound_manager.play_request[music_no] = match sound_manager.play_request[music_no] {
                    0 => 1,
                    n => -n,
                };
            }
        }
        for music_no in 0..0x20 {
            if bg.1.get_code_at(music_no as i32 + 4,22) == 'R' as u32 && !sound_manager.play_progress(music_no) && sound_manager.play_request[music_no] == 0 {
                sound_manager.play_request[music_no] = -1;
            }
        }
        if let Some(music_no) = music_playing {
            if !sound_manager.play_progress(music_no) && sound_manager.play_request[music_no] == 0 {
                music_playing = None;
            }
        }

        for ch in 0..NUM_OF_AUDIO_CHANNELS {
            let (w, f, g) = sound_data[ch];
            let freq = f as f64 / FREQ_ADJ_RATIO;
            let gain = if sound_generator.mute[ch] || freq <= 30.0 { 0 } else { g as i32 };
            let y = (6 + ch * 2) as i32;
            bg.1.set_palette_at(3, y, if sound_generator.mute[ch] { 5 } else { 4 });
            bg.1.set_cur_pos( 5, y).put_string(&format!("{:7.2}Hz {:1} {:2} ", freq, w, gain), None);
            bg.1.set_cur_pos(20, y).put_code_n(0x7f as u32, gain).put_code_n(' ', 15 - gain);
        }
        bg.1.set_cur_pos(4, 22);
        for music_no in 0..0x20 {
            let c = match sound_manager.play_request[music_no] {
                0 => ' ',
                n => if n > 0 { (n as u8 + '0' as u8) as char } else { 'R' }
            };
            bg.1.put_achar(&AChar::new(c, 1, BgSymmetry::Normal));
        }
        bg.1.set_cur_pos(4, 23);
        for music_no in 0..0x20 {
            let c = if sound_manager.play_progress(music_no) { '|' } else { '_' };
            bg.1.put_achar(&AChar::new(c, 4, BgSymmetry::Normal));
        }
        bg.1.set_code_n_at(4, 24, ' ', 0x20);
        bg.1.set_cur_pos(music_select as i32 + 4, 24);
        bg.1.put_achar(&AChar::new('^', 3, BgSymmetry::Normal));
        #[cfg(feature="develop")]
        {
            if input_role_state.get(InputRole::Pause).1 & 0b1111 == 0b0011 {
                println!("{:?}", sound_manager.get_ch_registers());
                println!("{:?}", sound_manager.play_request);
            }
        }
        bg.1.set_cur_pos( 8, 2).put_string(&format!("{:9} {:9} {:4} {:5}", audio_device_current, pos, pos - audio_device_current, offset), Some(&CharAttributes::new(2, BgSymmetry::Normal)));
        bg.1.set_cur_pos(36, 3).put_string(&format!("{:02X}", music_select), None);
        bg.1.set_cur_pos(37, 4).put_string(&format!("{:1}", master_volume), None);
        let speed = match play_step {
            1 => 100,
            2 => 50,
            3 => 33,
            4 => 25,
            _ => 0,
        };
        bg.1.set_cur_pos(21, 4).put_string(&format!("{:3}%", speed), None);
        let p = if suppress_last { 5 } else { 4 };
        bg.1.set_palette_at(12, 4, p);

        if wait_and_update::doing(
            &mut game_window,
            &mut spr,
            &mut bg,
            &mut keyboard_map,
            &mut button_map,
            &mut hat_map,
            &mut mousebutton_map,
            &mut pointer_pos,
        ) {
            break 'main_loop;
        }
        input_role_state.clear_state();
        input_role_state.update_state(&keyboard_map);
        input_role_state.update_state(&button_map);
        input_role_state.update_state(&hat_map);
        input_role_state.update_state(&mousebutton_map);
        input_role_state.update_history();
        t_count += 1;
    }
    audio_device_a.pause();
    sdl_context.mouse().show_cursor(true);
}
