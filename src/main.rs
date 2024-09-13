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

mod music_data;
use music_data::MUSIC;

// use once_cell::sync::OnceCell;
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
// const AUDIO_VOLUME: u16 = 5;


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

    const SAMPLING_FREQ: i32 = 48000;
    const SOUND_BUF_SIZE: usize = 4096;

    let mut audio_context = AudioContext::with_subsystem(audio_subsystem);
    audio_context.set_freq(Some(SAMPLING_FREQ));
    audio_context.set_channels(Some(1));
    audio_context.set_samples(Some(256));
    let mut audio_device_a = audio_context.open_device(SOUND_BUF_SIZE).unwrap();

    const WAVE_DATA_LENGTH: usize = 32;

    let mut wave_data2: SoundData16 = SoundData16::new();
    {
        let wave: [u8; WAVE_DATA_LENGTH] = [
            0x07, 0x0a, 0x0c, 0x0c, 0x0e, 0x0e, 0x0c, 0x0c,
            0x0e, 0x0e, 0x0c, 0x09, 0x07, 0x07, 0x05, 0x05,
            0x07, 0x09, 0x09, 0x07, 0x07, 0x05, 0x02, 0x00,
            0x00, 0x02, 0x02, 0x00, 0x00, 0x02, 0x02, 0x03,
        ];
        for a in wave {
            wave_data2.push(((a + 1) as u16) << 12);
        }
    }

    let mut wave_data4: SoundData16 = SoundData16::new();
    {
        let wave: [u8; WAVE_DATA_LENGTH] = [
            0x07, 0x0a, 0x0c, 0x0d, 0x0e, 0x0d, 0x0c, 0x0a,
            0x07, 0x04, 0x02, 0x01, 0x00, 0x01, 0x02, 0x04,
            0x07, 0x0b, 0x0d, 0x0e, 0x0d, 0x0b, 0x07, 0x03,
            0x01, 0x00, 0x01, 0x03, 0x07, 0x0e, 0x07, 0x00,
        ];
        for a in wave {
            wave_data4.push(((a + 1) as u16) << 12);
        }
    }

    let mut music_ch0_flat:Vec<(u32, u16)> = Vec::new();
    let mut music_ch1_flat:Vec<(u32, u16)> = Vec::new();
    let mut music_ch2_flat:Vec<(u32, u16)> = Vec::new();
    let mut music_ch3_flat:Vec<(u32, u16)> = Vec::new();
    let mut music_ch4_flat:Vec<(u32, u16)> = Vec::new();
    let mut music_ch5_flat:Vec<(u32, u16)> = Vec::new();
    let mut music_ch6_flat:Vec<(u32, u16)> = Vec::new();
    let mut music_ch7_flat:Vec<(u32, u16)> = Vec::new();
    {
        for frame in MUSIC {
            music_ch0_flat.push(frame[0]);
            music_ch1_flat.push(frame[1]);
            music_ch2_flat.push(frame[2]);
            music_ch3_flat.push(frame[3]);
            music_ch4_flat.push(frame[4]);
            music_ch5_flat.push(frame[5]);
            music_ch6_flat.push(frame[6]);
            music_ch7_flat.push(frame[7]);
        }
    }

    if game_window.full_screen() {
        sdl_context.mouse().show_cursor(false);
    }
    sdl_context.mouse().show_cursor(false);

    bg.1.fill_palette(2);
    {
        let s = "Test for WSG Play".to_string();
        let x = (VM_RECT_SIZE.0 - s.len() as i32) / 2;
        bg.1.set_cur_pos(x, 0)
            .put_string(&s, Some(&CharAttributes::new(3, BgSymmetry::Normal)));
        bg.1.set_cur_pos(3,  5).put_string(&"0:", Some(&CharAttributes::new(1, BgSymmetry::Normal)));
        bg.1.set_cur_pos(3,  7).put_string(&"1:", Some(&CharAttributes::new(1, BgSymmetry::Normal)));
        bg.1.set_cur_pos(3,  9).put_string(&"2:", Some(&CharAttributes::new(1, BgSymmetry::Normal)));
        bg.1.set_cur_pos(3, 11).put_string(&"3:", Some(&CharAttributes::new(1, BgSymmetry::Normal)));
        bg.1.set_cur_pos(3, 13).put_string(&"4:", Some(&CharAttributes::new(1, BgSymmetry::Normal)));
        bg.1.set_cur_pos(3, 15).put_string(&"5:", Some(&CharAttributes::new(1, BgSymmetry::Normal)));
        bg.1.set_cur_pos(3, 17).put_string(&"6:", Some(&CharAttributes::new(1, BgSymmetry::Normal)));
        bg.1.set_cur_pos(3, 19).put_string(&"7:", Some(&CharAttributes::new(1, BgSymmetry::Normal)));
    }
    spr.sp[0].code(0).palette(1).symmetry(SpSymmetry::Normal);

    let mut t_count = 0;
    let mut pointer_pos = (0.0, 0.0);
    let mut master_volume = 4;
    let mut mute = (false, false, false, false, false, false, false, false);
    //let mut volume = (0i32, 0, 0, 0, 0, 0, 0, 0);
    let mut playing = (false, false, false, false, false, false, false, false);
    let mut freq = (523, 587, 659, 698, 784, 880, 988, 1046);
    let mut drift = (0, 0, 0, 0, 0, 0, 0, 0);
    let mut note_pos = (0usize, 0, 0, 0, 0, 0, 0, 0);
    let mut ch0_buffer_i32 = vec![SETUP_U16; SOUND_BUF_SIZE];
    let mut ch1_buffer_i32 = vec![SETUP_U16; SOUND_BUF_SIZE];
    let mut ch2_buffer_i32 = vec![SETUP_U16; SOUND_BUF_SIZE];
    let mut ch3_buffer_i32 = vec![SETUP_U16; SOUND_BUF_SIZE];
    let mut ch4_buffer_i32 = vec![SETUP_U16; SOUND_BUF_SIZE];
    let mut ch5_buffer_i32 = vec![SETUP_U16; SOUND_BUF_SIZE];
    let mut ch6_buffer_i32 = vec![SETUP_U16; SOUND_BUF_SIZE];
    let mut ch7_buffer_i32 = vec![SETUP_U16; SOUND_BUF_SIZE];
    let mut mixed_buffer: SoundData16 = vec![SETUP_U16 as u16; 800];
    audio_device_a.resume();
    /*
    for i in 0..16 {
        audio_device_a.set_data(i * sin_wave.len(), &sin_wave);
    }
    */

    //const BASE_FREQ: u32 = ((SAMPLING_FREQ / WAVE_DATA_LENGTH as i32) * 44) as u32;

    input_role_state.clear_all();
    let mut buffer_pos = 0;
    let mut offset = 0;
    'main_loop: loop {
        audio_device_a.set_volume(master_volume);
        let mut pos = t_count * 800 + offset;
        while audio_device_a.current() > pos {
            pos += 800;
            offset += 800;
        }
        bg.1.set_cur_pos(30, 1).put_string(&format!("{:6}", offset), None);
        if playing.0 {
            {
                let sin_wave_length = SAMPLING_FREQ as f64 / freq.0 as f64;
                let remain_length = 800 - drift.0 as i32;
                let c = (remain_length * freq.0) / SAMPLING_FREQ + if (remain_length * freq.0) % SAMPLING_FREQ > 0 { 1 } else { 0 };
                let group_length = (sin_wave_length * c as f64).round_ties_even() as usize;
                for i in 0..group_length {
                    let x = std::f64::consts::PI * 2.0 * i as f64 / sin_wave_length;
                    let y = (x.sin() * (SETUP_U16 - 1) as f64) as i32;
                    ch0_buffer_i32[(buffer_pos + drift.0 + i) % SOUND_BUF_SIZE] = y;
                }
                drift.0 = drift.0 + group_length - 800;
            }
        } else {
            for i in 0..800 - drift.0 {
                ch0_buffer_i32[(buffer_pos + drift.0 + i) % SOUND_BUF_SIZE] = 0;
            }
            drift.0 = 0;
        }
        bg.1.set_cur_pos(8, 5).put_string(&format!("{:4}Hz", freq.0), None);
        bg.1.set_cur_pos(26, 5).put_string(&format!("{:4}", drift.0), None);

        if playing.1 {
            {
                let sin_wave_length = SAMPLING_FREQ as f64 / freq.1 as f64;
                let remain_length = 800 - drift.1 as i32;
                let c = (remain_length * freq.1) / SAMPLING_FREQ + if (remain_length * freq.1) % SAMPLING_FREQ > 0 { 1 } else { 0 };
                let group_length = (sin_wave_length * c as f64).round_ties_even() as usize;
                for i in 0..group_length {
                    let x = std::f64::consts::PI * 2.0 * i as f64 / sin_wave_length;
                    let y = (x.sin() * (SETUP_U16 - 1) as f64) as i32;
                    ch1_buffer_i32[(buffer_pos + drift.1 + i) % SOUND_BUF_SIZE] = y;
                }
                drift.1 = drift.1 + group_length - 800;
            }
        } else {
            for i in 0..800 - drift.1 {
                ch1_buffer_i32[(buffer_pos + drift.1 + i) % SOUND_BUF_SIZE] = 0;
            }
            drift.1 = 0;
        }
        bg.1.set_cur_pos(8, 7).put_string(&format!("{:4}Hz", freq.1), None);
        bg.1.set_cur_pos(26, 7).put_string(&format!("{:4}", drift.1), None);

        if playing.2 {
            {
                let sin_wave_length = SAMPLING_FREQ as f64 / freq.2 as f64;
                let remain_length = 800 - drift.2 as i32;
                let c = (remain_length * freq.2) / SAMPLING_FREQ + if (remain_length * freq.2) % SAMPLING_FREQ > 0 { 1 } else { 0 };
                let group_length = (sin_wave_length * c as f64).round_ties_even() as usize;
                for i in 0..group_length {
                    let x = std::f64::consts::PI * 2.0 * i as f64 / sin_wave_length;
                    let y = (x.sin() * (SETUP_U16 - 1) as f64) as i32;
                    ch2_buffer_i32[(buffer_pos + drift.2 + i) % SOUND_BUF_SIZE] = y;
                }
                drift.2 = drift.2 + group_length - 800;
            }
        } else {
            for i in 0..800 - drift.2 {
                ch2_buffer_i32[(buffer_pos + drift.2 + i) % SOUND_BUF_SIZE] = 0;
            }
            drift.2 = 0;
        }
        bg.1.set_cur_pos(8, 9).put_string(&format!("{:4}Hz", freq.2), None);
        bg.1.set_cur_pos(26, 9).put_string(&format!("{:4}", drift.2), None);

        if playing.3 {
            {
                let sin_wave_length = SAMPLING_FREQ as f64 / freq.3 as f64;
                let remain_length = 800 - drift.3 as i32;
                let c = (remain_length * freq.3) / SAMPLING_FREQ + if (remain_length * freq.3) % SAMPLING_FREQ > 0 { 1 } else { 0 };
                let group_length = (sin_wave_length * c as f64).round_ties_even() as usize;
                for i in 0..group_length {
                    let x = std::f64::consts::PI * 2.0 * i as f64 / sin_wave_length;
                    let y = (x.sin() * (SETUP_U16 - 1) as f64) as i32;
                    ch3_buffer_i32[(buffer_pos + drift.3 + i) % SOUND_BUF_SIZE] = y;
                }
                drift.3 = drift.3 + group_length - 800;
            }
        } else {
            for i in 0..800 - drift.3 {
                ch3_buffer_i32[(buffer_pos + drift.3 + i) % SOUND_BUF_SIZE] = 0;
            }
            drift.3 = 0;
        }
        bg.1.set_cur_pos(8, 11).put_string(&format!("{:4}Hz", freq.3), None);
        bg.1.set_cur_pos(26, 11).put_string(&format!("{:4}", drift.3), None);

        if playing.4 {
            {
                let sin_wave_length = SAMPLING_FREQ as f64 / freq.4 as f64;
                let remain_length = 800 - drift.4 as i32;
                let c = (remain_length * freq.4) / SAMPLING_FREQ + if (remain_length * freq.4) % SAMPLING_FREQ > 0 { 1 } else { 0 };
                let group_length = (sin_wave_length * c as f64).round_ties_even() as usize;
                for i in 0..group_length {
                    let x = std::f64::consts::PI * 2.0 * i as f64 / sin_wave_length;
                    let y = (x.sin() * (SETUP_U16 - 1) as f64) as i32;
                    ch4_buffer_i32[(buffer_pos + drift.4 + i) % SOUND_BUF_SIZE] = y;
                }
                drift.4 = drift.4 + group_length - 800;
            }
        } else {
            for i in 0..800 - drift.4 {
                ch4_buffer_i32[(buffer_pos + drift.4 + i) % SOUND_BUF_SIZE] = 0;
            }
            drift.4 = 0;
        }
        bg.1.set_cur_pos(8, 13).put_string(&format!("{:4}Hz", freq.4), None);
        bg.1.set_cur_pos(26, 13).put_string(&format!("{:4}", drift.4), None);

        if playing.5 {
            {
                let sin_wave_length = SAMPLING_FREQ as f64 / freq.5 as f64;
                let remain_length = 800 - drift.5 as i32;
                let c = (remain_length * freq.5) / SAMPLING_FREQ + if (remain_length * freq.5) % SAMPLING_FREQ > 0 { 1 } else { 0 };
                let group_length = (sin_wave_length * c as f64).round_ties_even() as usize;
                for i in 0..group_length {
                    let x = std::f64::consts::PI * 2.0 * i as f64 / sin_wave_length;
                    let y = (x.sin() * (SETUP_U16 - 1) as f64) as i32;
                    ch5_buffer_i32[(buffer_pos + drift.5 + i) % SOUND_BUF_SIZE] = y;
                }
                drift.5 = drift.5 + group_length - 800;
            }
        } else {
            for i in 0..800 - drift.5 {
                ch5_buffer_i32[(buffer_pos + drift.5 + i) % SOUND_BUF_SIZE] = 0;
            }
            drift.5 = 0;
        }
        bg.1.set_cur_pos(8, 15).put_string(&format!("{:4}Hz", freq.5), None);
        bg.1.set_cur_pos(26, 15).put_string(&format!("{:4}", drift.5), None);

        if playing.6 {
            {
                let sin_wave_length = SAMPLING_FREQ as f64 / freq.6 as f64;
                let remain_length = 800 - drift.6 as i32;
                let c = (remain_length * freq.6) / SAMPLING_FREQ + if (remain_length * freq.6) % SAMPLING_FREQ > 0 { 1 } else { 0 };
                let group_length = (sin_wave_length * c as f64).round_ties_even() as usize;
                for i in 0..group_length {
                    let x = std::f64::consts::PI * 2.0 * i as f64 / sin_wave_length;
                    let y = (x.sin() * (SETUP_U16 - 1) as f64) as i32;
                    ch6_buffer_i32[(buffer_pos + drift.6 + i) % SOUND_BUF_SIZE] = y;
                }
                drift.6 = drift.6 + group_length - 800;
            }
        } else {
            for i in 0..800 - drift.6 {
                ch6_buffer_i32[(buffer_pos + drift.6 + i) % SOUND_BUF_SIZE] = 0;
            }
            drift.6 = 0;
        }
        bg.1.set_cur_pos(8, 17).put_string(&format!("{:4}Hz", freq.6), None);
        bg.1.set_cur_pos(26, 17).put_string(&format!("{:4}", drift.6), None);

        if playing.7 {
            {
                let sin_wave_length = SAMPLING_FREQ as f64 / freq.7 as f64;
                let remain_length = 800 - drift.7 as i32;
                let c = (remain_length * freq.7) / SAMPLING_FREQ + if (remain_length * freq.7) % SAMPLING_FREQ > 0 { 1 } else { 0 };
                let group_length = (sin_wave_length * c as f64).round_ties_even() as usize;
                for i in 0..group_length {
                    let x = std::f64::consts::PI * 2.0 * i as f64 / sin_wave_length;
                    let y = (x.sin() * (SETUP_U16 - 1) as f64) as i32;
                    ch7_buffer_i32[(buffer_pos + drift.7 + i) % SOUND_BUF_SIZE] = y;
                }
                drift.7 = drift.7 + group_length - 800;
            }
        } else {
            for i in 0..800 - drift.7 {
                ch7_buffer_i32[(buffer_pos + drift.7 + i) % SOUND_BUF_SIZE] = 0;
            }
            drift.7 = 0;
        }
        bg.1.set_cur_pos(8, 19).put_string(&format!("{:4}Hz", freq.7), None);
        bg.1.set_cur_pos(26, 19).put_string(&format!("{:4}", drift.7), None);

        {
            let mut idx = buffer_pos;
            for i in 0..800 {
                let ch0 = if mute.0 { 0 } else { ch0_buffer_i32[idx] };
                let ch1 = if mute.1 { 0 } else { ch1_buffer_i32[idx] };
                let ch2 = if mute.2 { 0 } else { ch2_buffer_i32[idx] };
                let ch3 = if mute.3 { 0 } else { ch3_buffer_i32[idx] };
                let ch4 = if mute.4 { 0 } else { ch4_buffer_i32[idx] };
                let ch5 = if mute.5 { 0 } else { ch5_buffer_i32[idx] };
                let ch6 = if mute.6 { 0 } else { ch6_buffer_i32[idx] };
                let ch7 = if mute.7 { 0 } else { ch7_buffer_i32[idx] };
                mixed_buffer[i] = ((ch0 + ch1 + ch2 + ch3 + ch4 + ch5 + ch6 + ch7) / 8 + SETUP_U16) as u16;
                idx += 1;
                idx %= SOUND_BUF_SIZE;
            }
            buffer_pos = idx;
            audio_device_a.set_data(pos, &mixed_buffer);
        }

        let (m_pos_spx, m_pos_spy) = (pointer_pos.0 as i32 / PIXEL_SCALE, pointer_pos.1 as i32 / PIXEL_SCALE);
        let (m_pos_bgx, m_pos_bgy) = (m_pos_spx / PATTERN_SIZE as i32, m_pos_spy / PATTERN_SIZE as i32);
        bg.0.set_cur_pos(0, 0)
            .put_string(&format!("{:4}", t_count), Some(&CharAttributes::new(1, BgSymmetry::Normal)));
        bg.0.set_cur_pos(0, 1)
            .put_string(&format!("{:3} {:3}", m_pos_spx, m_pos_spy), Some(&CharAttributes::new(1, BgSymmetry::Normal)));
        bg.0.set_cur_pos(0, 2)
            .put_string(&format!("{:3} {:3}", m_pos_bgx, m_pos_bgy), Some(&CharAttributes::new(1, BgSymmetry::Normal)));
        spr.sp[0].pos(Pos::new(m_pos_spx, m_pos_spy)).visible(true);
        if input_role_state.get(InputRole::LeftButton).1 & 0b1111 == 0b0011 {
            bg.1.set_cur_pos(m_pos_bgx, m_pos_bgy);
            let (c, p) = (bg.1.read_code(), bg.1.read_palette());
            match (c as u8 as char, p) {
                ('0', 1) => mute.0 = true,
                ('0', 5) => mute.0 = false,
                ('1', 1) => mute.1 = true,
                ('1', 5) => mute.1 = false,
                ('2', 1) => mute.2 = true,
                ('2', 5) => mute.2 = false,
                ('3', 1) => mute.3 = true,
                ('3', 5) => mute.3 = false,
                ('4', 1) => mute.4 = true,
                ('4', 5) => mute.4 = false,
                ('5', 1) => mute.5 = true,
                ('5', 5) => mute.5 = false,
                ('6', 1) => mute.6 = true,
                ('6', 5) => mute.6 = false,
                ('7', 1) => mute.7 = true,
                ('7', 5) => mute.7 = false,
                (_, _) => {},
            }
            match p {
                1 => _ = bg.1.put_palette(5),
                5 => _ = bg.1.put_palette(1),
                _ => {},
            }
        }
        if input_role_state.get(InputRole::Up).1 & 0b1111 == 0b0011 {
            if master_volume < 7 {
                master_volume += 1;
            }
        }
        if input_role_state.get(InputRole::Down).1 & 0b1111 == 0b0011 {
            if master_volume > 0 {
                master_volume -= 1;
            }
        }
        if input_role_state.get(InputRole::Right).0 {
            if freq.0 < 1450 {
                freq.0 += 1;
            }
        }
        if input_role_state.get(InputRole::Left).0 {
            if freq.0 > 100 {
                freq.0 -= 1;
            }
        }
        if input_role_state.get(InputRole::Start).1 & 0b1111 == 0b0011 {
            playing.0 = !playing.0;
            note_pos.0 = 0;
            playing.1 = !playing.1;
            note_pos.1 = 0;
            playing.2 = !playing.2;
            note_pos.2 = 0;
            playing.3 = !playing.3;
            note_pos.3 = 0;
            playing.4 = !playing.4;
            note_pos.4 = 0;
            playing.5 = !playing.5;
            note_pos.5 = 0;
            playing.6 = !playing.6;
            note_pos.6 = 0;
            playing.7 = !playing.7;
            note_pos.7 = 0;
        }

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
