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

pub struct SoundGenerator {
    sampling_freq: i32,
    samples_per_frame: usize,
    generators: [GeneratorUnit; NUM_OF_GENARTORS],
    pub mute: [bool; NUM_OF_GENARTORS],
    mixed_buffer: Vec<i16>,
    work: Vec<i32>,
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
            mixed_buffer: vec![0; samples_per_frame],
            work: vec![0; INTERNAL_RATE as usize / 60],
        }
    }

    pub fn clear(&mut self) {
        for unit in self.generators.iter_mut() {
            unit.clear();
        }
        for m in self.mute.iter_mut() {
            *m = false;
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
        for dist in self.work.iter_mut() {
            *dist = 0;
        }
        for (ch, unit) in self.generators.iter_mut().enumerate() {
            let (w, f, g) = sound_data[ch];
            if g == 0 && unit.current_gain == 0x0_00 {
                unit.phase_pos = 0;
                unit.current_wave_form = None;
                unit.current_gain = 0x0_00;
                unit.current_freq = 0;
            } else {
                let specified_gain = if self.mute[ch] || f == 0 { 0x0_00 } else { g * 0x1_00 };
                for dist in self.work.iter_mut() {
                    if unit.current_freq == 0 {
                        unit.current_freq = f;
                    };
                    let pos = (unit.phase_pos / INTERNAL_SAMPLE_LENGTH) as usize;
                    let wave_form_no = if let Some(current_w) = unit.current_wave_form { current_w } else { w };
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
                    let a = WAVE_FORMS[wave_form_no][pos] as i32;
                    *dist += a * unit.current_gain / 0xf_00;
                }
            }
        }
        let mut cycle = 0;
        let mut i = 0;
        for dist in self.mixed_buffer.iter_mut() {
            let mut s = 0;
            let mut n = 0;
            while cycle < INTERNAL_RATE {
                s += self.work[i];
                i += 1;
                n += 1;
                cycle += self.sampling_freq;
            }
            cycle -= INTERNAL_RATE;
            *dist = (s / (n * NUM_OF_GENARTORS as i32)) as i16;
        }
    }
}