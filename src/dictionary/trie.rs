use std::{
    any::Any,
    cmp::Ordering,
    collections::VecDeque,
    error::Error,
    fmt::Debug,
    fs::{self, File},
    io::{self, BufWriter, Read, Write},
    iter,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    time::SystemTime,
};

use der::{
    DecodeValue, Document, Encode, EncodeValue, ErrorKind, FixedTag, Length, Reader, Sequence,
    SliceReader, Tag, TagMode, TagNumber, Tagged, Writer,
    asn1::{ContextSpecificRef, OctetStringRef, Utf8StringRef},
};
use log::{error, warn};

use crate::zhuyin::{Syllable, SyllableSlice};

use super::{
    BuildDictionaryError, Dictionary, DictionaryBuilder, DictionaryInfo, Entries, LookupStrategy,
    Phrase,
};

const DICT_FORMAT_VERSION: u8 = 0;

struct TrieNodeView<'a>(&'a [u8]);

impl TrieNodeView<'_> {
    const SIZE: usize = 8;
    fn syllable(&self) -> u16 {
        u16::from_be_bytes(self.0[6..8].try_into().unwrap())
    }
    fn child_begin(&self) -> usize {
        u32::from_be_bytes(self.0[..4].try_into().unwrap()) as usize * Self::SIZE
    }
    fn child_end(&self) -> usize {
        (u32::from_be_bytes(self.0[..4].try_into().unwrap()) as usize)
            .saturating_add(u16::from_be_bytes(self.0[4..6].try_into().unwrap()) as usize)
            * Self::SIZE
    }
}

struct TrieLeafView<'a>(&'a [u8]);

impl TrieLeafView<'_> {
    const SIZE: usize = 8;
    fn reserved_zero(&self) -> u16 {
        u16::from_be_bytes(self.0[6..8].try_into().unwrap())
    }
    fn data_begin(&self) -> usize {
        u32::from_be_bytes(self.0[..4].try_into().unwrap()) as usize
    }
    fn data_end(&self) -> usize {
        (u32::from_be_bytes(self.0[..4].try_into().unwrap()) as usize)
            .saturating_add(u16::from_be_bytes(self.0[4..6].try_into().unwrap()) as usize)
    }
}

/// A read-only dictionary using a pre-built [Trie][] index that is both space
/// efficient and fast to lookup.
///
/// `Trie`s can be used as system dictionaries or shared dictionaries.
/// The file format is defined using the platform independent [DER][DER]
/// encoding format, allowing them to be versioned and shared easily.
///
/// A new dictionary can be built using a [`TrieBuilder`].
///
/// # Examples
///
/// Read a dictionary from a [File][`std::fs::File`]:
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let tmpdir = tempfile::tempdir()?;
/// # std::env::set_current_dir(&tmpdir.path())?;
/// use std::fs::File;
///
/// use chewing::{syl, zhuyin::{Bopomofo, Syllable}};
/// # use chewing::dictionary::{DictionaryBuilder, TrieBuilder};
/// use chewing::dictionary::{Dictionary, LookupStrategy, Trie};
/// # let mut tempfile = File::create("dict.dat")?;
/// # let mut builder = TrieBuilder::new();
/// # builder.insert(&[
/// #     syl![Bopomofo::Z, Bopomofo::TONE4],
/// #     syl![Bopomofo::D, Bopomofo::I, Bopomofo::AN, Bopomofo::TONE3]
/// # ], ("字典", 0).into());
/// # builder.write(&mut tempfile)?;
///
/// let mut file = File::open("dict.dat")?;
/// let dict = Trie::new(&mut file)?;
///
/// // Find the phrase ㄗˋㄉ一ㄢˇ (dictionary)
/// let mut phrase = dict.lookup_first_phrase(&[
///     syl![Bopomofo::Z, Bopomofo::TONE4],
///     syl![Bopomofo::D, Bopomofo::I, Bopomofo::AN, Bopomofo::TONE3]
/// ], LookupStrategy::Standard);
/// assert_eq!("字典", phrase.unwrap().as_str());
/// # Ok(())
/// # }
/// ```
///
/// [Trie]: https://en.m.wikipedia.org/wiki/Trie
/// [DER]: https://en.m.wikipedia.org/wiki/X.690#DER_encoding
#[derive(Debug, Clone)]
pub struct Trie {
    info: DictionaryInfo,
    path: Option<PathBuf>,
    index: Box<[u8]>,
    phrase_seq: Box<[u8]>,

    fuzzy_search: bool,
}

fn io_error(e: impl Into<Box<dyn Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e)
}

impl Trie {
    /// Creates a new `Trie` instance from a file.
    ///
    /// The data in the file must conform to the dictionary format spec.
    ///
    /// See [`TrieBuilder`] on how to build a dictionary.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use chewing::dictionary::Trie;
    ///
    /// let dict = Trie::open("dict.dat")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Trie> {
        TrieOpenOptions::new().open(path)
    }
    /// Creates a new `Trie` instance from a input stream.
    ///
    /// The underlying data of the input stream must conform to the dictionary
    /// format spec.
    ///
    /// See [`TrieBuilder`] on how to build a dictionary.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use std::fs::File;
    ///
    /// use chewing::dictionary::Trie;
    ///
    /// let mut file = File::open("dict.dat")?;
    /// let dict = Trie::new(&mut file)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new<T>(stream: T) -> io::Result<Trie>
    where
        T: Read,
    {
        TrieOpenOptions::new().read_from(stream)
    }
    /// Enable or disable fuzzy search.
    pub fn enable_fuzzy_search(&mut self, fuzzy_search: bool) {
        self.fuzzy_search = fuzzy_search;
    }
}

/// Options and flags which can be used to configure how a trie dictionary is
/// opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TrieOpenOptions {
    fuzzy_search: bool,
}

impl TrieOpenOptions {
    pub fn new() -> TrieOpenOptions {
        TrieOpenOptions::default()
    }
    pub fn fuzzy_search(&mut self, fuzzy_search: bool) -> &mut Self {
        self.fuzzy_search = fuzzy_search;
        self
    }
    pub fn open<P: AsRef<Path>>(&self, path: P) -> io::Result<Trie> {
        let path = path.as_ref().to_path_buf();
        let mut file = File::open(&path)?;
        let mut trie = self.read_from(&mut file)?;
        trie.path = Some(path);
        Ok(trie)
    }
    pub fn read_from<T>(&self, mut stream: T) -> io::Result<Trie>
    where
        T: Read,
    {
        let mut buf = vec![];
        stream.read_to_end(&mut buf)?;
        let trie_dict_doc = Document::try_from(buf).map_err(io_error)?;
        let trie_ref: TrieFileRef<'_> = trie_dict_doc.decode_msg().map_err(io_error)?;
        let info = trie_ref.info.into();
        let index = trie_ref.index.as_bytes().into();
        let phrase_seq = trie_ref.phrase_seq.der_bytes.into();
        Ok(Trie {
            info,
            path: None,
            index,
            phrase_seq,
            fuzzy_search: self.fuzzy_search,
        })
    }
}

struct PhrasesIter<'a> {
    reader: SliceReader<'a>,
}

impl PhrasesIter<'_> {
    fn new(bytes: &[u8]) -> PhrasesIter<'_> {
        PhrasesIter {
            reader: SliceReader::new(bytes).unwrap(),
        }
    }
}

impl Iterator for PhrasesIter<'_> {
    type Item = Phrase;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.reader.is_finished() {
            return None;
        }
        self.reader.decode().ok()
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

impl Dictionary for Trie {
    fn lookup_first_n_phrases(
        &self,
        syllables: &dyn SyllableSlice,
        first: usize,
        strategy: LookupStrategy,
    ) -> Vec<Phrase> {
        let dict = self.index.as_ref();
        let data = self.phrase_seq.as_ref();

        bail_if_oob!(0, TrieNodeView::SIZE, dict.len());
        let root = TrieNodeView(&dict[..TrieNodeView::SIZE]);

        // Return early for empty dictionary
        if root.child_begin() == root.child_end() {
            warn!("[!] detected empty dictionary.");
            return vec![];
        }

        let search_predicate = match strategy {
            LookupStrategy::Standard => |n: u16, syl: &Syllable| n == syl.to_u16(),
            LookupStrategy::FuzzyPartialPrefix => |n: u16, syl: &Syllable| {
                if n == 0 {
                    return false;
                }
                if let Ok(syllable) = Syllable::try_from(n) {
                    syllable.starts_with(*syl)
                } else {
                    false
                }
            },
        };

        // Perform a BFS search to find all leaf nodes
        let mut threads: VecDeque<TrieNodeView<'_>> = VecDeque::new();
        threads.push_back(root);
        for syl in syllables.to_slice().iter() {
            debug_assert!(syl.to_u16() != 0);
            for _ in 0..threads.len() {
                let node = threads.pop_front().unwrap();
                bail_if_oob!(node.child_begin(), node.child_end(), dict.len());
                let child_nodes = dict[node.child_begin()..node.child_end()]
                    .chunks_exact(TrieNodeView::SIZE)
                    .map(TrieNodeView);
                for n in child_nodes {
                    if search_predicate(n.syllable(), syl) {
                        threads.push_back(n);
                    }
                }
            }
            if threads.is_empty() {
                return vec![];
            }
        }

        // Collect result from all threads
        let mut result = vec![];
        for node in threads.into_iter() {
            bail_if_oob!(node.child_begin(), node.child_end(), dict.len());
            let leaf_data = &dict[node.child_begin()..];
            bail_if_oob!(0, TrieLeafView::SIZE, leaf_data.len());
            let leaf = TrieLeafView(&leaf_data[..TrieLeafView::SIZE]);
            if leaf.reserved_zero() != 0 {
                // Skip non leaf nodes
                continue;
            }
            bail_if_oob!(leaf.data_begin(), leaf.data_end(), data.len());
            result.extend(PhrasesIter::new(&data[leaf.data_begin()..leaf.data_end()]));
            if result.len() > first {
                break;
            }
        }
        result
    }

    fn entries(&self) -> Entries<'_> {
        let dict = self.index.as_ref();
        let data = self.phrase_seq.as_ref();
        let mut results = Vec::new();
        let mut stack = Vec::new();
        let mut syllables = Vec::new();
        if dict.len() < TrieNodeView::SIZE {
            error!("[!] file corruption detected: index out of bound.");
            return Box::new(iter::empty());
        }
        let root = TrieNodeView(&dict[..TrieNodeView::SIZE]);
        let mut node = root;
        if node.child_begin() == node.child_end() {
            return Box::new(iter::empty());
        }

        let make_dict_entry =
            |syllables: &[u16], leaf: &TrieLeafView<'_>| -> (Vec<Syllable>, Vec<Phrase>) {
                debug_assert_eq!(leaf.reserved_zero(), 0);
                (
                    syllables
                        .iter()
                        // FIXME - skip invalid entry?
                        .map(|&syl_u16| Syllable::try_from(syl_u16).unwrap())
                        .collect::<Vec<_>>(),
                    PhrasesIter::new(&data[leaf.data_begin()..leaf.data_end()]).collect::<Vec<_>>(),
                )
            };

        let mut done = false;
        let it = iter::from_fn(move || {
            if !results.is_empty() {
                return results.pop();
            }
            if done {
                return None;
            }
            // descend until find a leaf node which is not also a internal node.
            loop {
                iter_bail_if_oob!(node.child_begin(), node.child_end(), dict.len());
                let mut child_iter = dict[node.child_begin()..node.child_end()]
                    .chunks_exact(TrieNodeView::SIZE)
                    .map(TrieNodeView);
                let mut next = child_iter
                    .next()
                    .expect("syllable node should have at least one child node");
                if next.syllable() == 0 {
                    // found a leaf syllable node
                    iter_bail_if_oob!(node.child_begin(), node.child_end(), dict.len());
                    let leaf_data = &dict[node.child_begin()..];
                    let leaf = TrieLeafView(&leaf_data[..TrieLeafView::SIZE]);
                    iter_bail_if_oob!(leaf.data_begin(), leaf.data_end(), data.len());
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

    fn path(&self) -> Option<&Path> {
        self.path.as_ref().map(|p| p as &Path)
    }

    fn as_dict_mut(&mut self) -> Option<&mut dyn super::DictionaryMut> {
        None
    }
}

fn context_specific<T: EncodeValue + Tagged>(
    tag_number: u8,
    value: &T,
) -> ContextSpecificRef<'_, T> {
    ContextSpecificRef {
        tag_number: TagNumber::new(tag_number),
        tag_mode: TagMode::Implicit,
        value,
    }
}

fn context_specific_opt<T: EncodeValue + Tagged>(
    tag_number: u8,
    value: &Option<T>,
) -> Option<ContextSpecificRef<'_, T>> {
    value
        .as_ref()
        .map(|value| context_specific(tag_number, value))
}

struct DictionaryInfoRef<'a> {
    name: Utf8StringRef<'a>,
    copyright: Utf8StringRef<'a>,
    license: Utf8StringRef<'a>,
    version: Utf8StringRef<'a>,
    software: Utf8StringRef<'a>,
}

impl From<DictionaryInfoRef<'_>> for DictionaryInfo {
    fn from(value: DictionaryInfoRef<'_>) -> Self {
        DictionaryInfo {
            name: value.name.into(),
            copyright: value.copyright.into(),
            license: value.license.into(),
            version: value.version.into(),
            software: value.software.into(),
        }
    }
}

impl DictionaryInfoRef<'_> {
    fn new(info: &DictionaryInfo) -> DictionaryInfoRef<'_> {
        DictionaryInfoRef {
            name: Utf8StringRef::new(&info.name).unwrap(),
            copyright: Utf8StringRef::new(&info.copyright).unwrap(),
            license: Utf8StringRef::new(&info.license).unwrap(),
            version: Utf8StringRef::new(&info.version).unwrap(),
            software: Utf8StringRef::new(&info.software).unwrap(),
        }
    }
}

impl FixedTag for DictionaryInfoRef<'_> {
    const TAG: Tag = Tag::Sequence;
}

impl<'a> DecodeValue<'a> for DictionaryInfoRef<'a> {
    fn decode_value<R: Reader<'a>>(reader: &mut R, header: der::Header) -> der::Result<Self> {
        reader.read_nested(header.length, |reader| {
            let name = reader.decode()?;
            let copyright = reader.decode()?;
            let license = reader.decode()?;
            let version = reader.decode()?;
            let software = reader.decode()?;
            Ok(DictionaryInfoRef {
                name,
                copyright,
                license,
                version,
                software,
            })
        })
    }
}

impl EncodeValue for DictionaryInfoRef<'_> {
    fn value_len(&self) -> der::Result<Length> {
        self.name.encoded_len()?
            + self.copyright.encoded_len()?
            + self.license.encoded_len()?
            + self.version.encoded_len()?
            + self.software.encoded_len()?
    }

    fn encode_value(&self, encoder: &mut impl Writer) -> der::Result<()> {
        self.name.encode(encoder)?;
        self.copyright.encode(encoder)?;
        self.license.encode(encoder)?;
        self.version.encode(encoder)?;
        self.software.encode(encoder)?;
        Ok(())
    }
}

struct TrieFileRef<'a> {
    info: DictionaryInfoRef<'a>,
    index: OctetStringRef<'a>,
    phrase_seq: PhraseSeqRef<'a>,
}

struct PhraseSeqRef<'a> {
    der_bytes: &'a [u8],
}

impl<'a> Sequence<'a> for TrieFileRef<'a> {}

impl<'a> DecodeValue<'a> for TrieFileRef<'a> {
    fn decode_value<R: Reader<'a>>(reader: &mut R, header: der::Header) -> der::Result<Self> {
        reader.read_nested(header.length, |reader| {
            let magic: Utf8StringRef<'_> = reader.decode()?;
            let version: u8 = reader.decode()?;
            if magic.as_str() != "CHEW" || version != DICT_FORMAT_VERSION {
                return Err(ErrorKind::Value { tag: header.tag }.at(reader.position()));
            }
            let info = reader.decode()?;
            let index = reader.decode()?;
            let phrase_seq = reader.decode()?;
            Ok(Self {
                info,
                index,
                phrase_seq,
            })
        })
    }
}

impl EncodeValue for TrieFileRef<'_> {
    fn value_len(&self) -> der::Result<Length> {
        Utf8StringRef::new("CHEW")?.encoded_len()?
            + DICT_FORMAT_VERSION.encoded_len()?
            + self.info.encoded_len()?
            + self.index.encoded_len()?
            + self.phrase_seq.encoded_len()?
    }

    fn encode_value(&self, encoder: &mut impl Writer) -> der::Result<()> {
        Utf8StringRef::new("CHEW")?.encode(encoder)?;
        DICT_FORMAT_VERSION.encode(encoder)?;
        self.info.encode(encoder)?;
        self.index.encode(encoder)?;
        self.phrase_seq.encode(encoder)?;
        Ok(())
    }
}

impl FixedTag for Phrase {
    const TAG: Tag = Tag::Sequence;
}

impl<'a> DecodeValue<'a> for Phrase {
    fn decode_value<R: Reader<'a>>(reader: &mut R, header: der::Header) -> der::Result<Self> {
        reader.read_nested(header.length, |reader| {
            let phrase: Utf8StringRef<'_> = reader.decode()?;
            let freq = reader.decode()?;
            let last_used = reader.context_specific(TagNumber::N0, TagMode::Implicit)?;
            Ok(Phrase {
                phrase: String::from(phrase).into_boxed_str(),
                freq,
                last_used,
            })
        })
    }
}

impl EncodeValue for Phrase {
    fn value_len(&self) -> der::Result<Length> {
        Utf8StringRef::new(self.as_str())?.encoded_len()?
            + self.freq.encoded_len()?
            + context_specific_opt(0, &self.last_used).encoded_len()?
    }

    fn encode_value(&self, encoder: &mut impl Writer) -> der::Result<()> {
        Utf8StringRef::new(self.as_ref())?.encode(encoder)?;
        self.freq.encode(encoder)?;
        context_specific_opt(0, &self.last_used).encode(encoder)?;
        Ok(())
    }
}

impl FixedTag for PhraseSeqRef<'_> {
    const TAG: Tag = Tag::Sequence;
}

impl EncodeValue for PhraseSeqRef<'_> {
    fn value_len(&self) -> der::Result<Length> {
        self.der_bytes.len().try_into()
    }

    fn encode_value(&self, encoder: &mut impl Writer) -> der::Result<()> {
        encoder.write(self.der_bytes)
    }
}

impl<'a> DecodeValue<'a> for PhraseSeqRef<'a> {
    fn decode_value<R: Reader<'a>>(reader: &mut R, header: der::Header) -> der::Result<Self> {
        reader.read_nested(header.length, |reader| {
            let der_bytes = reader.read_slice(header.length)?;
            Ok(Self { der_bytes })
        })
    }
}

#[derive(Default)]
struct VecWriter {
    buf: Vec<u8>,
}

impl VecWriter {
    fn new() -> VecWriter {
        VecWriter::default()
    }
    fn len(&self) -> usize {
        self.buf.len()
    }
}

impl Writer for VecWriter {
    fn write(&mut self, slice: &[u8]) -> der::Result<()> {
        self.buf.write_all(slice)?;
        Ok(())
    }
}

/// A builder to create a dictionary that can be loaded as a [`Trie`]
/// dictionary.
///
/// # Examples
///
/// Create a new dictionary:
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let tmpdir = tempfile::tempdir()?;
/// # std::env::set_current_dir(&tmpdir.path())?;
/// use std::fs::File;
///
/// use chewing::{syl, zhuyin::Bopomofo};
/// use chewing::dictionary::{DictionaryBuilder, TrieBuilder};
///
/// let mut file = File::create("dict.dat")?;
/// let mut builder = TrieBuilder::new();
/// builder.insert(&[
///     syl![Bopomofo::Z, Bopomofo::TONE4],
///     syl![Bopomofo::D, Bopomofo::I, Bopomofo::AN]
/// ], ("字典", 0).into());
/// builder.write(&mut file)?;
/// # Ok(())
/// # }
/// ```
///
/// # File Format
///
/// The dictionary file is defined using the platform independent [DER][DER]
/// encoding format, allowing them to be versioned and shared easily. All
/// integers are encoded in big endian.
///
///
/// <details>
/// <summary>Trie ASN.1 module definition</summary>
///
/// ```asn
#[doc = include_str!("trie.asn1")]
/// ```
/// </details>
///
/// ## File Header
///
/// A Trie file MUST begin with a SEQUENCE tag byte (0x30), followed
/// by a variable length integer that encodes the size of the remaining
/// document. Then there MUST be a Utf8String ("CHEW") and an INTEGER (0) that
/// indicates the version of the dictionary format. The file SHOULD NOT contain
/// any trailing data.
///
/// ### Info object
///
/// The info object contains information about the name, copyright, license of
/// the file, and other similar text.
///
/// ### Index object
///
/// The index object contains the trie node records serialized in BFS order. The
/// first record is the root node, followed by the nodes in the first layer,
/// followed by the nodes in the second layer, and so on.
///
/// <details>
/// <summary>Index OCTET STRING layout</summary>
///
/// ```text
///  0                   1                   2                   3
///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
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
/// ```
/// </details>
///
/// Each node record has fixed size.
///
/// **Internal node:**
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
/// **Leaf node:**
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
/// ### PhraseSeq object
///
/// The phraseSeq object contains all the phrases strings, their frequency, and
/// other data. Each phrase data contains several attributes.
///
/// Currently defined attributes:
///
/// - **Phrase: variable length**
///     - UTF-8 encoded string, not null-terminated.
/// - **Frequency: 32 bits (u32)**
///     - The frequency of the phrase.
/// - **Last used: 64 bits (u64) optional**
///     - The last used timestamp of a user phrase.
///
/// [Trie]: https://en.m.wikipedia.org/wiki/Trie
/// [DER]: https://en.m.wikipedia.org/wiki/X.690#DER_encoding
#[derive(Debug)]
pub struct TrieBuilder {
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
pub struct TrieStatistics {
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

impl TrieBuilder {
    /// Creates a new `TrieBuilder`.
    ///
    /// The builder is initialized with a empty root node. Use the [`insert`][Self::insert]
    /// method to add more entries to the dictionary.
    ///
    /// # Examples
    ///
    /// ```
    /// use chewing::dictionary::TrieBuilder;
    ///
    /// let mut builder = TrieBuilder::new();
    /// ```
    pub fn new() -> TrieBuilder {
        let root = TrieBuilderNode::default();
        TrieBuilder {
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
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use std::io::Cursor;
    ///
    /// use chewing::dictionary::TrieBuilder;
    ///
    /// let mut writer = Cursor::new(vec![]);
    /// let mut builder = TrieBuilder::new();
    ///
    /// builder.write(&mut writer)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write<T>(&self, mut writer: T) -> io::Result<usize>
    where
        T: Write,
    {
        const ROOT_ID: usize = 0;
        let mut dict_buf = Vec::new();
        let mut data_buf = VecWriter::new();
        let mut queue = VecDeque::new();

        // The root node's child index starts from 1 (0 is the root).
        let mut child_begin = 1;

        // Walk the tree in BFS order and write the nodes to the dict buffer.
        queue.push_back(ROOT_ID);
        while !queue.is_empty() {
            // Insert nodes layer by layer.
            let layer_nodes_count = queue.len();
            for _ in 0..layer_nodes_count {
                // OK to unwrap, we always have at least one queued item.
                let id = queue.pop_front().unwrap();
                let node = &self.arena[id];

                // An internal node has an associated syllable. The root node is
                // a special case with no syllable.
                if node.syllable.is_some() || id == ROOT_ID {
                    let syllable_u16 = node.syllable.map_or(0, |v| v.to_u16());
                    let child_len =
                        node.children.len() + if node.leaf_id.is_some() { 1 } else { 0 };
                    dict_buf.write_all(&(child_begin as u32).to_be_bytes())?;
                    dict_buf.write_all(&(child_len as u16).to_be_bytes())?;
                    dict_buf.write_all(&syllable_u16.to_be_bytes())?;
                } else {
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
                    let data_begin = data_buf.len();

                    for phrase in phrases {
                        phrase.encode(&mut data_buf).map_err(io_error)?;
                    }

                    let data_len = data_buf.len() - data_begin;
                    dict_buf.write_all(&(data_begin as u32).to_be_bytes())?;
                    dict_buf.write_all(&(data_len as u16).to_be_bytes())?;
                    dict_buf.write_all(&0u16.to_be_bytes())?;
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

        let trie_dict_ref = TrieFileRef {
            info: DictionaryInfoRef::new(&self.info),
            index: OctetStringRef::new(&dict_buf).map_err(io_error)?,
            phrase_seq: PhraseSeqRef {
                der_bytes: &data_buf.buf,
            },
        };

        let document = Document::encode_msg(&trie_dict_ref).map_err(io_error)?;
        writer.write_all(document.as_bytes())?;
        Ok(document.as_bytes().len())
    }

    /// Calculates the statistics of the dictionary.
    ///
    /// # Examples
    ///
    /// ```
    /// use chewing::{syl, zhuyin::Bopomofo};
    /// use chewing::dictionary::{DictionaryBuilder, TrieBuilder};
    ///
    /// let mut builder = TrieBuilder::new();
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
    pub fn statistics(&self) -> TrieStatistics {
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
                // OK to unwrap. We always have at least one queued item.
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

        TrieStatistics {
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

impl DictionaryBuilder for TrieBuilder {
    fn set_info(&mut self, info: DictionaryInfo) -> Result<(), BuildDictionaryError> {
        self.info = info;
        Ok(())
    }

    /// Inserts a new entry to the dictionary.
    ///
    /// If there exists an entry with same syllables and phrase then the entry
    /// is updated to the new value.
    ///
    /// # Examples
    ///
    /// ```
    /// use chewing::{syl, zhuyin::Bopomofo};
    /// use chewing::dictionary::{DictionaryBuilder, TrieBuilder};
    ///
    /// let mut builder = TrieBuilder::new();
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
        if let Some(it) = self.arena[leaf_id]
            .phrases
            .iter_mut()
            .find(|it| it.as_str() == phrase.as_str())
        {
            *it = phrase;
        } else {
            self.arena[leaf_id].phrases.push(phrase);
        }
        Ok(())
    }

    fn build(&mut self, path: &Path) -> Result<(), BuildDictionaryError> {
        let mut tmpname = path.to_path_buf();
        let pseudo_random = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|du| du.subsec_micros())
            .unwrap_or_default();
        tmpname.set_file_name(format!("chewing-{}.dat", pseudo_random));
        let database = File::create(&tmpname)?;
        let mut writer = BufWriter::new(&database);
        self.write(&mut writer)?;
        writer.flush()?;
        database.sync_data()?;
        fs::rename(&tmpname, path)?;
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Default for TrieBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Cursor, Seek},
        num::NonZeroUsize,
    };

    use crate::{
        dictionary::{
            Dictionary, DictionaryBuilder, DictionaryInfo, LookupStrategy, Phrase, TrieOpenOptions,
            trie::TrieBuilderNode,
        },
        syl,
        zhuyin::Bopomofo,
    };

    use super::{Trie, TrieBuilder};

    #[test]
    fn test_tree_construction() -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = TrieBuilder::new();
        builder.insert(
            &[
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
            ],
            ("測試", 100).into(),
        )?;
        builder.insert(
            &[
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
        let mut builder = TrieBuilder::new();
        builder.insert(
            &[syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            ("測", 1).into(),
        )?;
        builder.insert(
            &[syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            ("冊", 1).into(),
        )?;
        let mut cursor = Cursor::new(vec![]);
        builder.write(&mut cursor)?;
        cursor.rewind()?;
        let dict = Trie::new(&mut cursor)?;
        assert_eq!(
            vec![Phrase::new("測", 1), Phrase::new("冊", 1)],
            dict.lookup_all_phrases(
                &[syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
                LookupStrategy::Standard
            )
        );

        Ok(())
    }

    #[test]
    fn tree_lookup_word_fuzzy() -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = TrieBuilder::new();
        builder.insert(
            &[syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            ("測", 1).into(),
        )?;
        builder.insert(
            &[syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
            ("冊", 1).into(),
        )?;
        let mut cursor = Cursor::new(vec![]);
        builder.write(&mut cursor)?;
        cursor.rewind()?;
        let dict = TrieOpenOptions::new().read_from(&mut cursor)?;
        assert_eq!(
            vec![Phrase::new("測", 1), Phrase::new("冊", 1)],
            dict.lookup_all_phrases(
                &[syl![Bopomofo::C, Bopomofo::E]],
                LookupStrategy::FuzzyPartialPrefix
            )
        );
        assert_eq!(
            vec![Phrase::new("測", 1), Phrase::new("冊", 1)],
            dict.lookup_all_phrases(&[syl![Bopomofo::C]], LookupStrategy::FuzzyPartialPrefix)
        );

        Ok(())
    }

    #[test]
    fn tree_lookup_phrase() -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = TrieBuilder::new();
        builder.insert(
            &[
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
            ],
            ("測試", 1).into(),
        )?;
        builder.insert(
            &[
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
            ],
            ("策試", 2).into(),
        )?;
        builder.insert(
            &[
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
                syl![Bopomofo::CH, Bopomofo::ENG, Bopomofo::TONE2],
                syl![Bopomofo::G, Bopomofo::U, Bopomofo::ENG],
            ],
            ("測試成功", 3).into(),
        )?;
        let mut cursor = Cursor::new(vec![]);
        builder.write(&mut cursor)?;
        cursor.rewind()?;
        let dict = Trie::new(&mut cursor)?;
        assert_eq!(
            vec![Phrase::new("策試", 2), Phrase::new("測試", 1)],
            dict.lookup_all_phrases(
                &[
                    syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                    syl![Bopomofo::SH, Bopomofo::TONE4]
                ],
                LookupStrategy::Standard
            )
        );
        assert_eq!(
            vec![Phrase::new("測試成功", 3)],
            dict.lookup_all_phrases(
                &[
                    syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                    syl![Bopomofo::SH, Bopomofo::TONE4],
                    syl![Bopomofo::CH, Bopomofo::ENG, Bopomofo::TONE2],
                    syl![Bopomofo::G, Bopomofo::U, Bopomofo::ENG],
                ],
                LookupStrategy::Standard
            )
        );
        assert_eq!(
            Vec::<Phrase>::new(),
            dict.lookup_all_phrases(
                &[
                    syl![Bopomofo::C, Bopomofo::U, Bopomofo::O, Bopomofo::TONE4],
                    syl![Bopomofo::U, Bopomofo::TONE4]
                ],
                LookupStrategy::Standard
            )
        );

        Ok(())
    }

    #[test]
    fn tree_lookup_phrase_fuzzy() -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = TrieBuilder::new();
        builder.insert(
            &[
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
            ],
            ("測試", 1).into(),
        )?;
        builder.insert(
            &[
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
            ],
            ("策試", 2).into(),
        )?;
        builder.insert(
            &[
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
                syl![Bopomofo::CH, Bopomofo::ENG, Bopomofo::TONE2],
                syl![Bopomofo::G, Bopomofo::U, Bopomofo::ENG],
            ],
            ("測試成功", 3).into(),
        )?;
        let mut cursor = Cursor::new(vec![]);
        builder.write(&mut cursor)?;
        cursor.rewind()?;
        let dict = TrieOpenOptions::new()
            .fuzzy_search(true)
            .read_from(&mut cursor)?;
        assert_eq!(
            vec![Phrase::new("策試", 2), Phrase::new("測試", 1)],
            dict.lookup_all_phrases(
                &[syl![Bopomofo::C, Bopomofo::E], syl![Bopomofo::SH]],
                LookupStrategy::FuzzyPartialPrefix
            )
        );
        assert_eq!(
            vec![Phrase::new("策試", 2), Phrase::new("測試", 1)],
            dict.lookup_all_phrases(
                &[syl![Bopomofo::C], syl![Bopomofo::SH]],
                LookupStrategy::FuzzyPartialPrefix
            )
        );
        assert_eq!(
            vec![Phrase::new("測試成功", 3)],
            dict.lookup_all_phrases(
                &[
                    syl![Bopomofo::C, Bopomofo::E],
                    syl![Bopomofo::SH],
                    syl![Bopomofo::CH, Bopomofo::ENG],
                    syl![Bopomofo::G, Bopomofo::U, Bopomofo::ENG],
                ],
                LookupStrategy::FuzzyPartialPrefix
            )
        );
        assert_eq!(
            vec![Phrase::new("測試成功", 3)],
            dict.lookup_all_phrases(
                &[
                    syl![Bopomofo::C],
                    syl![Bopomofo::SH],
                    syl![Bopomofo::CH],
                    syl![Bopomofo::G],
                ],
                LookupStrategy::FuzzyPartialPrefix
            )
        );
        assert_eq!(
            Vec::<Phrase>::new(),
            dict.lookup_all_phrases(
                &[
                    syl![Bopomofo::C, Bopomofo::U, Bopomofo::O, Bopomofo::TONE4],
                    syl![Bopomofo::U, Bopomofo::TONE4]
                ],
                LookupStrategy::FuzzyPartialPrefix
            )
        );

        Ok(())
    }

    #[test]
    fn tree_builder_duplicate_phrase() {
        let mut builder = TrieBuilder::new();
        builder
            .insert(
                &[
                    syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                    syl![Bopomofo::SH, Bopomofo::TONE4],
                ],
                ("測試", 1).into(),
            )
            .expect("no error");
        builder
            .insert(
                &[
                    syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                    syl![Bopomofo::SH, Bopomofo::TONE4],
                ],
                ("測試", 2).into(),
            )
            .expect("no error");
    }

    #[test]
    fn stable_word_sort_order() -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = TrieBuilder::new();
        for word in ["冊", "策", "測", "側"] {
            builder.insert(
                &[syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4]],
                (word, 0).into(),
            )?;
        }
        let mut cursor = Cursor::new(vec![]);
        builder.write(&mut cursor)?;
        cursor.rewind()?;
        let dict = Trie::new(&mut cursor)?;
        assert_eq!(
            vec![
                Phrase::new("冊", 0),
                Phrase::new("策", 0),
                Phrase::new("測", 0),
                Phrase::new("側", 0),
            ],
            dict.lookup_all_phrases(
                &[syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],],
                LookupStrategy::Standard
            )
        );
        Ok(())
    }

    #[test]
    fn stable_phrase_sort_order() -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = TrieBuilder::new();
        builder.insert(
            &[
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
            ],
            ("側室", 318).into(),
        )?;
        builder.insert(
            &[
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
            ],
            ("側視", 318).into(),
        )?;
        builder.insert(
            &[
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
            ],
            ("策士", 318).into(),
        )?;
        builder.insert(
            &[
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
            ],
            ("策試", 318).into(),
        )?;
        builder.insert(
            &[
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
            ],
            ("測試", 9318).into(),
        )?;
        let mut cursor = Cursor::new(vec![]);
        builder.write(&mut cursor)?;
        cursor.rewind()?;
        let dict = Trie::new(&mut cursor)?;
        assert_eq!(
            vec![
                Phrase::new("測試", 9318),
                Phrase::new("策試", 318),
                Phrase::new("策士", 318),
                Phrase::new("側視", 318),
                Phrase::new("側室", 318),
            ],
            dict.lookup_all_phrases(
                &[
                    syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                    syl![Bopomofo::SH, Bopomofo::TONE4],
                ],
                LookupStrategy::Standard
            )
        );
        Ok(())
    }

    #[test]
    fn tree_builder_write_read_metadata() -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = TrieBuilder::new();
        let info = DictionaryInfo {
            name: "name".into(),
            copyright: "copyright".into(),
            license: "license".into(),
            version: "version".into(),
            software: "software".into(),
        };
        builder.set_info(info)?;

        let mut cursor = Cursor::new(vec![]);
        builder.write(&mut cursor)?;
        cursor.rewind()?;
        let dict = Trie::new(&mut cursor)?;
        let info = dict.about();

        assert_eq!("name", info.name);
        assert_eq!("copyright", info.copyright);
        assert_eq!("license", info.license);
        assert_eq!("version", info.version);
        assert_eq!("software", info.software);
        Ok(())
    }

    #[test]
    fn tree_entries() -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = TrieBuilder::new();
        builder.insert(
            &[
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
            ],
            ("測試", 1).into(),
        )?;
        builder.insert(
            &[
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
            ],
            ("策試", 2).into(),
        )?;
        builder.insert(
            &[
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
                syl![Bopomofo::CH, Bopomofo::ENG, Bopomofo::TONE2],
                syl![Bopomofo::G, Bopomofo::U, Bopomofo::ENG],
            ],
            ("測試成功", 3).into(),
        )?;
        builder.insert(
            &[
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
                syl![Bopomofo::SH],
                syl![Bopomofo::B, Bopomofo::AI, Bopomofo::TONE4],
            ],
            ("測試失敗", 3).into(),
        )?;
        let mut cursor = Cursor::new(vec![]);
        builder.write(&mut cursor)?;
        cursor.rewind()?;
        let dict = Trie::new(&mut cursor)?;
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
}
