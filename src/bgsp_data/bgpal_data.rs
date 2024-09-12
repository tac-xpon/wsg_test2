use bgsp_lib2::bgsp_common::{Rgba, NUM_PALETTE_TBL, NUM_PALETTE_COL};

pub const COLOR_TBL: [[Rgba<u8>; NUM_PALETTE_COL]; NUM_PALETTE_TBL] = {
    let mut tbl: [[Rgba<u8>; NUM_PALETTE_COL]; NUM_PALETTE_TBL] = [[Rgba([0, 0, 0, 0]); NUM_PALETTE_COL]; NUM_PALETTE_TBL];
    let mut tbl_no = 0;
    while tbl_no < PALS.len() {
        let pal_data = PALS[tbl_no];
        let mut idx = 0;
        while idx < pal_data.len() {
            let (r, g, b, a) = pal_data[idx];
            tbl[tbl_no][idx] = Rgba([r, g, b, a]);
            idx += 1;
        }
        tbl_no += 1;
    }
    tbl
};

const PALS: &[&[(u8, u8, u8, u8)]] = &[
    &[],
    BG_PAL_1,
    BG_PAL_2,
    BG_PAL_3,
    BG_PAL_4,
    BG_PAL_5,
];

const BG_PAL_1: &[(u8, u8, u8, u8)] = &[
    (  0,   0,   0,   0),
    (216, 216, 216, 255),
    (  0,   0,   0, 255),
    (216, 216, 216, 255),
];

const BG_PAL_2: &[(u8, u8, u8, u8)] = &[
    (  0,   0,   0,   0),
    (216,   0,   0, 255),
    (  0,   0,   0, 255),
    (216,   0,   0, 255),
];

const BG_PAL_3: &[(u8, u8, u8, u8)] = &[
    (  0,   0,   0,   0),
    (  0, 216,   0, 255),
    (  0,   0,   0, 255),
    (  0, 216,   0, 255),
];

const BG_PAL_4: &[(u8, u8, u8, u8)] = &[
    (  0,   0,   0,   0),
    (216, 216,   0, 255),
    (  0,   0,   0, 255),
    (216, 216,   0, 255),
];

const BG_PAL_5: &[(u8, u8, u8, u8)] = &[
    (  0,   0,   0,   0),
    ( 64,  64,  96, 255),
    (  0,   0,   0, 255),
    ( 64,  64,  96, 255),
];
