pub mod city_buffer;
pub mod deserializer;
use city_buffer::*;
use cjseq::CityJSONFeature;
use deserializer::to_cj_feature;

use crate::fb::{size_prefixed_root_as_city_feature, CityFeature};
use crate::{
    check_magic_bytes, size_prefixed_root_as_header, Column, Header, HEADER_MAX_BUFFER_SIZE,
};
use anyhow::{anyhow, Result};
use fallible_streaming_iterator::FallibleStreamingIterator;
use packed_rtree::PackedRTree;
use std::io::{self, Read, Seek, SeekFrom, Write};

pub mod geom_decoder;
use std::marker::PhantomData;
pub struct FcbReader<R> {
    reader: R,
    verify: bool,
    buffer: FcbBuffer,
}

pub struct FeatureIter<R, S> {
    reader: R,
    /// FlatBuffers verification
    verify: bool,
    // feature reading requires header access, therefore
    // header_buf is included in the FgbFeature struct.
    buffer: FcbBuffer,
    /// Select>ed features or None if no bbox filter
    item_filter: Option<Vec<packed_rtree::SearchResultItem>>,
    /// Number of selected features (None for undefined feature count)
    count: Option<usize>,
    /// Current feature number
    feat_no: usize,
    /// File offset within feature section
    cur_pos: u64,
    /// Reading state
    state: State,
    /// Whether or not the underlying reader is Seek
    seekable_marker: PhantomData<S>,
}

#[derive(Debug, PartialEq, Eq)]
enum State {
    Init,
    ReadFirstFeatureSize,
    Reading,
    Finished,
}

#[doc(hidden)]
pub mod reader_trait {
    pub struct Seekable;
    pub struct NotSeekable;
}
use reader_trait::*;

impl<R: Read> FcbReader<R> {
    pub fn open(reader: R) -> Result<FcbReader<R>> {
        let reader = Self::read_header(reader, true)?;
        Ok(reader)
    }

    /// Open a reader without verifying the FlatBuffers data.
    ///
    /// # Safety
    /// This function skips FlatBuffers verification. The caller must ensure that the input data
    /// is valid and properly formatted to avoid undefined behavior.
    pub unsafe fn open_unchecked(reader: R) -> Result<FcbReader<R>> {
        Self::read_header(reader, false)
    }

    fn read_header(mut reader: R, verify: bool) -> Result<FcbReader<R>> {
        let mut magic_buf: [u8; 8] = [0; 8];
        reader.read_exact(&mut magic_buf)?;
        if !check_magic_bytes(&magic_buf) {
            return Err(anyhow!("Missing magic bytes. Is this an fcb file?"));
        }

        let mut size_buf: [u8; 4] = [0; 4]; // MEMO: 4 bytes for size prefix. This is comvention for FlatBuffers's size_prefixed_root
        reader.read_exact(&mut size_buf)?;
        let header_size = u32::from_le_bytes(size_buf) as usize;
        if !((8..=HEADER_MAX_BUFFER_SIZE).contains(&header_size)) {
            return Err(anyhow!("Illegal header size: {header_size}"));
        }

        let mut header_buf = Vec::with_capacity(header_size + 4);
        header_buf.extend_from_slice(&size_buf);
        header_buf.resize(header_buf.capacity(), 0);
        reader.read_exact(&mut header_buf[4..])?;

        if verify {
            let _header = size_prefixed_root_as_header(&header_buf);
        }

        Ok(FcbReader {
            reader,
            verify,
            buffer: FcbBuffer {
                header_buf,
                features_buf: Vec::new(),
            },
        })
    }

    pub fn select_all_seq(mut self) -> Result<FeatureIter<R, NotSeekable>> {
        let index_size = self.index_size();
        // discard bufer of index
        io::copy(&mut (&mut self.reader).take(index_size), &mut io::sink())?;
        Ok(FeatureIter::new(
            self.reader,
            self.verify,
            self.buffer,
            None,
        ))
    }

    pub fn select_bbox_seq(
        mut self,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
    ) -> Result<FeatureIter<R, NotSeekable>> {
        // Read R-Tree index and build filter for features within bbox
        let header = self.buffer.header();
        if header.index_node_size() == 0 || header.features_count() == 0 {
            return Err(anyhow!("No index"));
        }
        let index = PackedRTree::from_buf(
            &mut self.reader,
            header.features_count() as usize,
            header.index_node_size(),
        )?;
        let (min_x, min_y, max_x, max_y) = (min_x as i64, min_y as i64, max_x as i64, max_y as i64);
        let list = index.search(min_x as f64, min_y as f64, max_x as f64, max_y as f64)?;
        debug_assert!(
            list.windows(2).all(|w| w[0].offset < w[1].offset),
            "Since the tree is traversed breadth first, list should be sorted by construction."
        );

        Ok(FeatureIter::new(
            self.reader,
            self.verify,
            self.buffer,
            Some(list),
        ))
    }
}

impl<R: Read + Seek> FcbReader<R> {
    pub fn select_all(mut self) -> Result<FeatureIter<R, Seekable>> {
        // skip index
        let index_size = self.index_size();
        self.reader.seek(SeekFrom::Current(index_size as i64))?;

        Ok(FeatureIter::new(
            self.reader,
            self.verify,
            self.buffer,
            None,
        ))
    }

    pub fn select_bbox(
        mut self,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
    ) -> Result<FeatureIter<R, Seekable>> {
        // Read R-Tree index and build filter for features within bbox
        let header = self.buffer.header();
        if header.index_node_size() == 0 || header.features_count() == 0 {
            return Err(anyhow!("No index"));
        }
        let (min_x, min_y, max_x, max_y) = (min_x as i64, min_y as i64, max_x as i64, max_y as i64);
        let list = PackedRTree::stream_search(
            &mut self.reader,
            header.features_count() as usize,
            PackedRTree::DEFAULT_NODE_SIZE,
            min_x as f64,
            min_y as f64,
            max_x as f64,
            max_y as f64,
        )?;
        debug_assert!(
            list.windows(2).all(|w| w[0].offset < w[1].offset),
            "Since the tree is traversed breadth first, list should be sorted by construction."
        );

        Ok(FeatureIter::new(
            self.reader,
            self.verify,
            self.buffer,
            Some(list),
        ))
    }
}

impl<R: Read> FcbReader<R> {
    pub fn header(&self) -> Header {
        self.buffer.header()
    }

    pub fn root_attr_schema(
        &self,
    ) -> Option<flatbuffers::Vector<flatbuffers::ForwardsUOffset<Column>>> {
        self.buffer.header().columns()
    }

    fn index_size(&self) -> u64 {
        let header = self.buffer.header();
        let feat_count = header.features_count() as usize;
        if header.index_node_size() > 0 && feat_count > 0 {
            PackedRTree::index_size(feat_count, header.index_node_size()) as u64
        } else {
            0
        }
    }
}

impl<R: Read> FallibleStreamingIterator for FeatureIter<R, NotSeekable> {
    type Item = FcbBuffer;
    type Error = anyhow::Error;

    fn advance(&mut self) -> Result<()> {
        if self.advance_finished() {
            return Ok(());
        }
        if let Some(filter) = &self.item_filter {
            let item = &filter[self.feat_no];
            if item.offset as u64 > self.cur_pos {
                if self.state == State::ReadFirstFeatureSize {
                    self.state = State::Reading;
                }
                // skip features
                let seek_bytes = item.offset as u64 - self.cur_pos;
                io::copy(&mut (&mut self.reader).take(seek_bytes), &mut io::sink())?;
                self.cur_pos += seek_bytes;
            }
        }
        self.read_feature()
    }

    fn get(&self) -> Option<&FcbBuffer> {
        self.iter_get()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter_size_hint()
    }
}

impl<R: Read + Seek> FallibleStreamingIterator for FeatureIter<R, Seekable> {
    type Item = FcbBuffer;
    type Error = anyhow::Error;

    fn advance(&mut self) -> Result<()> {
        if self.advance_finished() {
            return Ok(());
        }
        if let Some(filter) = &self.item_filter {
            let item = &filter[self.feat_no];
            if item.offset as u64 > self.cur_pos {
                if self.state == State::ReadFirstFeatureSize {
                    self.state = State::Reading;
                }
                // skip features
                let seek_bytes = item.offset as u64 - self.cur_pos;
                self.reader.seek(SeekFrom::Current(seek_bytes as i64))?;
                self.cur_pos += seek_bytes;
            }
        }
        self.read_feature()
    }

    fn get(&self) -> Option<&FcbBuffer> {
        self.iter_get()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter_size_hint()
    }
}

impl<R: Read> FeatureIter<R, NotSeekable> {
    pub fn cur_feature(&self) -> CityFeature {
        self.buffer.feature()
    }

    pub fn cur_cj_feature(&self) -> Result<CityJSONFeature> {
        let fcb_feature = self.buffer.feature();
        let root_attr_schema = self.buffer.header().columns();

        to_cj_feature(fcb_feature, root_attr_schema)
    }

    pub fn get_features(&mut self) -> Result<Vec<CityFeature>> {
        // Ok(features)
        todo!("implement")
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Option<&Self>> {
        self.advance()?;
        if self.get().is_some() {
            Ok(Some(self))
        } else {
            Ok(None)
        }
    }
}

impl<R: Read + Seek> FeatureIter<R, Seekable> {
    pub fn cur_feature(&self) -> CityFeature {
        self.buffer.feature()
    }
    /// Return current feature
    pub fn cur_cj_feature(&self) -> Result<CityJSONFeature> {
        let fcb_feature = self.buffer.feature();
        let root_attr_schema = self.buffer.header().columns();

        to_cj_feature(fcb_feature, root_attr_schema)
    }

    pub fn get_features(&mut self, _: impl Write) -> Result<()> {
        todo!("implement")
    }

    pub fn get_current_feature(&self) -> CityFeature {
        self.buffer.feature()
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Option<&Self>> {
        self.advance()?;
        if self.get().is_some() {
            Ok(Some(self))
        } else {
            Ok(None)
        }
    }
}

impl<R: Read, S> FeatureIter<R, S> {
    pub fn new(
        reader: R,
        verify: bool,
        buffer: FcbBuffer,
        item_filter: Option<Vec<packed_rtree::SearchResultItem>>,
    ) -> FeatureIter<R, S> {
        let mut iter = FeatureIter {
            reader,
            verify,
            buffer,
            item_filter,
            count: None,
            feat_no: 0,
            cur_pos: 0,
            state: State::Init,
            seekable_marker: PhantomData,
        };

        if iter.read_feature_size() {
            iter.state = State::Finished;
        } else {
            iter.state = State::ReadFirstFeatureSize
        }

        iter.count = match &iter.item_filter {
            Some(list) => Some(list.len()),
            None => {
                let feat_count = iter.buffer.header().features_count() as usize;
                if feat_count > 0 {
                    Some(feat_count)
                } else if iter.state == State::Finished {
                    Some(0)
                } else {
                    None
                }
            }
        };

        iter
    }

    pub fn header(&self) -> Header {
        self.buffer.header()
    }

    pub fn root_attr_schema(
        &self,
    ) -> Option<flatbuffers::Vector<flatbuffers::ForwardsUOffset<Column>>> {
        self.buffer.header().columns()
    }

    pub fn features_count(&self) -> Option<usize> {
        self.count
    }

    fn advance_finished(&mut self) -> bool {
        if self.state == State::Finished {
            return true;
        }
        if let Some(count) = self.count {
            if self.feat_no >= count {
                self.state = State::Finished;
                return true;
            }
        }
        false
    }

    /// Read feature size and return true if end of dataset reached
    fn read_feature_size(&mut self) -> bool {
        self.buffer.features_buf.resize(4, 0);
        self.cur_pos += 4;
        self.reader
            .read_exact(&mut self.buffer.features_buf)
            .is_err()
    }

    fn read_feature(&mut self) -> Result<()> {
        match self.state {
            State::ReadFirstFeatureSize => {
                self.state = State::Reading;
            }
            State::Reading => {
                if self.read_feature_size() {
                    self.state = State::Finished;
                    return Ok(());
                }
            }
            State::Finished => {
                debug_assert!(
                    false,
                    "shouldn't call read_feature on already finished Iter"
                );
                return Ok(());
            }
            State::Init => {
                unreachable!("should have read first feature size before reading any features")
            }
        }
        let sbuf = &self.buffer.features_buf;
        let feature_size = u32::from_le_bytes([sbuf[0], sbuf[1], sbuf[2], sbuf[3]]) as usize;
        self.buffer.features_buf.resize(feature_size + 4, 0);
        self.reader.read_exact(&mut self.buffer.features_buf[4..])?;
        if self.verify {
            let _feature = size_prefixed_root_as_city_feature(&self.buffer.features_buf)?;
        }
        self.feat_no += 1;
        self.cur_pos += feature_size as u64;

        Ok(())
    }

    fn iter_get(&self) -> Option<&FcbBuffer> {
        if self.state == State::Finished {
            None
        } else {
            debug_assert!(self.state == State::Reading);
            Some(&self.buffer)
        }
    }

    fn iter_size_hint(&self) -> (usize, Option<usize>) {
        if self.state == State::Finished {
            (0, Some(0))
        } else if let Some(count) = self.count {
            let remaining = count - self.feat_no;
            (remaining, Some(remaining))
        } else {
            (0, None)
        }
    }
}
