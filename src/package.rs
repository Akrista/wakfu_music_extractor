use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use failure::{Error, ResultExt};

use std::{
    fmt,
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
};

#[derive(Clone, Debug)]
pub struct Header {
    file_offsets: Box<[OggOffset]>,
}

impl Header {
    pub fn read_from<R: Read>(file: &mut R) -> Result<Header, Error> {
        let size = file.read_u32::<LittleEndian>()?;
        let file_count = file.read_u32::<LittleEndian>()?;
        let file_offsets = (0..file_count)
            .map(|i| {
                // first 4 bytes are padding and the 2nd 4 bytes are the offset
                let mut buf = [0; 8];
                file.read_exact(&mut buf)
                    .with_context(|_| format!("Could not read offset of file {}", OggName(i)))?;
                let offset = LittleEndian::read_u32(&buf[4..8]);
                Ok(OggOffset(offset + size + 4))
            })
            .collect::<Result<Vec<_>, Error>>()
            .map(Vec::into_boxed_slice)
            .with_context(|_| format!("Could not read the offsets of {} files", file_count))?;

        Ok(Header {
            file_offsets,
        })
    }

    pub fn offsets(&self) -> &[OggOffset] {
        &*self.file_offsets
    }
}

#[derive(Clone, Debug)]
pub struct OggFile {
    name: OggName,
    data: Box<[u8]>,
}

impl OggFile {
    pub fn read_from<R>(file: &mut R, info: OggInfo) -> Result<OggFile, Error>
    where
        R: Read + Seek,
    {
        file.seek(SeekFrom::from(info.offset))
            .with_context(|_| format!("Could not seek to file '{}'", info.name))?;
        let size = file
            .read_u64::<LittleEndian>()
            .with_context(|_| format!("Could not read file size from '{}'", info.name))?;
        let buf = file
            .take(size)
            .bytes()
            .map(|byte_res| {
                Ok(byte_res
                    .with_context(|_| format!("Could not read '{}'", info.name))?
                    .wrapping_sub(1))
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(OggFile {
            name: info.name,
            data: buf.into_boxed_slice(),
        })
    }

    pub fn write_to_file(&self) -> Result<(), Error> {
        File::create(self.name.to_string())
            .with_context(|_| format!("Could not open output file '{}'", self.name))?
            .write_all(&self.data)
            .with_context(|_| format!("Could not write to '{}'", self.name))?;
        Ok(())
    }

    pub fn name(&self) -> String {
        self.name.to_string()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct OggInfo {
    name: OggName,
    offset: OggOffset,
}

impl OggInfo {
    pub fn new(name: OggName, offset: OggOffset) -> OggInfo {
        OggInfo { name, offset }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct OggName(u32);

impl OggName {
    pub fn new(nr: u32) -> OggName {
        OggName(nr)
    }
}

impl fmt::Display for OggName {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:03}.ogg", self.0)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct OggOffset(u32);

impl From<OggOffset> for SeekFrom {
    fn from(fo: OggOffset) -> SeekFrom {
        SeekFrom::Start(fo.0 as u64)
    }
}
