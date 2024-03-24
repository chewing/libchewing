use std::{
    any::Any,
    cmp::Ordering,
    collections::VecDeque,
    error::Error,
    ffi::CString,
    fmt::{Debug, Display},
    fs::File,
    io::{self, BufWriter, Read, Seek, Write},
    iter,
    mem::size_of,
    num::NonZeroUsize,
    path::Path,
    str::{self, Utf8Error},
};

use log::error;
use riff::{Chunk, ChunkContents, ChunkId, RIFF_ID};

use crate::zhuyin::{Syllable, SyllableSlice};

use super::{
    BuildDictionaryError, DictEntries, Dictionary, DictionaryBuilder, DictionaryInfo,
    DictionaryUpdateError, DuplicatePhraseError, Phrase,
};

const DICT_FORMAT: u32 = 0;

const CHEW: ChunkId = ChunkId { value: *b"CHEW" };
const FMT: ChunkId = ChunkId { value: *b"fmt " };
const DICT: ChunkId = ChunkId { value: *b"dict" };
const DATA: ChunkId = ChunkId { value: *b"data" };
const LIST: ChunkId = ChunkId { value: *b"LIST" };
const INFO: ChunkId = ChunkId { value: *b"INFO" };
const INAM: ChunkId = ChunkId { value: *b"INAM" };
const ICOP: ChunkId = ChunkId { value: *b"ICOP" };
const ILIC: ChunkId = ChunkId { value: *b"ILIC" };
const IREV: ChunkId = ChunkId { value: *b"IREV" };
const ISFT: ChunkId = ChunkId { value: *b"ISFT" };

struct TrieNodeView<'a>(&'a [u8]);

impl TrieNodeView<'_> {
    const SIZE: usize = 8;
    fn syllable(&self) -> u16 {
        u16::from_le_bytes(self.0[6..8].try_into().unwrap())
    }
    fn child_begin(&self) -> usize {
        u32::from_le_bytes(self.0[..4].try_into().unwrap()) as usize * Self::SIZE
    }
    fn child_end(&self) -> usize {
        (u32::from_le_bytes(self.0[..4].try_into().unwrap()) as usize)
            .saturating_add(u16::from_le_bytes(self.0[4..6].try_into().unwrap()) as usize)
            * Self::SIZE
    }
}

struct TrieLeafView<'a>(&'a [u8]);

impl TrieLeafView<'_> {
    const SIZE: usize = 8;
    fn reserved_zero(&self) -> u16 {
        u16::from_le_bytes(self.0[6..8].try_into().unwrap())
    }
    fn data_begin(&self) -> usize {
        u32::from_le_bytes(self.0[..4].try_into().unwrap()) as usize
    }
    fn data_end(&self) -> usize {
        (u32::from_le_bytes(self.0[..4].try_into().unwrap()) as usize)
            .saturating_add(u16::from_le_bytes(self.0[4..6].try_into().unwrap()) as usize)
    }
}

struct PhraseData<'a>(&'a [u8]);

impl<'a> PhraseData<'a> {
    fn is_valid(&self) -> bool {
        self.0.len() > 4 && self.len() <= self.0.len() && self.phrase_str().is_ok()
    }
    fn frequency(&self) -> u32 {
        u32::from_le_bytes(self.0[..4].try_into().unwrap())
    }
    fn phrase_str(&self) -> Result<&'a str, Utf8Error> {
        let len = self.0[4] as usize;
        let data = &self.0[5..];
        str::from_utf8(&data[..len])
    }
    fn len(&self) -> usize {
        5 + self.0[4] as usize
    }
}

/// A read-only dictionary using a pre-built [Trie][] index that is both space
/// efficient and fast to lookup.
///
/// `TrieDictionary`s are used as system dictionaries, or shared dictionaries.
/// The dictionary file is defined using the platform independent [Resource
/// Interchange File Format (RIFF)][RIFF], allowing them to be versioned and
/// shared easily.
///
/// `TrieDictionary`s can be created from anything that implements the [`Read`]
/// and [`Seek`] trait, as long as the underlying data conforms to the file
/// format spec.
///
/// A new dictionary can be built using a [`TrieDictionaryBuilder`].
///
/// # Examples
///
/// We may want to read a dictionary from a [File][`std::fs::File`]:
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let tmpdir = tempfile::tempdir()?;
/// # std::env::set_current_dir(&tmpdir.path())?;
/// use std::fs::File;
///
/// use chewing::{syl, zhuyin::{Bopomofo, Syllable}};
/// # use chewing::dictionary::{DictionaryBuilder, TrieDictionaryBuilder};
/// use chewing::dictionary::{Dictionary, TrieDictionary};
/// # let mut tempfile = File::create("dict.dat")?;
/// # let mut builder = TrieDictionaryBuilder::new();
/// # builder.insert(&[
/// #     syl![Bopomofo::Z, Bopomofo::TONE4],
/// #     syl![Bopomofo::D, Bopomofo::I, Bopomofo::AN]
/// # ], ("字典", 0).into());
/// # builder.write(&mut tempfile)?;
///
/// let mut file = File::open("dict.dat")?;
/// let dict = TrieDictionary::new(&mut file)?;
///
/// // Find the phrase ㄗˋㄉ一ㄢˇ (dictionary)
/// let mut phrase = dict.lookup_first_phrase(&[
///     syl![Bopomofo::Z, Bopomofo::TONE4],
///     syl![Bopomofo::D, Bopomofo::I, Bopomofo::AN]
/// ]);
/// assert_eq!("字典", phrase.unwrap().as_str());
/// # Ok(())
/// # }
/// ```
///
/// [Trie]: https://en.m.wikipedia.org/wiki/Trie
/// [RIFF]: https://en.m.wikipedia.org/wiki/Resource_Interchange_File_Format
#[derive(Debug)]
pub struct TrieDictionary {
    info: DictionaryInfo,
    dict: Vec<u8>,
    data: Vec<u8>,
}

#[derive(Debug)]
pub(crate) enum TrieDictionaryError {
    ReadOnly,
}

impl Display for TrieDictionaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "trie dictionary error")
    }
}

impl Error for TrieDictionaryError {}

fn read_only_error() -> DictionaryUpdateError {
    DictionaryUpdateError {
        source: Some(Box::new(TrieDictionaryError::ReadOnly)),
    }
}

impl TrieDictionary {
    /// Creates a new `TrieDictionary` instance from a file.
    ///
    /// The data in the file must conform to the dictionary format spec. See
    /// [`TrieDictionaryBuilder`] on how to build a dictionary.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use chewing::dictionary::TrieDictionary;
    ///
    /// let dict = TrieDictionary::open("dict.dat")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<TrieDictionary> {
        let mut file = File::open(path)?;
        TrieDictionary::new(&mut file)
    }
    /// Creates a new `TrieDictionary` instance from a input stream.
    ///
    /// The underlying data of the input stream must conform to the dictionary
    /// format spec. See [`TrieDictionaryBuilder`] on how to build a dictionary.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use std::fs::File;
    ///
    /// use chewing::dictionary::TrieDictionary;
    ///
    /// let mut file = File::open("dict.dat")?;
    /// let dict = TrieDictionary::new(&mut file)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new<T>(mut stream: T) -> io::Result<TrieDictionary>
    where
        T: Read + Seek,
    {
        let root = Chunk::read(&mut stream, 0)?;
        if root.id() != RIFF_ID {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }
        let file_type = root.read_type(&mut stream)?;
        if file_type != CHEW {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }
        let mut fmt_chunk = None;
        let mut dict_chunk = None;
        let mut data_chunk = None;
        let mut info_chunk = None;
        for chunk in root.iter(&mut stream) {
            let chunk = chunk?;
            match chunk.id() {
                FMT => fmt_chunk = Some(chunk),
                LIST => info_chunk = Some(chunk),
                DICT => dict_chunk = Some(chunk),
                DATA => data_chunk = Some(chunk),
                _ => (),
            }
        }
        let fmt_version = Self::read_fmt_version(
            fmt_chunk.ok_or_else(|| {
                io::Error::new(io::ErrorKind::UnexpectedEof, "expecting fmt chunk")
            })?,
            &mut stream,
        )?;
        if fmt_version != DICT_FORMAT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unsupported file version",
            ));
        }
        let mut info = DictionaryInfo::default();
        if let Some(chunk) = info_chunk {
            info = Self::read_dictionary_info(chunk, &mut stream)?;
        }
        let dict = dict_chunk
            .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "expecting dict chunk"))?
            .read_contents(&mut stream)?;
        let data = data_chunk
            .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "expecting data chunk"))?
            .read_contents(&mut stream)?;
        let crc32 = Crc32::new();
        if dict.len() < size_of::<u32>() {
            return Err(io::ErrorKind::InvalidData.into());
        }
        let dict_len = dict.len() - size_of::<u32>();
        let crc = u32::from_le_bytes(dict[dict_len..].try_into().unwrap());
        let check = crc32.check(&dict[..dict_len]);
        if crc != check {
            return Err(io::ErrorKind::InvalidData.into());
        }
        if data.len() < size_of::<u32>() {
            return Err(io::ErrorKind::InvalidData.into());
        }
        let data_len = data.len().saturating_sub(size_of::<u32>());
        let crc = u32::from_le_bytes(data[data_len..].try_into().unwrap());
        let check = crc32.check(&data[..data_len]);
        if crc != check {
            return Err(io::ErrorKind::InvalidData.into());
        }
        Ok(TrieDictionary { info, dict, data })
    }

    fn read_fmt_version<T>(fmt_chunk: Chunk, mut stream: T) -> io::Result<u32>
    where
        T: Read + Seek,
    {
        if fmt_chunk.len() != size_of::<u32>() as u32 {
            return Err(io::ErrorKind::InvalidData.into());
        }
        let bytes = fmt_chunk.read_contents(&mut stream)?;
        Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
    }

    fn read_dictionary_info<T>(list_chunk: Chunk, mut stream: T) -> io::Result<DictionaryInfo>
    where
        T: Read + Seek,
    {
        let mut info = DictionaryInfo::default();
        let chunk_type = list_chunk.read_type(&mut stream)?;
        if chunk_type != INFO {
            return Ok(info);
        }

        let mut chunks = vec![];

        for chunk in list_chunk.iter(&mut stream) {
            let chunk = chunk?;
            match chunk.id() {
                INAM | ICOP | ILIC | IREV | ISFT => chunks.push((chunk.id(), chunk)),
                _ => (),
            }
        }

        for (id, chunk) in chunks {
            let content = match id {
                INAM | ICOP | ILIC | IREV | ISFT => Some(
                    CString::new(chunk.read_contents(&mut stream)?)?
                        .into_string()
                        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?,
                ),
                _ => None,
            };
            match id {
                INAM => info.name = content,
                ICOP => info.copyright = content,
                ILIC => info.license = content,
                IREV => info.version = content,
                ISFT => info.software = content,
                _ => (),
            }
        }

        Ok(info)
    }
}

struct PhrasesIter<'a> {
    bytes: &'a [u8],
}

impl Iterator for PhrasesIter<'_> {
    type Item = Phrase;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.bytes.is_empty() {
            return None;
        }
        let phrase_data = PhraseData(self.bytes);
        if !phrase_data.is_valid() {
            error!("[!] file corruption detected: malformed phrase data.");
            return None;
        }
        self.bytes = &self.bytes[phrase_data.len()..];
        Some(Phrase::new(
            phrase_data.phrase_str().unwrap(),
            phrase_data.frequency(),
        ))
    }
}

macro_rules! bail_if_oob {
    ($begin:expr, $end:expr, $len:expr) => {
        if $begin >= $end || $end > $len {
            error!("[!] file corruption detected: index out of bound.");
            return vec![];
        }
    };
}

macro_rules! iter_bail_if_oob {
    ($begin:expr, $end:expr, $len:expr) => {
        if $begin >= $end || $end > $len {
            error!("[!] file corruption detected: index out of bound.");
            return None;
        }
    };
}

impl Dictionary for TrieDictionary {
    fn lookup_first_n_phrases(&self, syllables: &dyn SyllableSlice, first: usize) -> Vec<Phrase> {
        bail_if_oob!(0, TrieNodeView::SIZE, self.dict.len());
        let root = TrieNodeView(&self.dict[..TrieNodeView::SIZE]);
        let mut node = root;
        'next: for syl in syllables.as_slice().iter() {
            debug_assert!(syl.to_u16() != 0);
            bail_if_oob!(node.child_begin(), node.child_end(), self.dict.len());
            let mut child_nodes = self.dict[node.child_begin()..node.child_end()]
                .chunks_exact(TrieNodeView::SIZE)
                .map(TrieNodeView);
            if let Some(n) = child_nodes.find(|n| n.syllable() == syl.to_u16()) {
                node = n;
                continue 'next;
            }
            return vec![];
        }
        bail_if_oob!(node.child_begin(), node.child_end(), self.dict.len());
        let leaf_data = &self.dict[node.child_begin()..];
        bail_if_oob!(0, TrieLeafView::SIZE, leaf_data.len());
        let leaf = TrieLeafView(&leaf_data[..TrieLeafView::SIZE]);
        if leaf.reserved_zero() != 0 {
            return vec![];
        }
        bail_if_oob!(leaf.data_begin(), leaf.data_end(), self.data.len());
        PhrasesIter {
            bytes: &self.data[leaf.data_begin()..leaf.data_end()],
        }
        .take(first)
        .collect()
    }

    fn entries(&self) -> DictEntries<'_> {
        let mut results = Vec::new();
        let mut stack = Vec::new();
        let mut syllables = Vec::new();
        if self.dict.len() < TrieNodeView::SIZE {
            error!("[!] file corruption detected: index out of bound.");
            return Box::new(iter::empty());
        }
        let root = TrieNodeView(&self.dict[..TrieNodeView::SIZE]);
        let mut node = root;
        let mut done = false;
        let make_dict_entry =
            |syllables: &[u16], leaf: &TrieLeafView<'_>| -> (Vec<Syllable>, Vec<Phrase>) {
                debug_assert_eq!(leaf.reserved_zero(), 0);
                let phrases = PhrasesIter {
                    bytes: &self.data[leaf.data_begin()..leaf.data_end()],
                }
                .collect::<Vec<_>>();
                (
                    syllables
                        .iter()
                        // FIXME - skip invalid entry?
                        .map(|&syl_u16| Syllable::try_from(syl_u16).unwrap())
                        .collect::<Vec<_>>(),
                    phrases,
                )
            };
        let it = iter::from_fn(move || {
            if !results.is_empty() {
                return results.pop();
            }
            if done {
                return None;
            }
            // descend until find a leaf node which is not also a internal node.
            loop {
                iter_bail_if_oob!(node.child_begin(), node.child_end(), self.dict.len());
                let mut child_iter = self.dict[node.child_begin()..node.child_end()]
                    .chunks_exact(TrieNodeView::SIZE)
                    .map(TrieNodeView);
                let mut next = child_iter
                    .next()
                    .expect("syllable node should have at least one child node");
                if next.syllable() == 0 {
                    // found a leaf syllable node
                    iter_bail_if_oob!(node.child_begin(), node.child_end(), self.dict.len());
                    let leaf_data = &self.dict[node.child_begin()..];
                    let leaf = TrieLeafView(&leaf_data[..TrieLeafView::SIZE]);
                    iter_bail_if_oob!(leaf.data_begin(), leaf.data_end(), self.data.len());
                    results.push(make_dict_entry(&syllables, &leaf));
                    if let Some(second) = child_iter.next() {
                        next = second;
                    } else {
                        break;
                    }
                }
                node = next;
                syllables.push(node.syllable());
                stack.push(child_iter);
            }
            // ascend until we can go down again
            loop {
                if let Some(mut child_nodes) = stack.pop() {
                    syllables.pop();
                    if let Some(next) = child_nodes.next() {
                        debug_assert_ne!(next.syllable(), 0);
                        node = next;
                        stack.push(child_nodes);
                        syllables.push(node.syllable());
                        break;
                    }
                } else {
                    done = true;
                    break;
                }
            }
            results.pop()
        });
        let entries = it.flat_map(|(syllables, phrases)| {
            phrases
                .into_iter()
                .map(move |phrase| (syllables.clone(), phrase))
        });
        Box::new(entries)
    }

    fn about(&self) -> DictionaryInfo {
        self.info.clone()
    }

    fn reopen(&mut self) -> Result<(), DictionaryUpdateError> {
        Ok(())
    }

    fn flush(&mut self) -> Result<(), DictionaryUpdateError> {
        Ok(())
    }

    fn add_phrase(
        &mut self,
        _syllables: &dyn SyllableSlice,
        _phrase: Phrase,
    ) -> Result<(), DictionaryUpdateError> {
        Err(read_only_error())
    }

    fn update_phrase(
        &mut self,
        _syllables: &dyn SyllableSlice,
        _phrase: Phrase,
        _user_freq: u32,
        _time: u64,
    ) -> Result<(), DictionaryUpdateError> {
        Err(read_only_error())
    }

    fn remove_phrase(
        &mut self,
        _syllables: &dyn SyllableSlice,
        _phrase_str: &str,
    ) -> Result<(), DictionaryUpdateError> {
        Err(read_only_error())
    }
}

/// A builder to create a dictionary that can be loaded by the
/// [`TrieDictionary`].
///
/// # Examples
///
/// We may want to create a dictionary [File][`std::fs::File`]:
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let tmpdir = tempfile::tempdir()?;
/// # std::env::set_current_dir(&tmpdir.path())?;
/// use std::fs::File;
///
/// use chewing::{syl, zhuyin::Bopomofo};
/// use chewing::dictionary::{DictionaryBuilder, TrieDictionaryBuilder};
///
/// let mut file = File::create("dict.dat")?;
/// let mut builder = TrieDictionaryBuilder::new();
/// builder.insert(&[
///     syl![Bopomofo::Z, Bopomofo::TONE4],
///     syl![Bopomofo::D, Bopomofo::I, Bopomofo::AN]
/// ], ("字典", 0).into());
/// builder.write(&mut file)?;
/// # Ok(())
/// # }
/// ```
///
/// # RIFF File Format
///
/// The dictionary file is defined using the platform independent [Resource
/// Interchange File Format (RIFF)][RIFF], allowing them to be versioned and
/// shared easily. The text describing the RIFF format in this document is
/// adopted from the [WebP Container Specification][WebP].
///
/// The basic element of a RIFF file is a chunk. It consists of:
///
/// ```text
///  0                   1                   2                   3
///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                         Chunk FourCC                          |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                          Chunk Size                           |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                         Chunk Payload                         |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
///
/// - **Chunk FourCC: 32 bits**
///     - ASCII four-character code used for chunk identification.
/// - **Chunk Size: 32 bits (u32)**
///     - The size of the chunk not including this field, the chunk identifier
///       or padding.
/// - **Chunk Payload: Chunk Size bytes**
///     - The data payload. If Chunk Size is odd, a single padding byte -- that
///       SHOULD be 0 -- is added.
/// - **ChunkHeader('ABCD')**
///     - This is used to describe the FourCC and Chunk Size header of
///       individual chunks, where 'ABCD' is the FourCC for the chunk. This
///       element's size is 8 bytes.
///
/// All integers are little endian.
///
/// ## File Header
///
/// ```text
///  0                   1                   2                   3
///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |      'R'      |      'I'      |      'F'      |      'F'      |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                           File Size                           |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |      'C'      |      'H'      |      'E'      |      'W'      |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
///
/// - **'RIFF': 32 bits**
///     - The ASCII characters 'R' 'I' 'F' 'F'.
/// - **File Size: 32 bits (u32)**
///     - The size of the file in bytes starting at offset 8. The maximum value
///       of this field is 2^32 minus 10 bytes and thus the size of the whole
///       file is at most 4GiB minus 2 bytes.
/// - **'CHEW': 32 bits**
///     - The ASCII characters 'C' 'H' 'E' 'W'.
///
/// A TrieDictionary file MUST begin with a RIFF header with the FourCC 'CHEW'.
/// The file size in the header is the total size of the chunks that follow plus
/// 4 bytes for the 'CHEW' FourCC. The file SHOULD NOT contain anything after
/// it. As the size of any chunk is even, the size given by the RIFF header is
/// also even. The contents of individual chunks will be described in the
/// following sections.
///
/// ## File Layout
///
/// ```text
///  0                   1                   2                   3
///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |              Dictionary file header (12 bytes)                |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |            Dictionary format chunk                            |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                         Info chunk                            |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                        Index chunk                            |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                      Phrases chunk                            |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
///
/// ### Dictionary format chunk:
///
/// ```text
///  0                   1                   2                   3
///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                      ChunkHeader('fmt ')                      |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                    Dictionary format version                  |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
///
/// The dictionary format chunk contains the format version number of the
/// index chunk and the phrases chunk, encoded in an unsigned 32 bits
/// integer (u32).
///
/// The currently supported versions are: 0
///
/// ### Info chunk:
///
/// ```text
///  0                   1                   2                   3
///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                      ChunkHeader('LIST')                      |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |      'I'      |      'N'      |      'F'      |      'O'      |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                         Sub chunk header                      |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                         Sub chunk payload                     |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                              ....                             |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
///
/// The `LIST` chunk and the list type `INFO` is a standard RIFF chunk. The list
/// contains information about the copyright, author, engineer of the file, and
/// other similar text. Each sub-chunk's data is a null-terminated string.
///
/// Info data chunks recognized by this library:
///
/// - **INAM**: The name of the file
/// - **ICOP**: Copyright information about the file
/// - **ILIC**: License information about the file
/// - **IREV**: The version of the file
/// - **ISFT**: The name of the software package used to create the file
///
/// ### Index chunk:
///
/// ```text
///  0                   1                   2                   3
///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                      ChunkHeader('DICT')                      |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                          Child Begin                          |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |           Child Len           |            Reserved           |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                          Child Begin                          |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |           Child Len           |         SyllableU16           |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ...                                                           ...
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                           Data Begin                          |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |            Data Len           |            Reserved           |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                           Data Begin                          |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |            Data Len           |            Reserved           |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ...                                                           ...
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                             CRC32                             |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
///
/// The index chunk contains the trie node records serialized in BFS order. The
/// first record is the root node, followed by the nodes in the first layer,
/// followed by the nodes in the second layer, and so on.
///
/// Each node record has fixed size. There are two kinds of nodes.
///
/// Internal node:
///
/// ```text
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |         Child Begin             | Child Len    |SyllableU16 |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
///
/// - **Child Begin: 32 bits (u32)**
///     - The record index of the first child node.
/// - **Child Len: 16 bits (u16)**
///     - The number of the child nodes.
/// - **SyllableU16: 16 bits (u16)**
///     - The [`Syllable`] encoded as an u16 integer.
///
/// Leaf node:
///
/// ```text
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |          Data Begin             |  Data Len    | Reserved   |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
///
/// - **Data Begin: 32 bits (u32)**
///     - The offset into the Phrases chunk for the phrase data.
/// - **Data Len: 16 bits (u16)**
///     - The length to the end of the phrase data, exclusive.
/// - **Reserved: 16 bits (u16)**
///     - Must be all zeros, indicating a leaf node.
///
/// ### Phrases chunk:
///
/// ```text
///  0                   1                   2                   3
///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                      ChunkHeader('DATA')                      |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                           Frequency                           |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |      Length     |                 Phrase                      |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                           Frequency                           |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |      Length     |                                           ...
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                             CRC32                             |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
///
/// The phrases chunk contains all the phrases strings and their frequency.
/// Each phrase is written as length prefixed strings.
///
/// - **Frequency: 32 bits (u32)**
///     - The frequency of the phrase. Might be unaligned.
/// - **Length: 8 bits (u8)**
///     - Each phrase encoded in UTF-8 must not exceed 255 bytes long.
/// - **Phrase: variable bits**
///     - UTF-8 encoded string, not null-terminated.
///
/// [WebP]: https://developers.google.com/speed/webp/docs/riff_container
/// [Trie]: https://en.m.wikipedia.org/wiki/Trie
/// [RIFF]: https://en.m.wikipedia.org/wiki/Resource_Interchange_File_Format
#[derive(Debug)]
pub struct TrieDictionaryBuilder {
    // The builder uses an arena to allocate nodes and reference each node with
    // node index.
    arena: Vec<TrieBuilderNode>,
    info: DictionaryInfo,
}

#[derive(Debug, PartialEq, Default)]
struct TrieBuilderNode {
    id: usize,
    syllable: Option<Syllable>,
    children: Vec<usize>,
    leaf_id: Option<NonZeroUsize>,
    phrases: Vec<Phrase>,
}

/// A container for trie dictionary statistics.
#[derive(Debug)]
pub struct TrieDictionaryStatistics {
    /// The number of nodes in the dictionary.
    pub node_count: usize,
    /// The number of leaf nodes (phrases with same syllables).
    pub leaf_count: usize,
    /// The number of phrases.
    pub phrase_count: usize,
    /// The max height (longest phrase) of the trie.
    pub max_height: usize,
    /// The average height (average phrase length) of the trie.
    pub avg_height: usize,
    /// The max branch count (shared prefix) including the root of the trie.
    pub root_branch_count: usize,
    /// The max branch count (shared prefix) excluding the root of the trie.
    pub max_branch_count: usize,
    /// The average branch count of the trie.
    pub avg_branch_count: usize,
}

impl TrieDictionaryBuilder {
    /// Creates a new `TrieDictionaryBuilder`.
    ///
    /// The builder is initialized with a empty root node. Use the [`insert`][Self::insert]
    /// method to add more entries to the dictionary.
    ///
    /// # Examples
    ///
    /// ```
    /// use chewing::dictionary::TrieDictionaryBuilder;
    ///
    /// let mut builder = TrieDictionaryBuilder::new();
    /// ```
    pub fn new() -> TrieDictionaryBuilder {
        let root = TrieBuilderNode::default();
        TrieDictionaryBuilder {
            arena: vec![root],
            info: Default::default(),
        }
    }

    /// Allocates a new leaf node and returns the new node id.
    fn alloc_leaf(&mut self) -> usize {
        let next_id = self.arena.len();
        let leaf = TrieBuilderNode {
            id: next_id,
            ..Default::default()
        };
        self.arena.push(leaf);
        next_id
    }

    /// Allocates a new internal node and returns the new node id.
    fn alloc_internal(&mut self, syl: Syllable) -> usize {
        let next_id = self.arena.len();
        let internal = TrieBuilderNode {
            id: next_id,
            syllable: Some(syl),
            ..Default::default()
        };
        self.arena.push(internal);
        next_id
    }

    /// Iterates through the syllables and insert all missing internal nodes.
    ///
    /// Returns the id to the leaf node so that we can append the phrase to it.
    fn find_or_insert_internal(&mut self, syllables: &[Syllable]) -> usize {
        let mut node_id = 0;
        'next: for &syl in syllables {
            for &child_node_id in &self.arena[node_id].children {
                if self.arena[child_node_id].syllable == Some(syl) {
                    node_id = child_node_id;
                    continue 'next;
                }
            }
            // We didn't find the child node so insert a new one
            let next_id = self.alloc_internal(syl);
            self.arena[node_id].children.push(next_id);
            node_id = next_id;
        }
        if let Some(leaf_id) = self.arena[node_id].leaf_id {
            node_id = leaf_id.get();
        } else {
            let leaf_id = self.alloc_leaf();
            self.arena[node_id].leaf_id = NonZeroUsize::new(leaf_id);
            node_id = leaf_id;
        }
        node_id
    }

    /// Writes the dictionary to an output stream and returns the number of
    /// bytes written.
    ///
    /// This method creates the sub-chunks containing the index, phrases, and
    /// metadata, then wraps them in a RIFF container.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use std::io::Cursor;
    ///
    /// use chewing::dictionary::TrieDictionaryBuilder;
    ///
    /// let mut writer = Cursor::new(vec![]);
    /// let mut builder = TrieDictionaryBuilder::new();
    ///
    /// builder.write(&mut writer)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write<T>(&self, mut writer: T) -> io::Result<u64>
    where
        T: Write + Seek,
    {
        const ROOT_ID: usize = 0;
        let mut dict_buf = Vec::new();
        let mut data_buf = Vec::new();
        let mut queue = VecDeque::new();

        // The root node's child index starts from 1 (0 is the root).
        let mut child_begin = 1;

        // Walk the tree in BFS order and write the nodes to the dict buffer.
        queue.push_back(ROOT_ID);
        while !queue.is_empty() {
            // Insert nodes layer by layer.
            let layer_nodes_count = queue.len();
            for _ in 0..layer_nodes_count {
                let id = queue.pop_front().unwrap();
                let node = &self.arena[id];

                // An internal node has an associated syllable. The root node is
                // a special case with no syllable.
                if node.syllable.is_some() || id == ROOT_ID {
                    let syllable_u16 = node.syllable.map_or(0, |v| v.to_u16());
                    let child_len =
                        node.children.len() + if node.leaf_id.is_some() { 1 } else { 0 };
                    dict_buf.write_all(&(child_begin as u32).to_le_bytes())?;
                    dict_buf.write_all(&(child_len as u16).to_le_bytes())?;
                    dict_buf.write_all(&syllable_u16.to_le_bytes())?;
                } else {
                    let data_begin = data_buf.len();

                    let mut phrases = node.phrases.clone();
                    phrases.sort_by(|a, b| {
                        // Don't sort single word leaves.
                        // But sort phrases first by the frequency, then by the UTF-8 order.
                        //
                        // NB: this must use a stable sorting algorithm so that lookup
                        // results are stable according to the input file.
                        match (a.as_str().chars().count(), b.as_str().chars().count()) {
                            (1, 1) => Ordering::Equal,
                            (1, _) | (_, 1) => a.as_str().len().cmp(&b.as_str().len()),
                            _ => {
                                if a.freq() == b.freq() {
                                    b.as_str().cmp(a.as_str())
                                } else {
                                    b.freq().cmp(&a.freq())
                                }
                            }
                        }
                    });
                    for phrase in phrases {
                        debug_assert!(phrase.as_str().len() <= u8::MAX as usize);
                        data_buf.write_all(&phrase.freq().to_le_bytes())?;
                        data_buf.write_all(&[phrase.as_str().len() as u8])?;
                        data_buf.write_all(phrase.as_str().as_bytes())?;
                    }

                    let data_len = data_buf.len() - data_begin;
                    dict_buf.write_all(&(data_begin as u32).to_le_bytes())?;
                    dict_buf.write_all(&(data_len as u16).to_le_bytes())?;
                    dict_buf.write_all(&0u16.to_le_bytes())?;
                }

                // Sort the children nodes by their syllables. Not really required,
                // but it makes using binary search possible in the future.
                let mut children = node.children.clone();
                children.sort_by(|&a, &b| self.arena[a].syllable.cmp(&self.arena[b].syllable));
                if let Some(leaf_id) = node.leaf_id {
                    child_begin += 1;
                    queue.push_back(leaf_id.get());
                }
                for child_id in children {
                    child_begin += 1;
                    queue.push_back(child_id);
                }
            }
        }

        let crc32 = Crc32::new();
        let check = crc32.check(&dict_buf);
        dict_buf.write_all(&check.to_le_bytes())?;
        let check = crc32.check(&data_buf);
        data_buf.write_all(&check.to_le_bytes())?;

        // Wrap the data in a RIFF container
        let contents = ChunkContents::Children(
            RIFF_ID,
            CHEW,
            vec![
                ChunkContents::Data(FMT, DICT_FORMAT.to_le_bytes().into()),
                ChunkContents::Children(LIST, INFO, self.info_chunks()?),
                ChunkContents::Data(DICT, dict_buf),
                ChunkContents::Data(DATA, data_buf),
            ],
        );

        contents.write(&mut writer)
    }

    fn info_chunks(&self) -> Result<Vec<ChunkContents>, io::Error> {
        let mut info_chunks = vec![];
        if let Some(name) = &self.info.name {
            info_chunks.push(ChunkContents::Data(
                INAM,
                CString::new(name.as_bytes())?.into_bytes(),
            ))
        }
        if let Some(copyright) = &self.info.copyright {
            info_chunks.push(ChunkContents::Data(
                ICOP,
                CString::new(copyright.as_bytes())?.into_bytes(),
            ))
        }
        if let Some(license) = &self.info.license {
            info_chunks.push(ChunkContents::Data(
                ILIC,
                CString::new(license.as_bytes())?.into_bytes(),
            ))
        }
        if let Some(version) = &self.info.version {
            info_chunks.push(ChunkContents::Data(
                IREV,
                CString::new(version.as_bytes())?.into_bytes(),
            ))
        }
        if let Some(software) = &self.info.software {
            info_chunks.push(ChunkContents::Data(
                ISFT,
                CString::new(software.as_bytes())?.into_bytes(),
            ))
        }
        Ok(info_chunks)
    }

    /// Calculates the statistics of the dictionary.
    ///
    /// # Examples
    ///
    /// ```
    /// use chewing::{syl, zhuyin::Bopomofo};
    /// use chewing::dictionary::{DictionaryBuilder, TrieDictionaryBuilder};
    ///
    /// let mut builder = TrieDictionaryBuilder::new();
    /// builder.insert(&[
    ///     syl![Bopomofo::G, Bopomofo::U, Bopomofo::O, Bopomofo::TONE2],
    /// ], ("國", 0).into());
    /// builder.insert(&[
    ///     syl![Bopomofo::M, Bopomofo::I, Bopomofo::EN, Bopomofo::TONE2]
    /// ], ("民", 0).into());
    /// builder.insert(&[
    ///     syl![Bopomofo::D, Bopomofo::A, Bopomofo::TONE4],
    /// ], ("大", 0).into());
    /// builder.insert(&[
    ///     syl![Bopomofo::H, Bopomofo::U, Bopomofo::EI, Bopomofo::TONE4],
    /// ], ("會", 0).into());
    /// builder.insert(&[
    ///     syl![Bopomofo::G, Bopomofo::U, Bopomofo::O, Bopomofo::TONE2],
    ///     syl![Bopomofo::M, Bopomofo::I, Bopomofo::EN, Bopomofo::TONE2]
    /// ], ("國民", 0).into());
    /// builder.insert(&[
    ///     syl![Bopomofo::G, Bopomofo::U, Bopomofo::O, Bopomofo::TONE2],
    ///     syl![Bopomofo::M, Bopomofo::I, Bopomofo::EN, Bopomofo::TONE2],
    ///     syl![Bopomofo::D, Bopomofo::A, Bopomofo::TONE4],
    ///     syl![Bopomofo::H, Bopomofo::U, Bopomofo::EI, Bopomofo::TONE4],
    /// ], ("國民大會", 0).into());
    /// let stats = builder.statistics();
    /// assert_eq!(14, stats.node_count);
    /// assert_eq!(6, stats.leaf_count);
    /// assert_eq!(6, stats.phrase_count);
    /// assert_eq!(6, stats.max_height);
    /// assert_eq!(2, stats.avg_height);
    /// assert_eq!(4, stats.root_branch_count);
    /// assert_eq!(1, stats.max_branch_count);
    /// assert_eq!(0, stats.avg_branch_count);
    /// ```
    pub fn statistics(&self) -> TrieDictionaryStatistics {
        let mut node_count = 0;
        let mut leaf_count = 0;
        let mut phrase_count = 0;
        let mut max_height = 0;
        let mut branch_heights = vec![];
        let mut root_branch_count = 0;
        let mut max_branch_count = 0;
        let mut branch_counts = vec![];

        const ROOT_ID: usize = 0;
        let mut queue = VecDeque::new();
        queue.push_back(ROOT_ID);

        while !queue.is_empty() {
            let layer_nodes_count = queue.len();

            max_height += 1;
            for _ in 0..layer_nodes_count {
                let id = queue.pop_front().unwrap();
                let node = &self.arena[id];

                node_count += 1;

                if node.syllable.is_some() || id == ROOT_ID {
                    if node.leaf_id.is_some() {
                        leaf_count += 1;
                        branch_heights.push(max_height);
                    }
                    let branch_count = node
                        .children
                        .iter()
                        .filter(|&&child_id| self.arena[child_id].syllable.is_some())
                        .count();
                    branch_counts.push(branch_count);
                    if branch_count > max_branch_count && id != ROOT_ID {
                        max_branch_count = node.children.len();
                    }
                    if branch_count > root_branch_count {
                        root_branch_count = node.children.len();
                    }
                } else {
                    phrase_count += node.phrases.len();
                }
                if let Some(leaf_id) = node.leaf_id {
                    queue.push_back(leaf_id.get());
                }
                for &child_id in &node.children {
                    queue.push_back(child_id);
                }
            }
        }

        TrieDictionaryStatistics {
            node_count,
            leaf_count,
            phrase_count,
            max_height,
            avg_height: branch_heights.iter().sum::<usize>() / branch_counts.len(),
            root_branch_count,
            max_branch_count,
            avg_branch_count: branch_counts.iter().sum::<usize>() / branch_counts.len(),
        }
    }
}

impl DictionaryBuilder for TrieDictionaryBuilder {
    fn set_info(&mut self, info: DictionaryInfo) -> Result<(), BuildDictionaryError> {
        self.info = info;
        Ok(())
    }

    /// Inserts a new entry to the dictionary.
    ///
    /// A DuplicatePhraseError is returned if a phrase is already present with
    /// the same syllables.
    ///
    /// # Examples
    ///
    /// ```
    /// use chewing::{syl, zhuyin::Bopomofo};
    /// use chewing::dictionary::{DictionaryBuilder, TrieDictionaryBuilder};
    ///
    /// let mut builder = TrieDictionaryBuilder::new();
    ///
    /// builder.insert(&[
    ///     syl![Bopomofo::Z, Bopomofo::TONE4],
    ///     syl![Bopomofo::D, Bopomofo::I, Bopomofo::AN]
    /// ], ("字典", 0).into());
    /// ```
    fn insert(
        &mut self,
        syllables: &[Syllable],
        phrase: Phrase,
    ) -> Result<(), BuildDictionaryError> {
        let leaf_id = self.find_or_insert_internal(syllables);
        if self.arena[leaf_id]
            .phrases
            .iter()
            .any(|it| it.as_str() == phrase.as_str())
        {
            return Err(BuildDictionaryError {
                source: Box::new(DuplicatePhraseError),
            });
        }
        self.arena[leaf_id].phrases.push(phrase);
        Ok(())
    }

    fn build(&mut self, path: &Path) -> Result<(), BuildDictionaryError> {
        let database = File::create(path)?;
        let mut writer = BufWriter::new(database);
        self.write(&mut writer)?;
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Default for TrieDictionaryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculates CRC32 with CRC-32-IEEE 802.3 poly
struct Crc32 {
    table: Box<[[u32; 256]; 8]>,
}

impl Crc32 {
    fn new() -> Crc32 {
        let mut table = Box::new([[0u32; 256]; 8]);
        // CRC-32-IEEE 802.3 poly
        let poly: u32 = 0xEDB88320;
        for i in 0..256usize {
            let mut crc = i as u32;
            for _ in 0..8 {
                crc = (crc >> 1) ^ ((crc & 1) * poly);
            }
            table[0][i] = crc;
        }
        for i in 0..256usize {
            table[1][i] = (table[0][i] >> 8) ^ table[0][(table[0][i] & 0xFF) as usize];
            table[2][i] = (table[1][i] >> 8) ^ table[0][(table[1][i] & 0xFF) as usize];
            table[3][i] = (table[2][i] >> 8) ^ table[0][(table[2][i] & 0xFF) as usize];
            table[4][i] = (table[3][i] >> 8) ^ table[0][(table[3][i] & 0xFF) as usize];
            table[5][i] = (table[4][i] >> 8) ^ table[0][(table[4][i] & 0xFF) as usize];
            table[6][i] = (table[5][i] >> 8) ^ table[0][(table[5][i] & 0xFF) as usize];
            table[7][i] = (table[6][i] >> 8) ^ table[0][(table[6][i] & 0xFF) as usize];
        }
        Crc32 { table }
    }
    fn check(&self, buf: &[u8]) -> u32 {
        // Slice-by-8 algorithm
        !buf.chunks(8).fold(!0u32, |crc, bytes| {
            if bytes.len() == 8 {
                let one = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) ^ crc;
                let two = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
                self.table[0][(two >> 24 & 0xFF) as usize]
                    ^ self.table[1][(two >> 16 & 0xFF) as usize]
                    ^ self.table[2][(two >> 8 & 0xFF) as usize]
                    ^ self.table[3][(two & 0xFF) as usize]
                    ^ self.table[4][(one >> 24 & 0xFF) as usize]
                    ^ self.table[5][(one >> 16 & 0xFF) as usize]
                    ^ self.table[6][(one >> 8 & 0xFF) as usize]
                    ^ self.table[7][(one & 0xFF) as usize]
            } else {
                bytes.iter().fold(crc, |crc, byte| {
                    (crc >> 8) ^ self.table[0][((crc & 0xFF) ^ *byte as u32) as usize]
                })
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Cursor, num::NonZeroUsize};

    use crate::{
        dictionary::{
            trie::TrieBuilderNode, Dictionary, DictionaryBuilder, DictionaryInfo, Phrase,
        },
        syl,
        zhuyin::Bopomofo,
    };

    use super::{Crc32, TrieDictionary, TrieDictionaryBuilder};

    #[test]
    fn test_tree_construction() -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = TrieDictionaryBuilder::new();
        builder.insert(
            &vec![
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
            ],
            ("測試", 100).into(),
        )?;
        builder.insert(
            &vec![
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::S, Bopomofo::U, Bopomofo::O, Bopomofo::TONE3],
            ],
            ("廁所", 100).into(),
        )?;
        assert_eq!(
            vec![
                TrieBuilderNode {
                    id: 0,
                    syllable: None,
                    children: vec![1],
                    leaf_id: None,
                    phrases: vec![]
                },
                TrieBuilderNode {
                    id: 1,
                    syllable: Some(syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]),
                    children: vec![2, 4],
                    leaf_id: None,
                    phrases: vec![]
                },
                TrieBuilderNode {
                    id: 2,
                    syllable: Some(syl![Bopomofo::SH, Bopomofo::TONE4]),
                    children: vec![],
                    leaf_id: NonZeroUsize::new(3),
                    phrases: vec![]
                },
                TrieBuilderNode {
                    id: 3,
                    syllable: None,
                    children: vec![],
                    leaf_id: None,
                    phrases: vec![("測試", 100).into()]
                },
                TrieBuilderNode {
                    id: 4,
                    syllable: Some(syl![Bopomofo::S, Bopomofo::U, Bopomofo::O, Bopomofo::TONE3]),
                    children: vec![],
                    leaf_id: NonZeroUsize::new(5),
                    phrases: vec![]
                },
                TrieBuilderNode {
                    id: 5,
                    syllable: None,
                    children: vec![],
                    leaf_id: None,
                    phrases: vec![("廁所", 100).into()]
                }
            ],
            builder.arena
        );
        Ok(())
    }

    #[test]
    fn tree_lookup_word() -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = TrieDictionaryBuilder::new();
        builder.insert(
            &vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            ("測", 1).into(),
        )?;
        builder.insert(
            &vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            ("冊", 1).into(),
        )?;
        let mut cursor = Cursor::new(vec![]);
        builder.write(&mut cursor)?;

        let dict = TrieDictionary::new(&mut cursor)?;
        assert_eq!(
            vec![Phrase::new("測", 1), Phrase::new("冊", 1)],
            dict.lookup_all_phrases(&[syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]])
        );

        Ok(())
    }

    #[test]
    fn tree_lookup_phrase() -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = TrieDictionaryBuilder::new();
        builder.insert(
            &vec![
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
            ],
            ("測試", 1).into(),
        )?;
        builder.insert(
            &vec![
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
            ],
            ("策試", 2).into(),
        )?;
        builder.insert(
            &vec![
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
                syl![Bopomofo::CH, Bopomofo::ENG, Bopomofo::TONE2],
                syl![Bopomofo::G, Bopomofo::U, Bopomofo::ENG],
            ],
            ("測試成功", 3).into(),
        )?;
        let mut cursor = Cursor::new(vec![]);
        builder.write(&mut cursor)?;

        let dict = TrieDictionary::new(&mut cursor)?;
        assert_eq!(
            vec![Phrase::new("策試", 2), Phrase::new("測試", 1)],
            dict.lookup_all_phrases(&[
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4]
            ])
        );
        assert_eq!(
            vec![Phrase::new("測試成功", 3)],
            dict.lookup_all_phrases(&[
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
                syl![Bopomofo::CH, Bopomofo::ENG, Bopomofo::TONE2],
                syl![Bopomofo::G, Bopomofo::U, Bopomofo::ENG],
            ])
        );
        assert_eq!(
            Vec::<Phrase>::new(),
            dict.lookup_all_phrases(&[
                syl![Bopomofo::C, Bopomofo::U, Bopomofo::O, Bopomofo::TONE4],
                syl![Bopomofo::U, Bopomofo::TONE4]
            ])
        );

        Ok(())
    }

    #[test]
    #[should_panic]
    fn tree_builder_duplicate_phrase_error() {
        let mut builder = TrieDictionaryBuilder::new();
        builder
            .insert(
                &vec![
                    syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                    syl![Bopomofo::SH, Bopomofo::TONE4],
                ],
                ("測試", 1).into(),
            )
            .expect("Duplicate phrase error");
        builder
            .insert(
                &vec![
                    syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                    syl![Bopomofo::SH, Bopomofo::TONE4],
                ],
                ("測試", 2).into(),
            )
            .expect("Duplicate phrase error");
    }

    #[test]
    fn stable_word_sort_order() -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = TrieDictionaryBuilder::new();
        for word in ["冊", "策", "測", "側"] {
            builder.insert(
                &vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
                (word, 0).into(),
            )?;
        }
        let mut cursor = Cursor::new(vec![]);
        builder.write(&mut cursor)?;

        let dict = TrieDictionary::new(&mut cursor)?;
        assert_eq!(
            vec![
                Phrase::new("冊", 0),
                Phrase::new("策", 0),
                Phrase::new("測", 0),
                Phrase::new("側", 0),
            ],
            dict.lookup_all_phrases(&vec![syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],])
        );
        Ok(())
    }

    #[test]
    fn stable_phrase_sort_order() -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = TrieDictionaryBuilder::new();
        builder.insert(
            &vec![
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
            ],
            ("側室", 318).into(),
        )?;
        builder.insert(
            &vec![
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
            ],
            ("側視", 318).into(),
        )?;
        builder.insert(
            &vec![
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
            ],
            ("策士", 318).into(),
        )?;
        builder.insert(
            &vec![
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
            ],
            ("策試", 318).into(),
        )?;
        builder.insert(
            &vec![
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
            ],
            ("測試", 9318).into(),
        )?;
        let mut cursor = Cursor::new(vec![]);
        builder.write(&mut cursor)?;

        let dict = TrieDictionary::new(&mut cursor)?;
        assert_eq!(
            vec![
                Phrase::new("測試", 9318),
                Phrase::new("策試", 318),
                Phrase::new("策士", 318),
                Phrase::new("側視", 318),
                Phrase::new("側室", 318),
            ],
            dict.lookup_all_phrases(&vec![
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
            ])
        );
        Ok(())
    }

    #[test]
    fn tree_builder_write_read_metadata() {
        let mut builder = TrieDictionaryBuilder::new();
        let info = DictionaryInfo {
            name: Some("name".into()),
            copyright: Some("copyright".into()),
            license: Some("license".into()),
            version: Some("version".into()),
            software: Some("software".into()),
        };
        builder.set_info(info).unwrap();

        let mut cursor = Cursor::new(vec![]);
        builder.write(&mut cursor).unwrap();

        let dict = TrieDictionary::new(&mut cursor).unwrap();
        let info = dict.about();

        assert_eq!("name", info.name.unwrap());
        assert_eq!("copyright", info.copyright.unwrap());
        assert_eq!("license", info.license.unwrap());
        assert_eq!("version", info.version.unwrap());
        assert_eq!("software", info.software.unwrap());
    }

    #[test]
    fn tree_entries() -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = TrieDictionaryBuilder::new();
        builder.insert(
            &vec![
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
            ],
            ("測試", 1).into(),
        )?;
        builder.insert(
            &vec![
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
            ],
            ("策試", 2).into(),
        )?;
        builder.insert(
            &vec![
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
                syl![Bopomofo::CH, Bopomofo::ENG, Bopomofo::TONE2],
                syl![Bopomofo::G, Bopomofo::U, Bopomofo::ENG],
            ],
            ("測試成功", 3).into(),
        )?;
        builder.insert(
            &vec![
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
                syl![Bopomofo::SH],
                syl![Bopomofo::B, Bopomofo::AI, Bopomofo::TONE4],
            ],
            ("測試失敗", 3).into(),
        )?;
        let mut cursor = Cursor::new(vec![]);
        builder.write(&mut cursor)?;

        let dict = TrieDictionary::new(&mut cursor)?;
        assert_eq!(
            vec![
                (
                    vec![
                        syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                        syl![Bopomofo::SH, Bopomofo::TONE4],
                        syl![Bopomofo::CH, Bopomofo::ENG, Bopomofo::TONE2],
                        syl![Bopomofo::G, Bopomofo::U, Bopomofo::ENG],
                    ],
                    Phrase::new("測試成功", 3)
                ),
                (
                    vec![
                        syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                        syl![Bopomofo::SH, Bopomofo::TONE4]
                    ],
                    Phrase::new("策試", 2)
                ),
                (
                    vec![
                        syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                        syl![Bopomofo::SH, Bopomofo::TONE4]
                    ],
                    Phrase::new("測試", 1)
                ),
                (
                    vec![
                        syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                        syl![Bopomofo::SH, Bopomofo::TONE4],
                        syl![Bopomofo::SH],
                        syl![Bopomofo::B, Bopomofo::AI, Bopomofo::TONE4],
                    ],
                    Phrase::new("測試失敗", 3)
                ),
            ],
            dict.entries().collect::<Vec<_>>()
        );
        Ok(())
    }

    #[test]
    fn crc32() {
        let crc32 = Crc32::new();
        assert_eq!(0x152ddece, crc32.check(b"asd\n"));
        assert_eq!(
            0x414fa339,
            crc32.check(b"The quick brown fox jumps over the lazy dog")
        );
    }
}
