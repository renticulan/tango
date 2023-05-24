use crate::save;

pub const SAVE_SIZE: usize = 0xc7a8;
pub const MASK_OFFSET: usize = 0x3c84;
pub const GAME_NAME_OFFSET: usize = 0x4ba8;
pub const CHECKSUM_OFFSET: usize = 0x4b88;

#[derive(Clone)]
pub struct Save {
    buf: [u8; SAVE_SIZE],
}

impl Save {
    pub fn new(buf: &[u8]) -> Result<Self, save::Error> {
        let mut buf: [u8; SAVE_SIZE] = buf
            .get(..SAVE_SIZE)
            .and_then(|buf| buf.try_into().ok())
            .ok_or(save::Error::InvalidSize(buf.len()))?;
        save::mask_save(&mut buf[..], MASK_OFFSET);

        let n = &buf[GAME_NAME_OFFSET..][..20];
        if n != b"ROCKMANEXE4RO 040607" && n != b"ROCKMANEXE4RO 041217" {
            return Err(save::Error::InvalidGameName(n.to_vec()));
        }

        let save = Self { buf };
        let computed_checksum = save.compute_checksum();
        if save.checksum() != computed_checksum {
            return Err(save::Error::ChecksumMismatch {
                actual: save.checksum(),
                expected: vec![computed_checksum],
                shift: 0,
                attempt: 0,
            });
        }

        Ok(save)
    }

    pub fn checksum(&self) -> u32 {
        bytemuck::pod_read_unaligned::<u32>(&self.buf[CHECKSUM_OFFSET..][..4])
    }

    pub fn compute_checksum(&self) -> u32 {
        save::compute_save_raw_checksum(&self.buf, CHECKSUM_OFFSET) + 0x38
    }

    pub fn from_wram(buf: &[u8]) -> Result<Self, save::Error> {
        Ok(Self {
            buf: buf
                .get(..SAVE_SIZE)
                .and_then(|buf| buf.try_into().ok())
                .ok_or(save::Error::InvalidSize(buf.len()))?,
        })
    }

    pub fn current_navi(&self) -> u8 {
        self.buf[0x4ad1]
    }
}

impl save::Save for Save {
    fn view_chips(&self) -> Option<Box<dyn save::ChipsView + '_>> {
        Some(Box::new(ChipsView { save: self }))
    }

    fn view_navi(&self) -> Option<Box<dyn save::NaviView + '_>> {
        Some(Box::new(NaviView { save: self }))
    }

    fn as_raw_wram<'a>(&'a self) -> std::borrow::Cow<'a, [u8]> {
        std::borrow::Cow::Borrowed(&self.buf)
    }

    fn to_sram_dump(&self) -> Vec<u8> {
        let mut buf = vec![0; 65536];
        buf[..SAVE_SIZE].copy_from_slice(&self.buf);
        save::mask_save(&mut buf[..SAVE_SIZE], MASK_OFFSET);
        buf
    }

    fn rebuild_checksum(&mut self) {
        let checksum = self.compute_checksum();
        self.buf[CHECKSUM_OFFSET..][..4]
            .copy_from_slice(&bytemuck::cast::<_, [u8; std::mem::size_of::<u32>()]>(checksum));
    }
}

pub struct ChipsView<'a> {
    save: &'a Save,
}

impl<'a> save::ChipsView<'a> for ChipsView<'a> {
    fn num_folders(&self) -> usize {
        1
    }

    fn equipped_folder_index(&self) -> usize {
        0
    }

    fn regular_chip_index(&self, _folder_index: usize) -> Option<usize> {
        None
    }

    fn tag_chip_indexes(&self, _folder_index: usize) -> Option<[usize; 2]> {
        None
    }

    fn chip(&self, folder_index: usize, chip_index: usize) -> Option<save::Chip> {
        if folder_index >= 1 || chip_index >= 30 {
            return None;
        }

        let raw = bytemuck::pod_read_unaligned::<u16>(
            &self.save.buf
                [0x7500 + self.save.current_navi() as usize * (30 * 2) + chip_index * std::mem::size_of::<u16>()..]
                [..std::mem::size_of::<u16>()],
        );

        Some(save::Chip {
            id: (raw & 0x1ff) as usize,
            code: b"ABCDEFGHIJKLMNOPQRSTUVWXYZ*"[(raw >> 9) as usize] as char,
        })
    }
}

pub struct NaviView<'a> {
    save: &'a Save,
}

impl<'a> save::NaviView<'a> for NaviView<'a> {
    fn navi(&self) -> usize {
        self.save.buf[0x4ad1] as usize
    }
}
