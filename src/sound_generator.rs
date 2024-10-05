mod wave_data;
use wave_data::*;

const NUM_OF_GENARTORS: usize = 8;
const GAIN_UP_TRANSITION: f64 = 0.1;
const GAIN_DOWN_TRANSITION: f64 = 0.1;

struct GeneratorUnit {
    buffer: Vec<i32>,
    drift: usize,
    current_gain: f64,
}

#[allow(dead_code)]
impl GeneratorUnit {
    fn new(buffer_size: usize) -> Self {
        Self {
            buffer: vec![0; buffer_size],
            drift: 0,
            current_gain: 0.0,
        }
    }

    fn clear(&mut self) {
        for d in self.buffer.iter_mut() {
            *d = 0;
        }
        self.drift = 0;
        self.current_gain = 0.0;
    }
}

pub struct SoundGenerator {
    sampling_freq: i32,
    buffer_size: usize,
    freq_adj_ratio: f64,
    samples_per_frame: usize,
    base_freq: f64,
    generators: [GeneratorUnit; NUM_OF_GENARTORS],
    pub mute: [bool; NUM_OF_GENARTORS],
    buffer_pos: usize,
    mixed_buffer: Vec<u16>,
}

#[allow(dead_code)]
impl SoundGenerator {
    pub fn new(sampling_freq: i32, op_buffer_size: Option<usize>, op_freq_adj_ratio: Option<f64>) -> Self {
        const DEFAULT_BUFFER_SIZE: usize = 4096;
        const DEFAULT_FREQ_ADJ_RATIO: f64 = 43.69; // 0x10000(=65536) -> 1500Hz
        let buffer_size = op_buffer_size.unwrap_or(DEFAULT_BUFFER_SIZE);
        let freq_adj_ratio = op_freq_adj_ratio.unwrap_or(DEFAULT_FREQ_ADJ_RATIO);
        let samples_per_frame = (sampling_freq / 60) as usize;
        let base_freq = sampling_freq as f64 / WAVE_DATA_LENGTH as f64;
        Self {
            sampling_freq,
            buffer_size,
            freq_adj_ratio,
            samples_per_frame,
            base_freq,
            generators: [
                GeneratorUnit::new(buffer_size),
                GeneratorUnit::new(buffer_size),
                GeneratorUnit::new(buffer_size),
                GeneratorUnit::new(buffer_size),
                GeneratorUnit::new(buffer_size),
                GeneratorUnit::new(buffer_size),
                GeneratorUnit::new(buffer_size),
                GeneratorUnit::new(buffer_size),
            ],
            mute: [false; NUM_OF_GENARTORS],
            buffer_pos: 0,
            mixed_buffer: vec![SETUP_U16 as u16; samples_per_frame],
        }
    }

    pub fn clear(&mut self) {
        for unit in self.generators.iter_mut() {
            unit.clear();
        }
        for m in self.mute.iter_mut() {
            *m = false;
        }
        self.buffer_pos = 0;
        for d in self.mixed_buffer.iter_mut() {
            *d = SETUP_U16 as u16;
        }
    }

    pub fn sampling_freq(&self) -> i32 {
        self.sampling_freq
    }

    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }

    pub fn samples_per_frame(&self) -> usize {
        self.samples_per_frame
    }

    pub fn base_freq(&self) -> f64 {
        self.base_freq
    }

    pub fn buffer_pos(&self) -> usize {
        self.buffer_pos
    }

    pub fn mixed_buffer(&self) -> &[u16] {
        &self.mixed_buffer
    }

    pub fn generate(&mut self, sound_data: &[(usize, i32, u16); NUM_OF_GENARTORS]) {
        for (ch, unit) in self.generators.iter_mut().enumerate() {
            let remain_length = self.samples_per_frame - unit.drift;
            let buffer_top_current = self.buffer_pos + unit.drift;
            let silence = {
                let (w, f, g) = sound_data[ch];
                let freq = f as f64 / self.freq_adj_ratio;
                if g == 0 && unit.current_gain <= 0.0 || freq <= 30.0 || w >= NUM_OF_WAVE_FORMS {
                    true // silence
                } else {
                    let specified_gain = if self.mute[ch] { 0.0 } else { g as f64};
                    let wave_length = self.sampling_freq as f64 / freq;
                    let c = (remain_length as f64 / wave_length).ceil() as usize;
                    let group_length = (wave_length * c as f64).round_ties_even() as usize;
                    {
                        let f_ratio = self.base_freq / freq;
                        for i in 0..group_length {
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
                            let src_pos = i as f64 / f_ratio;
                            let src_pos_floor = src_pos as usize;
                            let a = WAVE_FORMS[w][(src_pos_floor + 0) % WAVE_DATA_LENGTH] as f64;
                            // let sample_0 = WAVE_FORMS[w][(src_pos_floor + 0) % WAVE_DATA_LENGTH] as f64;
                            // let sample_1 = WAVE_FORMS[w][(src_pos_floor + 1) % WAVE_DATA_LENGTH] as f64;
                            // let a = sample_0 + (sample_1 - sample_0) * (src_pos - src_pos_floor as f64);
                            unit.buffer[(buffer_top_current + i) % self.buffer_size] = (a * unit.current_gain / 15.0) as i32;
                        }
                    }
                    unit.drift = (group_length - remain_length) % self.samples_per_frame;
                    false // sound
                }
            };
            if silence {
                for i in 0..remain_length {
                    unit.buffer[(buffer_top_current + i) % self.buffer_size] = 0;
                }
                unit.current_gain = 0.0;
                unit.drift = 0;
            }
        }
        for i in 0..self.samples_per_frame {
            let mut integrated = 0;
            for unit in self.generators.iter() {
                integrated += unit.buffer[self.buffer_pos];
            }
            self.mixed_buffer[i] = (integrated / NUM_OF_GENARTORS as i32 + SETUP_U16) as u16;
            self.buffer_pos = (self.buffer_pos + 1) % self.buffer_size;
        }
    }
}