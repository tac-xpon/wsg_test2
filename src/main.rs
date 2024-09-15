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

mod wave_data;
use wave_data::*;

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
const SAMPLING_FREQ: i32 = 48000;
const SOUND_BUF_SIZE: usize = 4096;
const SAMPLES_PER_FRAME: usize = (SAMPLING_FREQ / 60) as usize;
const NUM_OF_AUDIO_CHANNELS: usize = 8;
const BASE_FREQ: f64 = SAMPLING_FREQ as f64 / WAVE_DATA_LENGTH as f64;
const FREQ_ADJ_RATIO: f64 = 44.0;

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

    if game_window.full_screen() {
        sdl_context.mouse().show_cursor(false);
    }
    sdl_context.mouse().show_cursor(false);

    bg.1.fill_palette(2);
    {
        let s = "Test for WSG Play".to_string();
        let x = (VM_RECT_SIZE.0 - s.len() as i32) / 2;
        bg.1.set_cur_pos(x,  0).put_string(&s,    Some(&CharAttributes::new(3, BgSymmetry::Normal)));
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
    let waveform = [&WAVE_0, &WAVE_1, &WAVE_2, &WAVE_3, &WAVE_4, &WAVE_5, &WAVE_6, &WAVE_7];
    let mut mute = [false; NUM_OF_AUDIO_CHANNELS];
    let mut drift = [0usize; NUM_OF_AUDIO_CHANNELS];
    let mut playing = false;
    let mut note_pos = 0;
    let mut ch0_buffer_i32 = vec![0; SOUND_BUF_SIZE];
    let mut ch1_buffer_i32 = vec![0; SOUND_BUF_SIZE];
    let mut ch2_buffer_i32 = vec![0; SOUND_BUF_SIZE];
    let mut ch3_buffer_i32 = vec![0; SOUND_BUF_SIZE];
    let mut ch4_buffer_i32 = vec![0; SOUND_BUF_SIZE];
    let mut ch5_buffer_i32 = vec![0; SOUND_BUF_SIZE];
    let mut ch6_buffer_i32 = vec![0; SOUND_BUF_SIZE];
    let mut ch7_buffer_i32 = vec![0; SOUND_BUF_SIZE];
    let buffers_i32 = [
        &mut ch0_buffer_i32,
        &mut ch1_buffer_i32,
        &mut ch2_buffer_i32,
        &mut ch3_buffer_i32,
        &mut ch4_buffer_i32,
        &mut ch5_buffer_i32,
        &mut ch6_buffer_i32,
        &mut ch7_buffer_i32,
    ];
    let mut mixed_buffer: SoundData16 = vec![SETUP_U16 as u16; SAMPLES_PER_FRAME];

    audio_device_a.set_volume(master_volume);
    audio_device_a.resume();

    input_role_state.clear_all();
    let mut buffer_pos = 0;
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
        bg.1.set_cur_pos(10, 2).put_string(&format!("{:7} {:7} {:5} {:6}", audio_device_current, pos, pos - audio_device_current, offset), None);
        bg.1.set_cur_pos(30, 3).put_string(&format!("Volume:{:1}", master_volume), None);

        let sound_data = if playing {
            note_pos += 1;
            if note_pos >= MUSIC.len() {
                playing = false;
            }
            MUSIC[note_pos - 1]
        } else {
            [(0usize, 0i32, 0u16);8]
        };
        for ch in 0..8 {
            let remain_length = SAMPLES_PER_FRAME - drift[ch];
            let buffer_top_current = buffer_pos + drift[ch];
            let out = {
                    let (w, f, v) = sound_data[ch];
                    let freq = f as f64 / FREQ_ADJ_RATIO;
                    if freq <= 30.0 || w >= NUM_OF_WAVE_FORMS {
                        None // silence
                    } else {
                        let wave_length = SAMPLING_FREQ as f64 / freq;
                        let c = (remain_length as f64 / wave_length).ceil() as usize;
                        let group_length = (wave_length * c as f64).round_ties_even() as usize;
                        if mute[ch] || v == 0 {
                            for i in 0..group_length {
                                buffers_i32[ch][(buffer_top_current + i) % SOUND_BUF_SIZE] = 0;
                            }
                        } else {
                            let f_ratio = BASE_FREQ / freq;
                            for i in 0..group_length {
                                let src_pos = i as f64 / f_ratio;
                                let src_pos_floor = src_pos as usize;
                                let sample_0 = waveform[w][(src_pos_floor + 0) % WAVE_DATA_LENGTH] as f64;
                                let sample_1 = waveform[w][(src_pos_floor + 1) % WAVE_DATA_LENGTH] as f64;
                                let a = sample_0 + (sample_1 - sample_0) * (src_pos - src_pos_floor as f64);
                                buffers_i32[ch][(buffer_top_current + i) % SOUND_BUF_SIZE] = (a * v as f64 / 15.0) as i32;
                            }
                        }
                        drift[ch] = (group_length - remain_length) % SAMPLES_PER_FRAME;
                        Some((freq, v)) // emit
                    }
            };
            if out.is_none() {
                for i in 0..remain_length {
                    buffers_i32[ch][(buffer_top_current + i) % SOUND_BUF_SIZE] = 0;
                }
                drift[ch] = 0;
            }
            let (freq, output_level) = out.unwrap_or((0.0, 0));
            let bar_length = if mute[ch] { 0 } else { output_level as i32 };
            let y = (5 + ch * 2) as i32;
            bg.1.set_cur_pos(5, y).put_string(&format!("{:7.2}Hz {:3} {:2}", freq, drift[ch], output_level), None);
            bg.1.put_code_n('*', bar_length).put_code_n(' ', 15 - bar_length);
        }

        {
            for i in 0..SAMPLES_PER_FRAME {
                let mut integrated = 0;
                for ch_no in 0..NUM_OF_AUDIO_CHANNELS {
                    integrated += buffers_i32[ch_no][buffer_pos];
                }
                mixed_buffer[i] = (integrated / 8 + SETUP_U16) as u16;
                buffer_pos = (buffer_pos + 1) % SOUND_BUF_SIZE;
            }
            audio_device_a.set_data(pos, &mixed_buffer);
        }

        let (m_pos_spx, m_pos_spy) = (pointer_pos.0 as i32 / PIXEL_SCALE, pointer_pos.1 as i32 / PIXEL_SCALE);
        let (m_pos_bgx, m_pos_bgy) = (m_pos_spx / PATTERN_SIZE as i32, m_pos_spy / PATTERN_SIZE as i32);
        bg.0.set_cur_pos(0, 0).put_string(&format!("{:4}", t_count), Some(&CharAttributes::new(1, BgSymmetry::Normal)));
        bg.0.set_cur_pos(0, 1).put_string(&format!("{:3} {:3}", m_pos_spx, m_pos_spy), Some(&CharAttributes::new(1, BgSymmetry::Normal)));
        bg.0.set_cur_pos(0, 2).put_string(&format!("{:3} {:3}", m_pos_bgx, m_pos_bgy), Some(&CharAttributes::new(1, BgSymmetry::Normal)));
        spr.sp[0].pos(Pos::new(m_pos_spx, m_pos_spy)).visible(true);
        if input_role_state.get(InputRole::LeftButton).1 & 0b1111 == 0b0011 {
            bg.1.set_cur_pos(m_pos_bgx, m_pos_bgy);
            let achar = bg.1.read_achar();
            if ('0'..='7').contains(&(achar.code as u8 as char)) && ((achar.palette == 1) || (achar.palette ==5)) {
                let ch = achar.code as usize - '0' as usize;
                mute[ch] = if achar.palette == 1 { true } else { false };
                bg.1.put_palette(if achar.palette == 1 { 5 } else { 1 });
            }
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
            playing = !playing;
            note_pos = 0;
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
