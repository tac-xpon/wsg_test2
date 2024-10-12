mod scale_set;
use scale_set::*;
mod envelope_tbl;
use envelope_tbl::*;
mod sound_score;
use sound_score::*;
mod sound_index;
use sound_index::*;

#[derive(Default, Debug)]
struct ChRegisters {
    wave_form: usize,
    freq: i32,
    gain: i32,
}

impl ChRegisters {
    fn clear(&mut self) {
        self.wave_form = 0;
        self.freq = 0;
        self.gain = 0;
    }

    fn get_registers(&self) -> (usize, i32, i32) {
        (self.wave_form, self.freq, self.gain)
    }
}

type SoundRegisters = [ChRegisters; 8];

#[derive(Default, Debug)]
struct ChPrepare<'a> {
    pre_data: ChRegisters,
    read_adr: &'a[u8],
    remain_frames: usize,
    unit_frames: usize,
    envelope: usize,
    envelope_read_pos: usize,
    work_c: usize,
    // work_d: usize,
    // work_e: usize,
    // work_f: usize,
}

impl<'a> ChPrepare<'a> {
    fn clear(&mut self) {
        self.pre_data.clear();
        self.remain_frames = 0;
        self.unit_frames = 0;
        self.envelope_read_pos = 0;
        self.work_c = 0;
        // self.work_d = 0;
        // self.work_e = 0;
        // self.work_f = 0;
    }
}

type PlayRequest = [i32; NUM_SOUND_IDX];
type PlayProgress = [bool; NUM_SOUND_IDX];
#[derive(Default, Debug)]
pub struct SoundManager<'a> {
    pub play_request: PlayRequest,
    play_progress: PlayProgress,
    group_0100: [ChPrepare<'a>; 7],
    group_0169: [ChPrepare<'a>; 2],
    group_0187: [ChPrepare<'a>; 4],
    group_01c3: [ChPrepare<'a>; 4],
    group_01ff: [ChPrepare<'a>; 4],
    group_023b: [ChPrepare<'a>; 4],
    group_0277: [ChPrepare<'a>; 2],
    group_0295: [ChPrepare<'a>; 2],
    group_02b3: [ChPrepare<'a>; 1],
    group_02c2: [ChPrepare<'a>; 8],
    group_033a: [ChPrepare<'a>; 4],
    group_0376: [ChPrepare<'a>; 3],
    registers: SoundRegisters,
    pub suppress_last_silence: bool,
}

#[allow(dead_code)]
impl<'a> SoundManager<'a> {
    pub fn clear(&mut self) {
        for request in self.play_request.iter_mut() {
            *request = 0;
        }
        for progress in self.play_progress.iter_mut() {
            *progress = false;
        }
        for ch_prepare in self.group_0100.iter_mut() {
            ch_prepare.clear();
        }
        for ch_prepare in self.group_0169.iter_mut() {
            ch_prepare.clear();
        }
        for ch_prepare in self.group_0187.iter_mut() {
            ch_prepare.clear();
        }
        for ch_prepare in self.group_01c3.iter_mut() {
            ch_prepare.clear();
        }
        for ch_prepare in self.group_01ff.iter_mut() {
            ch_prepare.clear();
        }
        for ch_prepare in self.group_023b.iter_mut() {
            ch_prepare.clear();
        }
        for ch_prepare in self.group_0277.iter_mut() {
            ch_prepare.clear();
        }
        for ch_prepare in self.group_0295.iter_mut() {
            ch_prepare.clear();
        }
        for ch_prepare in self.group_02b3.iter_mut() {
            ch_prepare.clear();
        }
        for ch_prepare in self.group_02c2.iter_mut() {
            ch_prepare.clear();
        }
        for ch_prepare in self.group_033a.iter_mut() {
            ch_prepare.clear();
        }
        for ch_prepare in self.group_0376.iter_mut() {
            ch_prepare.clear();
        }
        self.clear_ch_registers();
    }

    pub fn get_ch_registers(&self) -> [(usize, i32, i32); 8] {
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

    pub fn clear_ch_registers(&mut self) {
        self.registers[0].clear();
        self.registers[1].clear();
        self.registers[2].clear();
        self.registers[3].clear();
        self.registers[4].clear();
        self.registers[5].clear();
        self.registers[6].clear();
        self.registers[7].clear();
    }

    pub fn play_progress(&self, sound_index: usize) -> bool {
        self.play_progress[sound_index]
    }

    pub fn run(&mut self) {
        fn prepare<'a>(idx: usize, request: &mut PlayRequest, progress: &mut PlayProgress, score: &'a[&[(&[u8], &ScaleSet)]], group: &mut[ChPrepare<'a>], registers: &mut[ChRegisters], start_ch: usize, suppress_last_silence: bool) {
            let mut finishd = false;
            for (part_no, ch_score) in score[idx].iter().enumerate() {
                if finishd {
                    group[part_no].clear();
                    continue;
                }
                if !progress[idx] {
                    group[part_no].read_adr = score[idx][part_no].0;
                    group[part_no].remain_frames = 0; // !! 本来不要だが、remain_frames のアンダーフロー対策のため !!
                }
                loop {
                    let r0 = group[part_no].read_adr[0];
                    if r0 >= 0xf0 {
                        match r0 {
                            0xf0 => {
                                let r1 = group[part_no].read_adr[1];
                                group[part_no].pre_data.wave_form = (r1 >> 4) as usize;
                                group[part_no].read_adr = &group[part_no].read_adr[2..];
                                #[cfg(feature="develop")]
                                {
                                    println!("{}.wave form:{}", part_no, group[part_no].pre_data.wave_form);
                                }
                                continue;
                            },
                            0xf1 => {
                                let r1 = group[part_no].read_adr[1];
                                group[part_no].envelope = r1 as usize;
                                group[part_no].work_c = 0;
                                group[part_no].read_adr = &group[part_no].read_adr[2..];
                                #[cfg(feature="develop")]
                                {
                                    println!("{}.envelope:{}", part_no, group[part_no].envelope);
                                }
                                continue;
                            },
                            0xf2 => {
                                let r1 = group[part_no].read_adr[1];
                                group[part_no].unit_frames = r1 as usize;
                                group[part_no].read_adr = &group[part_no].read_adr[2..];
                                #[cfg(feature="develop")]
                                {
                                    println!("{}.unit:{}", part_no, group[part_no].unit_frames);
                                }
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
                                #[cfg(feature="develop")]
                                {
                                    println!("{}.end mark", part_no);
                                }
                                break;
                            },
                            _ => continue,
                        }
                    } else {
                        let key = r0 >> 4;
                        let oct = r0 & 0x0f;
                        group[part_no].pre_data.freq = ch_score.1[key as usize] >> oct;
                        if group[part_no].remain_frames == 0 {
                            let len = group[part_no].read_adr[1] as usize * group[part_no].unit_frames;
                            #[cfg(feature="develop")]
                            {
                                println!("{}.key:{} oct:{} len:{}", part_no, key, oct, len);
                            }
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
                                    let gain = ENVELOPE_TBL[envelope][env_pos - 1] as i32;
                                    gain
                                },
                                0x11 => {
                                    let gain = group[part_no].pre_data.gain;
                                    if gain > 0 {
                                        if (gain - 1) <= ENVELOPE_TBL[envelope][env_pos + 1] {
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
                                        (remain - 1) as i32
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
                                    g
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
            for part_no in 0..score[idx].len() {
                registers[start_ch + part_no] = ChRegisters { ..group[part_no].pre_data };
                group[part_no].remain_frames -= 1; // !! usize がアンダーフロー(0以下)になるケース有り !!
                if group[part_no].remain_frames == 0 {
                    group[part_no].read_adr = &group[part_no].read_adr[2..];
                    // 独自実装：末尾の無音１フレームを出力しない
                    if suppress_last_silence && !finishd {
                        if group[part_no].read_adr[0] == 0xf3 {
                            if idx == SoundIdx::CreditUpPre as usize {
                                if request[idx] > 0 {
                                    request[idx] -= 1;
                                    if request[idx] == 0 {
                                        request[SoundIdx::CreditUpPost as usize] = 1;
                                    }
                                }
                            } else {
                                request[idx] = 0;
                            }
                            progress[idx] = false;
                            finishd = true;
                            group[part_no].clear();
                            #[cfg(feature="develop")]
                            {
                                println!("{}.end mark", part_no);
                            }
                        }
                    }
                }
            }
        }

        enum SoundType {
            OneShot,
            Retriggerable,
        }

        enum Group {
            G0100, G0169, G0187, G01c3, G01ff, G023b,
            G0277, G0295, G02b3, G02c2, G033a, G0376,
        }

        const SOUND_INFO: [(SoundType, Group, usize); SoundIdx::_EndOfVariants as usize] = [
            (SoundType::OneShot      , Group::G02c2, 0), // FloorStart
            (SoundType::OneShot      , Group::G02c2, 0), // FloorFinish
            (SoundType::OneShot      , Group::G02c2, 0), // FinalFloorFinish
            (SoundType::OneShot      , Group::G02c2, 0), // Zapped
            (SoundType::OneShot      , Group::G0100, 0), // IshtarFloor
            (SoundType::OneShot      , Group::G0100, 0), // NormalFloor
            (SoundType::OneShot      , Group::G0100, 0), // DragonFloor
            (SoundType::OneShot      , Group::G0100, 0), // DruagaFloor
            (SoundType::OneShot      , Group::G01ff, 0), // Chime
            (SoundType::Retriggerable, Group::G0169, 4), // SlimeMove
            (SoundType::OneShot      , Group::G0295, 4), // Spell
            (SoundType::Retriggerable, Group::G0277, 4), // Fire
            (SoundType::Retriggerable, Group::G0187, 3), // BreakWall
            (SoundType::OneShot      , Group::G0376, 3), // DragonFlame
            (SoundType::Retriggerable, Group::G01c3, 5), // Sword1
            (SoundType::Retriggerable, Group::G01c3, 5), // Sword2
            (SoundType::OneShot      , Group::G01c3, 5), // Sword3
            (SoundType::Retriggerable, Group::G01c3, 5), // Sword4
            (SoundType::Retriggerable, Group::G01c3, 4), // CutMonster
            (SoundType::Retriggerable, Group::G01c3, 5), // NoUse1
            (SoundType::OneShot      , Group::G01c3, 5), // BlockSpell
            (SoundType::OneShot      , Group::G01ff, 0), // OpenDoor
            (SoundType::OneShot      , Group::G01ff, 0), // GetKey
            (SoundType::OneShot      , Group::G01ff, 0), // GetItem
            (SoundType::OneShot      , Group::G0100, 0), // NoUse2
            (SoundType::OneShot      , Group::G02b3, 7), // GilWalk
            (SoundType::OneShot      , Group::G033a, 4), // CreditUpPost
            (SoundType::OneShot      , Group::G02c2, 0), // Miss
            (SoundType::OneShot      , Group::G02c2, 0), // GameOver
            (SoundType::OneShot      , Group::G02c2, 0), // NameEntry
            (SoundType::OneShot      , Group::G033a, 4), // Extend
            (SoundType::OneShot      , Group::G023b, 4), // CreditUpPre
        ];

        for (idx, info) in SOUND_INFO.iter().enumerate() {
            let start_ch = info.2;
            let group = match info.1 {
                Group::G0100 => &mut self.group_0100[0..],
                Group::G0169 => &mut self.group_0169[0..],
                Group::G0187 => &mut self.group_0187[0..],
                Group::G01c3 => &mut self.group_01c3[0..],
                Group::G01ff => &mut self.group_01ff[0..],
                Group::G023b => &mut self.group_023b[0..],
                Group::G0277 => &mut self.group_0277[0..],
                Group::G0295 => &mut self.group_0295[0..],
                Group::G02b3 => &mut self.group_02b3[0..],
                Group::G02c2 => &mut self.group_02c2[0..],
                Group::G033a => &mut self.group_033a[0..],
                Group::G0376 => &mut self.group_0376[0..],
            };
            match info.0 {
                SoundType::OneShot => if self.play_request[idx] != 0 {
                    prepare(idx, &mut self.play_request, &mut self.play_progress, MUSIC_SCORES, group, &mut self.registers, start_ch, self.suppress_last_silence);
                } else {
                    self.play_progress[idx] = false
                }
                SoundType::Retriggerable => if self.play_request[idx] != 0 {
                    self.play_progress[idx] = false;
                    prepare(idx, &mut self.play_request, &mut self.play_progress, MUSIC_SCORES, group, &mut self.registers, start_ch, self.suppress_last_silence);
                    self.play_request[idx] = 0;
                } else {
                    if self.play_progress[idx] {
                        prepare(idx, &mut self.play_request, &mut self.play_progress, MUSIC_SCORES, group, &mut self.registers, start_ch, self.suppress_last_silence);
                    }
                }
            }
        }
    }
}