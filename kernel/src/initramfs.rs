//! Initramfs CPIO archive reader for ZuuposOS.
//!
//! Parses the CPIO `newc` format archive passed from bootloader into physical memory.
//! Allows searching and loading user-space binaries such as `/bin/hello`.

pub struct CpioEntry<'a> {
    pub name: &'a str,
    pub data: &'a [u8],
}

pub struct CpioReader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> CpioReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }
}

impl<'a> Iterator for CpioReader<'a> {
    type Item = CpioEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.offset + 110 <= self.data.len() {
            let header = &self.data[self.offset..self.offset + 110];

            // Validate newc magic: "070701" or "070702"
            if &header[0..6] != b"070701" && &header[0..6] != b"070702" {
                return None;
            }

            let filesize = parse_hex(&header[54..62])?;
            let namesize = parse_hex(&header[94..102])?;

            if namesize == 0 {
                return None;
            }

            let name_start = self.offset + 110;
            let name_end = name_start + namesize;
            if name_end > self.data.len() {
                return None;
            }

            // Exclude trailing null byte if present
            let raw_name = &self.data[name_start..name_end];
            let name_bytes = if !raw_name.is_empty() && raw_name[raw_name.len() - 1] == 0 {
                &raw_name[..raw_name.len() - 1]
            } else {
                raw_name
            };

            let name_str = core::str::from_utf8(name_bytes).unwrap_or("");

            // "TRAILER!!!" denotes the end of the cpio archive
            if name_str == "TRAILER!!!" {
                return None;
            }

            // Alignment padding: header (110) + namesize padded to 4 bytes
            let name_pad = (4 - ((110 + namesize) % 4)) % 4;
            let data_start = name_end + name_pad;
            let data_end = data_start + filesize;

            if data_end > self.data.len() {
                return None;
            }

            let file_data = &self.data[data_start..data_end];

            // Next entry starts padded to 4-byte boundary
            let data_pad = (4 - (filesize % 4)) % 4;
            self.offset = data_end + data_pad;

            // Skip directory entries or empty files
            if !name_str.is_empty() && name_str != "." {
                return Some(CpioEntry {
                    name: name_str,
                    data: file_data,
                });
            }
        }
        None
    }
}

/// Find a file in the cpio archive by path (e.g. "bin/hello" or "/bin/hello").
pub fn find_file<'a>(archive: &'a [u8], path: &str) -> Option<&'a [u8]> {
    let target = path.trim_start_matches('.').trim_start_matches('/');
    for entry in CpioReader::new(archive) {
        let entry_clean = entry.name.trim_start_matches('.').trim_start_matches('/');
        if entry_clean == target {
            return Some(entry.data);
        }
    }
    None
}

/// Parses an 8-character ASCII hexadecimal string into a usize.
fn parse_hex(bytes: &[u8]) -> Option<usize> {
    let mut val = 0usize;
    for &b in bytes {
        let digit = match b {
            b'0'..=b'9' => (b - b'0') as usize,
            b'a'..=b'f' => (b - b'a' + 10) as usize,
            b'A'..=b'F' => (b - b'A' + 10) as usize,
            _ => return None,
        };
        val = (val << 4) | digit;
    }
    Some(val)
}
