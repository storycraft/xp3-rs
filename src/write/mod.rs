pub mod error;

use std::io::{self, SeekFrom};

use tokio::io::{AsyncSeek, AsyncSeekExt, AsyncWrite, AsyncWriteExt};

use crate::{XP3_MAGIC, XP3_VERSION_IDENTIFIER, entry::XP3Entries, header::XP3Version, write::error::XP3WriteError};

#[derive(Debug)]
pub struct XP3Writer<T> {
    start: u64,
    index_offset_pos: u64,
    entries: XP3Entries,
    stream: T,
}

impl<T> XP3Writer<T>
where
    T: AsyncWrite + AsyncSeek + Unpin,
{
    pub async fn new(version: XP3Version, mut stream: T) -> io::Result<Self> {
        let start = stream.stream_position().await?;
        // write magic
        stream.write_all(&XP3_MAGIC).await?;
        stream.write_u8(1).await?;

        let index_offset_pos = match version {
            XP3Version::Old => {
                let pos = stream.stream_position().await?;
                // placeholder index offset value
                stream.write_u64_le(0).await?;
                pos
            }
            XP3Version::Current { minor } => {
                stream.write_u32_le(minor).await?;
                stream.write_u8(XP3_VERSION_IDENTIFIER).await?;
                stream.write_u64_le(0).await?;
                let pos = stream.stream_position().await?;
                // placeholder index offset value
                stream.write_u64_le(0).await?;
                pos
            }
        };

        Ok(Self {
            start,
            index_offset_pos,
            entries: XP3Entries::new(),
            stream,
        })
    }

    

    pub async fn finish(mut self) -> Result<T, XP3WriteError> {
        let index_start = self.stream.stream_position().await?;

        let end = self.stream.stream_position().await?;
        self.stream.seek(SeekFrom::Start(self.index_offset_pos)).await?;
        self.stream.write_u64_le(index_start - self.start).await?;
        self.stream.seek(SeekFrom::Start(end)).await?;
        self.stream.flush().await?;
        Ok(self.stream)
    }
}
