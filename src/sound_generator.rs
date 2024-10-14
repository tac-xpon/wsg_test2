mod wave_data;
use wave_data::*;

const INTERNAL_RATE: i32 = 192_000;
const REFERENCE_TONE: i32 = 1500; // 1500Hz
const REFERENCE_REG_VALUE: i32 = 0x1_0000; // 0x1_0000 -> 1500Hz
const INTERNAL_SAMPLE_LENGTH: i32 = INTERNAL_RATE / (REFERENCE_TONE * WAVE_DATA_LENGTH as i32) * REFERENCE_REG_VALUE;
const INTERNAL_WAVE_LENGTH: i32 = INTERNAL_SAMPLE_LENGTH * WAVE_DATA_LENGTH as i32;
const NUM_OF_GENARTORS: usize = 8;
const GAIN_UP_TRANSITION: i32 = 0x0_10;
const GAIN_DOWN_TRANSITION: i32 = 0x0_10;

struct GeneratorUnit {
    phase_pos: i32,
    current_wave_form: Option<usize>,
    current_gain: i32,
    current_freq: i32,
}

#[allow(dead_code)]
impl GeneratorUnit {
    fn new() -> Self {
        Self {
            phase_pos: 0,
            current_wave_form: None,
            current_gain: 0x0_00,
            current_freq: 0,
        }
    }

    fn clear(&mut self) {
        self.phase_pos = 0;
        self.current_wave_form = None;
        self.current_gain = 0x0_00;
        self.current_freq = 0;
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum PanPot {
    Left = -1,
    Center = 0,
    Right = 1,
}

pub struct SoundGenerator {
    sampling_freq: i32,
    samples_per_frame: usize,
    generators: [GeneratorUnit; NUM_OF_GENARTORS],
    pub mute: [bool; NUM_OF_GENARTORS],
    pub panpot: [PanPot; NUM_OF_GENARTORS],
    pub master_gain: i32,
    mixed_buffer: Vec<i16>,
    work: Vec<(i32, i32)>,
}

#[allow(dead_code)]
impl SoundGenerator {
    pub fn new(sampling_freq: i32) -> Self {
        let samples_per_frame = (sampling_freq / 60) as usize;
        Self {
            sampling_freq,
            samples_per_frame,
            generators: [
                GeneratorUnit::new(),
                GeneratorUnit::new(),
                GeneratorUnit::new(),
                GeneratorUnit::new(),
                GeneratorUnit::new(),
                GeneratorUnit::new(),
                GeneratorUnit::new(),
                GeneratorUnit::new(),
            ],
            mute: [false; NUM_OF_GENARTORS],
            panpot: [PanPot::Center; NUM_OF_GENARTORS],
            master_gain: 7,
            mixed_buffer: vec![0; samples_per_frame * 2], // Stereo
            work: vec![(0, 0); INTERNAL_RATE as usize / 60],
        }
    }

    pub fn clear(&mut self) {
        for unit in self.generators.iter_mut() {
            unit.clear();
        }
        for m in self.mute.iter_mut() {
            *m = false;
        }
        for p in self.panpot.iter_mut() {
            *p = PanPot::Center;
        }
        for d in self.mixed_buffer.iter_mut() {
            *d = 0;
        }
    }

    pub fn sampling_freq(&self) -> i32 {
        self.sampling_freq
    }

    pub fn samples_per_frame(&self) -> usize {
        self.samples_per_frame
    }

    pub fn mixed_buffer(&self) -> &[i16] {
        &self.mixed_buffer
    }

    pub fn generate(&mut self, sound_data: &[(usize, i32, i32); NUM_OF_GENARTORS]) {
        for work in self.work.iter_mut() {
            *work = (0, 0);
        }
        for (ch, unit) in self.generators.iter_mut().enumerate() {
            let (w, f, g) = sound_data[ch];
            let panpot = self.panpot[ch];
            if g == 0 && unit.current_gain == 0x0_00 {
                unit.phase_pos = 0;
                unit.current_wave_form = None;
                unit.current_freq = 0;
            } else {
                let specified_gain = if self.mute[ch] || f == 0 { 0x0_00 } else { g * 0x1_00 };
                for work in self.work.iter_mut() {
                    if unit.current_freq == 0 {
                        unit.current_freq = f;
                    };
                    let wave_form_no = if let Some(current_w) = unit.current_wave_form { current_w } else { w };
                    let pos = (unit.phase_pos / INTERNAL_SAMPLE_LENGTH) as usize;
                    let s = WAVE_FORMS[wave_form_no][pos] as i32;
                    unit.phase_pos += unit.current_freq;
                    if unit.phase_pos >= INTERNAL_WAVE_LENGTH {
                        unit.phase_pos -= INTERNAL_WAVE_LENGTH;
                        unit.current_wave_form = None;
                        unit.current_freq = 0;
                    }
                    if specified_gain != unit.current_gain {
                        if specified_gain > unit.current_gain {
                            unit.current_gain += GAIN_UP_TRANSITION;
                            if unit.current_gain > specified_gain {
                                unit.current_gain = specified_gain;
                            }
                        } else {
                            unit.current_gain -= GAIN_DOWN_TRANSITION;
                            if unit.current_gain < specified_gain {
                                unit.current_gain = specified_gain;
                            }
                        }
                    }
                    let a = s * unit.current_gain / 0xf_00;
                    let (l, r) = match panpot {
                        PanPot::Left => (a, 0),
                        PanPot::Right => (0, a),
                        PanPot::Center => {
                            let c = a * 3 >> 2;
                            (c, c)
                        }
                    };
                    (*work).0 += l;
                    (*work).1 += r;
                }
            }
        }
        if self.master_gain <= 0 {
            for dist in self.mixed_buffer.iter_mut() {
                *dist = 0;
            }
        } else {
            let shift = match self.master_gain {
                1 => 6,
                2 => 5,
                3 => 4,
                4 => 3,
                5 => 2,
                6 => 1,
                _ => 0,
            };
            let mut cycle = 0;
            let mut i = 0;
            for pos in 0..self.samples_per_frame {
                let mut left_sum  = 0;
                let mut right_sum = 0;
                let mut n = 0;
                while cycle < INTERNAL_RATE {
                    left_sum  += self.work[i].0;
                    right_sum += self.work[i].1;
                    i += 1;
                    n += 1;
                    cycle += self.sampling_freq;
                }
                cycle -= INTERNAL_RATE;
                let left  = ((left_sum / (n * NUM_OF_GENARTORS as i32)) >> shift) as i16;
                let right = ((right_sum / (n * NUM_OF_GENARTORS as i32)) >> shift) as i16;
                self.mixed_buffer[pos * 2    ] = left;
                self.mixed_buffer[pos * 2 + 1] = right;
            }
        }
    }
}
