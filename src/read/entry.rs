use std::io::{self, Cursor, ErrorKind};

use async_compression::tokio::bufread::ZlibDecoder;
use byteorder::{LittleEndian, ReadBytesExt};
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};

use crate::{
    XP3_INDEX_ADLR_IDENTIFIER, XP3_INDEX_CONTINUE, XP3_INDEX_FILE_IDENTIFIER,
    XP3_INDEX_INFO_IDENTIFIER, XP3_INDEX_SEGM_IDENTIFIER, XP3_INDEX_TIME_IDENTIFIER,
    XP3_PROTECTED_FLAG, read::error::XP3OpenError,
};

/// FileIndex for xp3 archive.
/// Contains information about file and data offsets.
#[derive(Debug, Clone, Default)]
pub struct XP3FileEntry {
    pub protected: bool,
    pub name: String,
    pub size: u64,
    pub archive_size: u64,
    pub checksum: u32,
    pub timestamp: Option<u64>,
}

#[derive(Debug, Default)]
pub(super) struct XP3Entries {
    pub entries: Vec<XP3FileEntry>,
    pub file_starts: Vec<usize>,
    pub segments: Vec<DataSegment>,
}

impl XP3Entries {
    pub async fn open(stream: &mut (impl AsyncRead + Unpin)) -> Result<Self, XP3OpenError> {
        let compressed = loop {
            let flag = stream.read_u8().await?;
            if flag == XP3_INDEX_CONTINUE {
                continue;
            }

            break flag != 0;
        };
        let size = stream.read_u64_le().await?;

        let entries = Self::default();
        Ok(if compressed {
            let original_size = stream.read_u64_le().await?;
            entries
                .open_inner(
                    ZlibDecoder::new(BufReader::new(stream.take(size))),
                    original_size,
                )
                .await?
        } else {
            entries.open_inner(stream.take(size), size).await?
        })
    }

    async fn open_inner(
        mut self,
        mut stream: impl AsyncRead + Unpin,
        original_size: u64,
    ) -> Result<Self, XP3OpenError> {
        let mut read = 0;
        let mut buf = vec![];
        while read < original_size {
            let (key, index_size) = read_index(&mut stream).await?;
            if read + index_size > original_size {
                return Err(XP3OpenError::Io(ErrorKind::UnexpectedEof.into()));
            }
            (&mut stream).take(index_size).read_to_end(&mut buf).await?;

            match key {
                XP3_INDEX_FILE_IDENTIFIER => {
                    self.read_file_index(&buf).await?;
                }

                _ => {
                    // Unknown identifier
                }
            }
            buf.clear();
            read += index_size + 12;
        }

        Ok(self)
    }

    async fn read_file_index(&mut self, data: &[u8]) -> Result<(), XP3OpenError> {
        let total_size = data.len() as u64;
        let mut cursor = Cursor::new(data);

        let mut entry = XP3FileEntry::default();
        let mut start_segment_index: Option<usize> = None;
        let mut prev_segment_index: Option<usize> = None;
        while cursor.position() < total_size {
            let (key, index_size) = read_index(&mut cursor).await?;
            if cursor.position() + index_size > total_size {
                return Err(XP3OpenError::Io(ErrorKind::UnexpectedEof.into()));
            }

            let mut sub_data =
                Cursor::new(&cursor.get_ref()[cursor.position() as usize..][..index_size as usize]);
            match key {
                XP3_INDEX_INFO_IDENTIFIER => {
                    entry.protected = ReadBytesExt::read_u32::<LittleEndian>(&mut sub_data)?
                        == XP3_PROTECTED_FLAG;
                    entry.size = ReadBytesExt::read_u64::<LittleEndian>(&mut sub_data)?;
                    entry.archive_size = ReadBytesExt::read_u64::<LittleEndian>(&mut sub_data)?;

                    let name_len = ReadBytesExt::read_u16::<LittleEndian>(&mut sub_data)?;
                    entry.name = char::decode_utf16(
                        sub_data.get_ref()[sub_data.position() as usize..][..name_len as usize]
                            .chunks_exact(2)
                            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]])),
                    )
                    .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
                    .collect();
                }

                XP3_INDEX_SEGM_IDENTIFIER => {
                    let count = sub_data.get_ref().len() / 28;
                    for _ in 0..count {
                        let id = self.segments.len();

                        let compressed =
                            ReadBytesExt::read_u32::<LittleEndian>(&mut sub_data)? != 0;
                        let start = ReadBytesExt::read_u64::<LittleEndian>(&mut sub_data)?;
                        let _size = ReadBytesExt::read_u64::<LittleEndian>(&mut sub_data)?;
                        let archive_size = ReadBytesExt::read_u64::<LittleEndian>(&mut sub_data)?;
                        self.segments.push(DataSegment {
                            compressed,
                            start,
                            archive_size,
                            next: None,
                        });

                        if start_segment_index.is_none() {
                            start_segment_index = Some(id);
                        }
                        if let Some(prev) = prev_segment_index.replace(id) {
                            self.segments[prev].next = Some(id);
                        }
                    }
                }

                XP3_INDEX_ADLR_IDENTIFIER => {
                    entry.checksum = ReadBytesExt::read_u32::<LittleEndian>(&mut sub_data)?;
                }

                XP3_INDEX_TIME_IDENTIFIER => {
                    entry.timestamp = Some(ReadBytesExt::read_u64::<LittleEndian>(&mut sub_data)?);
                }

                _ => {
                    // SKIP sections we don't know yet.
                }
            }

            cursor.set_position(cursor.position() + index_size);
        }

        let Some(start_segment_index) = start_segment_index else {
            return Err(XP3OpenError::InvalidSection(XP3_INDEX_SEGM_IDENTIFIER));
        };

        self.entries.push(entry);
        self.file_starts.push(start_segment_index);

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct DataSegment {
    pub compressed: bool,
    pub start: u64,
    pub archive_size: u64,
    pub next: Option<usize>,
}

// Read index key and size
async fn read_index(stream: &mut (impl AsyncRead + Unpin)) -> io::Result<(u32, u64)> {
    Ok((stream.read_u32_le().await?, stream.read_u64_le().await?))
}
