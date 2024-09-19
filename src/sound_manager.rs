#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub enum SoundIdx {
    FloorStart = 0x00,
    FloorFinish,
    FinalFloorFinish,
    Zapped,
    IshtarFloor,
    NormalFloor,
    DragonFloor,
    DruagaFloor,
    Chime,
    SlimeMove,
    Spell,
    Fire,
    BreakWall,
    DragonFlame,
    Sword1,
    Sword2,
    Sword3,
    Sword4,
    CutMonster,
    NoUse1,
    BlockSpell,
    OpenDoor,
    GetKey,
    GetItem,
    NoUse2,
    GilWalk,
    CreditUpPost,
    Miss,
    GameOver,
    NameEntry,
    Extend,
    CreditUpPre,
    _EndOfVariants
}

pub const NUM_SOUND_IDX: usize = SoundIdx::_EndOfVariants as usize;

impl From<i32> for SoundIdx {
    fn from(n: i32) -> Self {
        match n {
            0x00 => Self::FloorStart,
            0x01 => Self::FloorFinish,
            0x02 => Self::FinalFloorFinish,
            0x03 => Self::Zapped,
            0x04 => Self::IshtarFloor,
            0x05 => Self::NormalFloor,
            0x06 => Self::DragonFloor,
            0x07 => Self::DruagaFloor,
            0x08 => Self::Chime,
            0x09 => Self::SlimeMove,
            0x0a => Self::Spell,
            0x0b => Self::Fire,
            0x0c => Self::BreakWall,
            0x0d => Self::DragonFlame,
            0x0e => Self::Sword1,
            0x0f => Self::Sword2,
            0x10 => Self::Sword3,
            0x11 => Self::Sword4,
            0x12 => Self::CutMonster,
            0x13 => Self::NoUse1,
            0x14 => Self::BlockSpell,
            0x15 => Self::OpenDoor,
            0x16 => Self::GetKey,
            0x17 => Self::GetItem,
            0x18 => Self::NoUse2,
            0x19 => Self::GilWalk,
            0x1a => Self::CreditUpPost,
            0x1b => Self::Miss,
            0x1c => Self::GameOver,
            0x1d => Self::NameEntry,
            0x1e => Self::Extend,
            _    => Self::CreditUpPre
        }
    }
}

#[derive(Default, Debug)]
struct ChRegisters {
    wave_form: usize,
    freq: i32,
    gain: u16,
}

impl ChRegisters {
    fn clear(&mut self) {
        self.wave_form = 0;
        self.freq = 0;
        self.gain = 0;
    }

    fn get_registers(&self) -> (usize, i32, u16) {
        (self.wave_form, self.freq, self.gain)
    }
}

type SoundRegisters = [ChRegisters; 8];

#[derive(Default, Debug)]
struct ChPrepare {
    pre_data: ChRegisters,
    read_pos: usize,
    remain_frames: usize,
    unit_frames: usize,
    envelope: usize,
    envelope_read_pos: usize,
    work_c: usize,
    work_d: usize,
    work_e: usize,
    work_f: usize,
}

impl ChPrepare {
    fn clear(&mut self) {
        self.pre_data.clear();
        self.read_pos = 0;
        self.remain_frames = 0;
        self.unit_frames = 0;
        self.envelope_read_pos = 0;
        self.work_c = 0;
        self.work_d = 0;
        self.work_e = 0;
        self.work_f = 0;
    }
}

type PlayRequest = [usize; NUM_SOUND_IDX];
type PlayProgress = [bool; NUM_SOUND_IDX];
#[derive(Default, Debug)]
#[allow(dead_code)]
pub struct SoundManager {
    pub play_request: PlayRequest,
    play_progress: PlayProgress,
    group_0100: [ChPrepare; 7],
    group_0169: [ChPrepare; 2],
    group_0187: [ChPrepare; 4],
    group_01c3: [ChPrepare; 4],
    group_01ff: [ChPrepare; 4],
    group_023b: [ChPrepare; 4],
    group_0277: [ChPrepare; 2],
    group_0295: [ChPrepare; 2],
    group_02b3: [ChPrepare; 1],
    group_02c2: [ChPrepare; 8],
    group_033a: [ChPrepare; 4],
    group_0376: [ChPrepare; 3],
    registers: SoundRegisters,
}

#[allow(dead_code)]
impl SoundManager {
    pub fn get_ch_registers(&self) -> [(usize, i32, u16); 8] {
        [
            self.registers[0].get_registers(),
            self.registers[1].get_registers(),
            self.registers[2].get_registers(),
            self.registers[3].get_registers(),
            self.registers[4].get_registers(),
            self.registers[5].get_registers(),
            self.registers[6].get_registers(),
            self.registers[7].get_registers(),
        ]
    }

    pub fn run(&mut self) -> bool {
        type ScaleSet = [i32; 13];
        const SCALE_0: ScaleSet = [
            0x02_54A8, 0x02_7828, 0x02_9DB4, 0x02_C578, 0x02_EFCB, 0x03_1C82,
            0x03_4BC8, 0x03_7DF6, 0x03_B335, 0x03_EB87, 0x04_2717, 0x04_6669,
            0x00_0000,
        ];
        const SCALE_1: ScaleSet = [
            0x02_58C0, 0x02_7C6C, 0x02_A24F, 0x02_CA6B, 0x02_F4EA, 0x03_21F8,
            0x03_5196, 0x03_841A, 0x03_B9B1, 0x03_F25B, 0x04_2E6E, 0x04_6E17,
            0x00_0000,
        ];
        const SCALE_2: ScaleSet = [
            0x02_5CD9, 0x02_80DC, 0x02_A6EB, 0x02_CF5E, 0x02_FA08, 0x03_276E,
            0x03_5763, 0x03_8A3F, 0x03_C02E, 0x03_F92E, 0x04_35C5, 0x04_75C5,
            0x00_0000,
        ];

        const ENVELOPE_TBL: &[&[u8]] = &[
            &[0x0f, 0x10],
            &[0x0c, 0x10],
            &[0x0a, 0x10],
            &[0x07, 0x10],
            &[0x05, 0x10],
            &[0x03, 0x10],
            &[0x02, 0x10],
            &[0x09, 0x0b, 0x0d, 0x0f, 0x0c, 0x06, 0x04, 0x00, 0x10],
            &[0x0f, 0x0f, 0x0e, 0x0c, 0x0a, 0x08, 0x06, 0x04, 0x02, 0x00, 0x10],
            &[0x0a, 0x08, 0x06, 0x04, 0x02, 0x00, 0x10],
            &[0x0f, 0x12],
            &[0x0a, 0x12],
            &[0x0f, 0x11, 0x00, 0x10],
            &[0x0a, 0x11, 0x00, 0x10],
            &[0x08, 0x08, 0x11, 0x02, 0x10],
            &[0x0f, 0x0f, 0x0b, 0x0b, 0x13],
            &[0x06, 0x08, 0x0a, 0x0c, 0x0e, 0x0f, 0x0f, 0x0e, 0x0c, 0x0a, 0x08, 0x08, 0x03, 0x03, 0x06, 0x06, 0x02, 0x02, 0x00, 0x10],
            &[0x14, 0x0c, 0x0e, 0x0f, 0x0f, 0x0f, 0x0f, 0x0c, 0x06, 0x05, 0x04, 0x00, 0x10],
            &[0x14, 0x0f, 0x0e, 0x0c, 0x0a, 0x09, 0x08, 0x06, 0x04, 0x02, 0x00, 0x10],
            &[0x05, 0x11, 0x02, 0x10],
            &[0x07, 0x12],
            &[0x14, 0x0f, 0x0f, 0x0e, 0x0d, 0x0b, 0x0a, 0x09, 0x10],
            &[0x14, 0x06, 0x11, 0x00, 0x10],
            &[0x14, 0x0c, 0x0c, 0x0c, 0x0f, 0x0f, 0x0f, 0x0f, 0x11, 0x00, 0x10],
            &[0x08, 0x0a, 0x0c, 0x0e, 0x0f, 0x12],
            &[0x06, 0x07, 0x08, 0x12],
            &[0x00, 0x10],
        ];

        const FLOOR_START_EBA3: &[u8] = &[
            0xf0, 0x40, 0xf1, 0x0b, 0xf2, 0x02, 0x14, 0x08, 0xf1, 0x0d, 0x84, 0x04, 0xf1, 0x0b, 0x13, 0x0c,
            0xb4, 0x0c, 0xf1, 0x0d, 0xa4, 0x03, 0x84, 0x03, 0x64, 0x03, 0xa4, 0x03, 0xf1, 0x0b, 0x84, 0x09,
            0xf1, 0x0d, 0x64, 0x03, 0xf1, 0x0b, 0x84, 0x18, 0x64, 0x08, 0xf1, 0x0d, 0x84, 0x04, 0xf1, 0x0b,
            0x33, 0x0c, 0x13, 0x0c, 0xb4, 0x06, 0xa4, 0x06, 0x84, 0x06, 0x64, 0x06, 0x84, 0x24, 0xc0, 0x0c,
            0xf3,
        ];
        const FLOOR_START_EBE4: &[u8] = &[
            0xf0, 0x20, 0xf1, 0x14, 0xf2, 0x02, 0xc0, 0x03, 0x14, 0x08, 0x84, 0x04, 0x13, 0x0c, 0xb4, 0x0c,
            0xa4, 0x03, 0x84, 0x03, 0x64, 0x03, 0xa4, 0x03, 0x84, 0x09, 0x64, 0x03, 0x84, 0x18, 0x64, 0x08,
            0x84, 0x04, 0x33, 0x0c, 0x13, 0x0c, 0xb4, 0x06, 0xa4, 0x06, 0x84, 0x06, 0x64, 0x06, 0x84, 0x24,
            0xc0, 0x0c, 0xf3,
        ];
        const FLOOR_START_EC17: &[u8] = &[
            0xf0, 0x40, 0xf1, 0x0d, 0xf2, 0x02, 0xc0, 0x08, 0x54, 0x04, 0xf1, 0x0b, 0x84, 0x0c, 0x64, 0x0c,
            0xf1, 0x0d, 0x64, 0x03, 0x54, 0x03, 0x34, 0x03, 0x64, 0x03, 0xf1, 0x0b, 0x54, 0x09, 0xf1, 0x0d,
            0x34, 0x03, 0xf1, 0x0b, 0x54, 0x18, 0x34, 0x08, 0xf1, 0x0d, 0x54, 0x04, 0xf1, 0x0b, 0xb4, 0x0c,
            0xa4, 0x0c, 0x84, 0x06, 0x64, 0x06, 0x54, 0x06, 0x34, 0x06, 0x54, 0x24, 0xc0, 0x0c, 0xf3,
        ];
        const FLOOR_START_EC56: &[u8] = &[
            0xf0, 0x20, 0xf1, 0x14, 0xf2, 0x02, 0xc0, 0x0b, 0x54, 0x04, 0x84, 0x0c, 0x64, 0x0c, 0x64, 0x03,
            0x54, 0x03, 0x34, 0x03, 0x64, 0x03, 0x54, 0x09, 0x34, 0x03, 0x54, 0x18, 0x34, 0x08, 0x54, 0x04,
            0xb4, 0x0c, 0xa4, 0x0c, 0x84, 0x06, 0x64, 0x06, 0x54, 0x06, 0x34, 0x06, 0x54, 0x24, 0xc0, 0x0c,
            0xf3,
        ];
        const FLOOR_START_EC87: &[u8] = &[
            0xf0, 0x40, 0xf1, 0x0b, 0xf2, 0x02, 0xc0, 0x0c, 0x54, 0x0c, 0x34, 0x0c, 0xf1, 0x0d, 0x14, 0x03,
            0x14, 0x03, 0xb5, 0x03, 0x34, 0x03, 0xf1, 0x0b, 0x14, 0x09, 0xf1, 0x0d, 0xb5, 0x03, 0xf1, 0x0b,
            0x14, 0x18, 0xb5, 0x08, 0xf1, 0x0d, 0x14, 0x04, 0xf1, 0x0b, 0x64, 0x0c, 0x64, 0x0c, 0x44, 0x06,
            0x34, 0x06, 0x14, 0x06, 0xb5, 0x06, 0x14, 0x24, 0xc0, 0x0c, 0xf3,
        ];
        const FLOOR_START_ECC2: &[u8] = &[
            0xf0, 0x20, 0xf1, 0x14, 0xf2, 0x02, 0xc0, 0x0f, 0x54, 0x0c, 0x34, 0x0c, 0x14, 0x03, 0x14, 0x03,
            0xb5, 0x03, 0x34, 0x03, 0x14, 0x09, 0xb5, 0x03, 0x14, 0x18, 0xb5, 0x08, 0x14, 0x04, 0x64, 0x0c,
            0x64, 0x0c, 0x44, 0x06, 0x34, 0x06, 0x14, 0x06, 0xb5, 0x06, 0x14, 0x24, 0xc0, 0x0c, 0xf3,
        ];
        const FLOOR_START_ECF1: &[u8] = &[
            0xf0, 0x20, 0xf1, 0x0f, 0xf2, 0x02, 0x15, 0x3c, 0xf1, 0x0c, 0x15, 0x06, 0x86, 0x02, 0xc0, 0x01,
            0x86, 0x02, 0xc0, 0x01, 0x15, 0x06, 0x86, 0x06, 0x15, 0x06, 0x86, 0x06, 0xf1, 0x0f, 0x15, 0x30,
            0xf1, 0x0c, 0x15, 0x0c, 0x86, 0x0c, 0x16, 0x0c, 0xc0, 0x0c, 0xf3,
        ];
        const FLOOR_START_SCORE: &[(usize, &ScaleSet, &[u8])] = &[
            (0xeba3, &SCALE_0, FLOOR_START_EBA3),
            (0xebe4, &SCALE_0, FLOOR_START_EBE4),
            (0xec17, &SCALE_0, FLOOR_START_EC17),
            (0xec56, &SCALE_0, FLOOR_START_EC56),
            (0xec87, &SCALE_0, FLOOR_START_EC87),
            (0xecc2, &SCALE_0, FLOOR_START_ECC2),
            (0xecf1, &SCALE_0, FLOOR_START_ECF1),
            (0xecf1, &SCALE_1, FLOOR_START_ECF1),
        ];

        const FLOOR_FINISH_F6B6: &[u8] = &[
            0xf0, 0x40, 0xf1, 0x0b, 0xf2, 0x05, 0x94, 0x02, 0x23, 0x02, 0x13, 0x02, 0x23, 0x0a, 0x23, 0x02,
            0x53, 0x02, 0x53, 0x02, 0x53, 0x02, 0x73, 0x03, 0xf1, 0x0d, 0x73, 0x01, 0xf1, 0x0b, 0xa3, 0x02,
            0x93, 0x04, 0xf1, 0x0d, 0x93, 0x01, 0x93, 0x01, 0xf1, 0x0b, 0x93, 0x0c, 0xf3,
        ];
        const FLOOR_FINISH_F6E3: &[u8] = &[
            0xf0, 0x20, 0xf1, 0x0b, 0xf2, 0x05, 0x24, 0x06, 0x95, 0x06, 0x65, 0x06, 0x55, 0x02, 0x25, 0x02,
            0x55, 0x02, 0x75, 0x02, 0x25, 0x02, 0x75, 0x02, 0x95, 0x02, 0x25, 0x02, 0x95, 0x02, 0x25, 0x02,
            0x95, 0x02, 0x25, 0x02, 0xf1, 0x0f, 0x95, 0x06, 0xf3,
        ];
        const FLOOR_FINISH_F70C: &[u8] = &[
            0xf0, 0x40, 0xf1, 0x0b, 0xf2, 0x05, 0x64, 0x02, 0x94, 0x02, 0x94, 0x02, 0x94, 0x0a, 0x94, 0x02,
            0x23, 0x02, 0x23, 0x02, 0x23, 0x02, 0x43, 0x03, 0xf1, 0x0d, 0x43, 0x01, 0xf1, 0x0b, 0x73, 0x02,
            0x63, 0x04, 0xf1, 0x0d, 0x63, 0x01, 0x63, 0x01, 0xf1, 0x0b, 0x63, 0x0c, 0xf3,
        ];
        const FLOOR_FINISH_F739: &[u8] = &[
            0xf0, 0x40, 0xf1, 0x0b, 0xf2, 0x05, 0x24, 0x02, 0x64, 0x02, 0x44, 0x02, 0x64, 0x0a, 0x64, 0x02,
            0xa4, 0x02, 0xa4, 0x02, 0xa4, 0x02, 0x03, 0x03, 0xf1, 0x0d, 0x03, 0x01, 0xf1, 0x0b, 0x43, 0x02,
            0x23, 0x04, 0xf1, 0x0d, 0x23, 0x01, 0x23, 0x01, 0xf1, 0x0b, 0x23, 0x0c, 0xf3,
        ];
        const FLOOR_FINISH_F766: &[u8] = &[
            0xf0, 0x40, 0xf1, 0x14, 0xf2, 0x05, 0xc0, 0x02, 0x94, 0x02, 0x23, 0x02, 0x13, 0x02, 0x23, 0x0a,
            0x23, 0x02, 0x53, 0x02, 0x53, 0x02, 0x53, 0x02, 0x73, 0x03, 0x73, 0x01, 0xa3, 0x02, 0x93, 0x04,
            0x93, 0x01, 0x93, 0x01, 0x93, 0x0c, 0xf3,
        ];
        const FLOOR_FINISH_F78D: &[u8] = &[
            0xf0, 0x40, 0xf1, 0x14, 0xf2, 0x05, 0xc0, 0x02, 0x64, 0x02, 0x94, 0x02, 0x94, 0x02, 0x94, 0x0a,
            0x94, 0x02, 0x23, 0x02, 0x23, 0x02, 0x23, 0x02, 0x43, 0x03, 0x43, 0x01, 0x73, 0x02, 0x63, 0x04,
            0x63, 0x01, 0x63, 0x01, 0x63, 0x0c, 0xf3,
        ];
        const FLOOR_FINISH_SCORE: &[(usize, &ScaleSet, &[u8])] = &[
            (0xf6b6, &SCALE_0, FLOOR_FINISH_F6B6),
            (0xf6e3, &SCALE_0, FLOOR_FINISH_F6E3),
            (0xf70c, &SCALE_0, FLOOR_FINISH_F70C),
            (0xf739, &SCALE_0, FLOOR_FINISH_F739),
            (0xf6e3, &SCALE_1, FLOOR_FINISH_F6E3),
            (0xf766, &SCALE_1, FLOOR_FINISH_F766),
            (0xf78d, &SCALE_1, FLOOR_FINISH_F78D),
            (0xf6e3, &SCALE_2, FLOOR_FINISH_F6E3),
        ];
        const MUSIC_SCORES: &[&[(usize, &ScaleSet, &[u8])]] = &[
            FLOOR_START_SCORE,
            FLOOR_FINISH_SCORE,
        ];

        fn prepare(idx: usize, request: &mut PlayRequest, progress: &mut PlayProgress, score: &[&[(usize, &ScaleSet, &[u8])]], group: &mut[ChPrepare], registers: &mut[ChRegisters], start_ch: usize) {
            let mut finishd = false;
            for (part_no, ch_score) in score[idx].iter().enumerate() {
                if finishd {
                    group[part_no].clear();
                    continue;
                }
                if !progress[idx] {
                    group[part_no].clear(); // !! 本来不要だが、remain_frames のアンダーフロー対策のため !!
                }
                loop {
                    //println!("part no.{}/{:?}\n", part_no, group[part_no]);
                    let read_pos = group[part_no].read_pos;
                    let r0 = ch_score.2[read_pos];
                    if r0 >= 0xf0 {
                        match r0 {
                            0xf0 => {
                                let r1 = ch_score.2[read_pos + 1];
                                group[part_no].pre_data.wave_form = (r1 >> 4) as usize;
                                group[part_no].read_pos += 2;
                                println!("{}.wave form:{}", part_no, group[part_no].pre_data.wave_form);
                                continue;
                            },
                            0xf1 => {
                                let r1 = ch_score.2[read_pos + 1];
                                group[part_no].envelope = r1 as usize;
                                group[part_no].work_c = 0;
                                group[part_no].read_pos += 2;
                                println!("{}.envelope:{}", part_no, group[part_no].envelope);
                                continue;
                            },
                            0xf2 => {
                                let r1 = ch_score.2[read_pos + 1];
                                group[part_no].unit_frames = r1 as usize;
                                group[part_no].read_pos += 2;
                                println!("{}.unit:{}", part_no, group[part_no].unit_frames);
                                continue;
                            },
                            0xf3 => {
                                if idx == SoundIdx::CreditUpPre as usize {
                                    request[idx] -= 1;
                                    if request[idx] == 0 {
                                        request[SoundIdx::CreditUpPost as usize] = 1;
                                    }
                                } else {
                                    request[idx] = 0;
                                }
                                progress[idx] = false;
                                finishd = true;
                                group[part_no].clear();
                                println!("{}.end mark", part_no);
                                break;
                            },
                            0xf4 => {
                                let r1 = ch_score.2[read_pos + 1];
                                group[part_no].work_d += 1;
                                if group[part_no].work_d < r1 as usize {
                                    let r2 = ch_score.2[read_pos + 2];
                                    let r3 = ch_score.2[read_pos + 3];
                                    let adr = (r2 as usize) << 8 + r3 as usize;
                                    group[part_no].read_pos = adr - ch_score.0;
                                } else {
                                    group[part_no].work_d = 0;
                                    group[part_no].read_pos += 4;
                                }
                                println!("{}.repeat {}/{}", part_no, group[part_no].work_d, r1);
                                continue;
                            },
                            0xf5 => {
                                let r1 = ch_score.2[read_pos + 1];
                                group[part_no].work_e += 1;
                                if group[part_no].work_e == r1 as usize {
                                    group[part_no].work_e = 0;
                                    let r2 = ch_score.2[read_pos + 2];
                                    let r3 = ch_score.2[read_pos + 3];
                                    let adr = (r2 as usize) << 8 + r3 as usize;
                                    group[part_no].read_pos = adr - ch_score.0;
                                } else {
                                    group[part_no].read_pos += 4;
                                }
                                println!("{}.until {}/{}", part_no, group[part_no].work_e, r1);
                                continue;
                            },
                            0xf6 => {
                                let r1 = ch_score.2[read_pos + 1];
                                let r2 = ch_score.2[read_pos + 2];
                                let adr = (r1 as usize) << 8 + r2 as usize;
                                group[part_no].read_pos = adr - ch_score.0;
                                println!("{}.jump", part_no);
                                continue;
                            },
                            _ => continue,
                        }
                    } else {
                        let key = r0 >> 4;
                        let oct = r0 & 0x0f;
                        group[part_no].pre_data.freq = ch_score.1[key as usize] >> oct;
                        if group[part_no].remain_frames == 0 {
                            let len = ch_score.2[read_pos + 1] as usize * group[part_no].unit_frames;
                            println!("{}.key:{} oct:{} len:{}", part_no, key, oct, len);
                            group[part_no].remain_frames = len;
                            if group[part_no].work_c == 0 {
                                group[part_no].envelope_read_pos = 0;
                            }
                        }
                        let envelope = group[part_no].envelope;
                        loop {
                            let env_pos = group[part_no].envelope_read_pos;
                            let g = ENVELOPE_TBL[envelope][env_pos];
                            let gain = match g {
                                0x10 => {
                                    let gain = ENVELOPE_TBL[envelope][env_pos - 1] as u16;
                                    gain
                                },
                                0x11 => {
                                    let gain = group[part_no].pre_data.gain;
                                    if gain > 0 {
                                        if (gain - 1) <= ENVELOPE_TBL[envelope][env_pos + 1] as u16 {
                                            group[part_no].envelope_read_pos += 1;
                                        }
                                        gain - 1
                                    } else {
                                        group[part_no].envelope_read_pos += 1;
                                        0
                                    }
                                },
                                0x12 => {
                                    let remain = group[part_no].remain_frames;
                                    let gain = group[part_no].pre_data.gain;
                                    if remain > gain as usize {
                                        gain
                                    } else {
                                        (remain - 1) as u16
                                    }
                                },
                                0x13 => {
                                    group[part_no].work_c = 0;
                                    group[part_no].envelope_read_pos = 0;
                                    continue;
                                },
                                0x14 => {
                                    group[part_no].work_c = 1;
                                    group[part_no].envelope_read_pos += 1;
                                    continue;
                                },
                                _ => {
                                    group[part_no].envelope_read_pos += 1;
                                    g as u16
                                },
                            };
                            group[part_no].pre_data.gain = gain;
                            break;
                        }
                    }
                    break;
                }
            }
            if !finishd {
                progress[idx] = true;
            }
            //print!("[");
            for part_no in 0..score[idx].len() {
                registers[start_ch + part_no] = ChRegisters { ..group[part_no].pre_data };
                group[part_no].remain_frames -= 1; // !! usize がアンダーフロー(0以下)になるケース有り !!
                if group[part_no].remain_frames == 0 {
                    group[part_no].read_pos += 2;
                }
                //let ch = start_ch + part_no;
                //print!("({:1}, 0x{:04X}, 0x{:02X}), ", registers[ch].wave_form, registers[ch].freq, registers[ch].gain);
                //print!(" ch.{}:{:?} e={}:{}", start_ch + part_no, registers[start_ch + part_no], group[part_no].envelope, group[part_no].envelope_read_pos);
            }
            //print!("],\n");
        }

        {
            const IDX: usize = SoundIdx::FloorStart as usize;
            const START_CH: usize = 0;
            let group = &mut self.group_02c2;
            if self.play_request[IDX] > 0 {
                prepare(IDX, &mut self.play_request, &mut self.play_progress, MUSIC_SCORES, group, &mut self.registers, START_CH);
            } else {
                self.play_progress[IDX] = false;
            }
        }
        {
            const IDX: usize = SoundIdx::FloorFinish as usize;
            const START_CH: usize = 0;
            let group = &mut self.group_02c2;
            if self.play_request[IDX] > 0 {
                prepare(IDX, &mut self.play_request, &mut self.play_progress, MUSIC_SCORES, group, &mut self.registers, START_CH);
            } else {
                self.play_progress[IDX] = false;
            }
        }
        {
            const IDX: usize = SoundIdx::FinalFloorFinish as usize;
            const START_CH: usize = 0;
            let group = &mut self.group_02c2;
            if self.play_request[IDX] > 0 {
                prepare(IDX, &mut self.play_request, &mut self.play_progress, MUSIC_SCORES, group, &mut self.registers, START_CH);
            } else {
                self.play_progress[IDX] = false;
            }
        }
        {
            const IDX: usize = SoundIdx::Zapped as usize;
            const START_CH: usize = 0;
            let group = &mut self.group_02c2;
            if self.play_request[IDX] > 0 {
                prepare(IDX, &mut self.play_request, &mut self.play_progress, MUSIC_SCORES, group, &mut self.registers, START_CH);
            } else {
                self.play_progress[IDX] = false;
            }
        }
        {
            const IDX: usize = SoundIdx::IshtarFloor as usize;
            const START_CH: usize = 0;
            let group = &mut self.group_0100;
            if self.play_request[IDX] > 0 {
                prepare(IDX, &mut self.play_request, &mut self.play_progress, MUSIC_SCORES, group, &mut self.registers, START_CH);
            } else {
                self.play_progress[IDX] = false;
            }
        }
        {
            const IDX: usize = SoundIdx::NormalFloor as usize;
            const START_CH: usize = 0;
            let group = &mut self.group_0100;
            if self.play_request[IDX] > 0 {
                prepare(IDX, &mut self.play_request, &mut self.play_progress, MUSIC_SCORES, group, &mut self.registers, START_CH);
            } else {
                self.play_progress[IDX] = false;
            }
        }
        {
            const IDX: usize = SoundIdx::DragonFloor as usize;
            const START_CH: usize = 0;
            let group = &mut self.group_0100;
            if self.play_request[IDX] > 0 {
                prepare(IDX, &mut self.play_request, &mut self.play_progress, MUSIC_SCORES, group, &mut self.registers, START_CH);
            } else {
                self.play_progress[IDX] = false;
            }
        }
        {
            const IDX: usize = SoundIdx::DruagaFloor as usize;
            const START_CH: usize = 0;
            let group = &mut self.group_0100;
            if self.play_request[IDX] > 0 {
                prepare(IDX, &mut self.play_request, &mut self.play_progress, MUSIC_SCORES, group, &mut self.registers, START_CH);
            } else {
                self.play_progress[IDX] = false;
            }
        }
        {
            const IDX: usize = SoundIdx::Chime as usize;
            const START_CH: usize = 0;
            let group = &mut self.group_01ff;
            if self.play_request[IDX] > 0 {
                prepare(IDX, &mut self.play_request, &mut self.play_progress, MUSIC_SCORES, group, &mut self.registers, START_CH);
            } else {
                self.play_progress[IDX] = false;
            }
        }
        {
            const IDX: usize = SoundIdx::SlimeMove as usize;
            const START_CH: usize = 4;
            let group = &mut self.group_0169;
            if self.play_request[IDX] > 0 {
                self.play_progress[IDX] = false;
                prepare(IDX, &mut self.play_request, &mut self.play_progress, MUSIC_SCORES, group, &mut self.registers, START_CH);
                self.play_request[IDX] = 0;
            } else {
                if self.play_progress[IDX] {
                    prepare(IDX, &mut self.play_request, &mut self.play_progress, MUSIC_SCORES, group, &mut self.registers, START_CH);
                }
            }
        }
        false
    }
}