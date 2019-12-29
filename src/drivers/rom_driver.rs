use std::fs::File;
use std::io::Read;

pub struct RomDriver{
    pub rom: [u8; 3584],
    pub size: usize
}

impl RomDriver{
    // read in the ROM file as a byte buffer
    pub fn new(filename: &str) -> RomDriver{
        let mut file_handle = File::open(filename).expect("File not found");
        let mut buffer:[u8; 3584] = [0; 3584];
        let size = if let Ok(size) = file_handle.read(&mut buffer){size} else {0};
        RomDriver{
            rom: buffer,
            size
        }
    }
}