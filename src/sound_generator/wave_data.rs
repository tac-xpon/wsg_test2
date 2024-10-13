pub const WAVE_DATA_LENGTH: usize = 32;
pub const NUM_OF_WAVE_FORMS: usize = 8;

const SETUP_U16: i32 = 0x8000;

const WAVE_0: [i16; WAVE_DATA_LENGTH] = {
    const SAMPLES: [u8; WAVE_DATA_LENGTH] = [
        0x00, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e,
        0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    let mut wave: [i16; WAVE_DATA_LENGTH] = [0; WAVE_DATA_LENGTH];
    let mut idx = 0;
    while idx < WAVE_DATA_LENGTH {
        let s = SAMPLES[idx] as i32;
        wave[idx] = if s < 0x0f {
            (((s + 1) << 12) - SETUP_U16) as i16
        } else {
            i16::MAX
        };
        idx += 1;
    }
    wave
};

const WAVE_1: [i16; WAVE_DATA_LENGTH] = {
    const SAMPLES: [u8; WAVE_DATA_LENGTH] = [
        0x0d, 0x06, 0x09, 0x01, 0x06, 0x05, 0x0f, 0x0c,
        0x0a, 0x0c, 0x04, 0x04, 0x02, 0x0b, 0x08, 0x0e,
        0x05, 0x08, 0x03, 0x0a, 0x06, 0x09, 0x02, 0x09,
        0x07, 0x00, 0x09, 0x05, 0x0a, 0x05, 0x08, 0x06,
    ];
    let mut wave: [i16; WAVE_DATA_LENGTH] = [0; WAVE_DATA_LENGTH];
    let mut idx = 0;
    while idx < WAVE_DATA_LENGTH {
        let s = SAMPLES[idx] as i32;
        wave[idx] = if s < 0x0f {
            (((s + 1) << 12) - SETUP_U16) as i16
        } else {
            i16::MAX
        };
        idx += 1;
    }
    wave
};

const WAVE_2: [i16; WAVE_DATA_LENGTH] = {
    const SAMPLES: [u8; WAVE_DATA_LENGTH] = [
        0x07, 0x0a, 0x0c, 0x0c, 0x0e, 0x0e, 0x0c, 0x0c,
        0x0e, 0x0e, 0x0c, 0x09, 0x07, 0x07, 0x05, 0x05,
        0x07, 0x09, 0x09, 0x07, 0x07, 0x05, 0x02, 0x00,
        0x00, 0x02, 0x02, 0x00, 0x00, 0x02, 0x02, 0x03,
    ];
    let mut wave: [i16; WAVE_DATA_LENGTH] = [0; WAVE_DATA_LENGTH];
    let mut idx = 0;
    while idx < WAVE_DATA_LENGTH {
        let s = SAMPLES[idx] as i32;
        wave[idx] = if s < 0x0f {
            (((s + 1) << 12) - SETUP_U16) as i16
        } else {
            i16::MAX
        };
        idx += 1;
    }
    wave
};

const WAVE_3: [i16; WAVE_DATA_LENGTH] = {
    const SAMPLES: [u8; WAVE_DATA_LENGTH] = [
        0x0a, 0x0c, 0x0e, 0x0e, 0x0c, 0x0b, 0x0a, 0x09,
        0x0a, 0x0b, 0x0b, 0x03, 0x03, 0x04, 0x05, 0x05,
        0x04, 0x02, 0x00, 0x00, 0x02, 0x03, 0x04, 0x05,
        0x04, 0x03, 0x03, 0x03, 0x03, 0x02, 0x01, 0x01,
    ];
    let mut wave: [i16; WAVE_DATA_LENGTH] = [0; WAVE_DATA_LENGTH];
    let mut idx = 0;
    while idx < WAVE_DATA_LENGTH {
        let s = SAMPLES[idx] as i32;
        wave[idx] = if s < 0x0f {
            (((s + 1) << 12) - SETUP_U16) as i16
        } else {
            i16::MAX
        };
        idx += 1;
    }
    wave
};

const WAVE_4: [i16; WAVE_DATA_LENGTH] = {
    const SAMPLES: [u8; WAVE_DATA_LENGTH] = [
        0x07, 0x0a, 0x0c, 0x0d, 0x0e, 0x0d, 0x0c, 0x0a,
        0x07, 0x04, 0x02, 0x01, 0x00, 0x01, 0x02, 0x04,
        0x07, 0x0b, 0x0d, 0x0e, 0x0d, 0x0b, 0x07, 0x03,
        0x01, 0x00, 0x01, 0x03, 0x07, 0x0e, 0x07, 0x00,
    ];
    let mut wave: [i16; WAVE_DATA_LENGTH] = [0; WAVE_DATA_LENGTH];
    let mut idx = 0;
    while idx < WAVE_DATA_LENGTH {
        let s = SAMPLES[idx] as i32;
        wave[idx] = if s < 0x0f {
            (((s + 1) << 12) - SETUP_U16) as i16
        } else {
            i16::MAX
        };
        idx += 1;
    }
    wave
};

const WAVE_5: [i16; WAVE_DATA_LENGTH] = {
    const SAMPLES: [u8; WAVE_DATA_LENGTH] = [
        0x0a, 0x0c, 0x0c, 0x0a, 0x07, 0x07, 0x08, 0x0b,
        0x0d, 0x0e, 0x0d, 0x0a, 0x06, 0x05, 0x05, 0x07,
        0x09, 0x09, 0x08, 0x04, 0x01, 0x00, 0x01, 0x03,
        0x06, 0x07, 0x07, 0x04, 0x02, 0x02, 0x04, 0x07,
    ];
    let mut wave: [i16; WAVE_DATA_LENGTH] = [0; WAVE_DATA_LENGTH];
    let mut idx = 0;
    while idx < WAVE_DATA_LENGTH {
        let s = SAMPLES[idx] as i32;
        wave[idx] = if s < 0x0f {
            (((s + 1) << 12) - SETUP_U16) as i16
        } else {
            i16::MAX
        };
        idx += 1;
    }
    wave
};

const WAVE_6: [i16; WAVE_DATA_LENGTH] = {
    const SAMPLES: [u8; WAVE_DATA_LENGTH] = [
        0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0d, 0x0e, 0x0e,
        0x0e, 0x0d, 0x0d, 0x0c, 0x0b, 0x0a, 0x09, 0x07,
        0x05, 0x04, 0x03, 0x02, 0x01, 0x01, 0x00, 0x00,
        0x00, 0x01, 0x01, 0x02, 0x03, 0x04, 0x05, 0x07,
    ];
    let mut wave: [i16; WAVE_DATA_LENGTH] = [0; WAVE_DATA_LENGTH];
    let mut idx = 0;
    while idx < WAVE_DATA_LENGTH {
        let s = SAMPLES[idx] as i32;
        wave[idx] = if s < 0x0f {
            (((s + 1) << 12) - SETUP_U16) as i16
        } else {
            i16::MAX
        };
        idx += 1;
    }
    wave
};

const WAVE_7: [i16; WAVE_DATA_LENGTH] = {
    const SAMPLES: [u8; WAVE_DATA_LENGTH] = [
        0x0f, 0x0f, 0x0e, 0x00, 0x01, 0x01, 0x00, 0x00,
        0x0c, 0x0c, 0x0b, 0x00, 0x01, 0x01, 0x00, 0x00,
        0x0a, 0x0a, 0x09, 0x00, 0x01, 0x01, 0x00, 0x00,
        0x08, 0x08, 0x07, 0x00, 0x01, 0x01, 0x00, 0x00,
    ];
    let mut wave: [i16; WAVE_DATA_LENGTH] = [0; WAVE_DATA_LENGTH];
    let mut idx = 0;
    while idx < WAVE_DATA_LENGTH {
        let s = SAMPLES[idx] as i32;
        wave[idx] = if s < 0x0f {
            (((s + 1) << 12) - SETUP_U16) as i16
        } else {
            i16::MAX
        };
        idx += 1;
    }
    wave
};

pub const WAVE_FORMS: [&[i16]; NUM_OF_WAVE_FORMS] = [
    &WAVE_0,
    &WAVE_1,
    &WAVE_2,
    &WAVE_3,
    &WAVE_4,
    &WAVE_5,
    &WAVE_6,
    &WAVE_7,
];
