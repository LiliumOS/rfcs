# Lilium Filesystem

## Summary

The Lilium Filesystem is a filesystem designed specifically to work with the Lilium IO subsystem filesystem APIs. It exposes features that are specifically tailored to the Lilium OS, though may function on other operating systems as well.

## Motivation

<!--Provide a more concrete reasoning for this proposal-->

## Informative Explanation

<!--Provide an informative explanation of proposal. 
This is intended to be read by someone who wishes to understand the proposal but may not have advanced technical background.
This section is intended for:
* People using the Lilium Operating System as a Software Developer
* People looking to understand the Lilium Operating System
* People looking to understand the Lilium Project as a whole

This section is not normative-->

## Normative Text

### GUID Partition Table

When a partition in a GUID Partition Table is formatted for LiliumFS, the partition type ID `2355710d-7e9e-5b2c-811f-52ad19c67e0b` shall be used unless a specific purpose requrires another type.

The top 16 bits of the partition attributes are presently reserved and must be 0.

### MBR Partition Table

There is no specific support specified for LiliumFS partitions allocated in MBR format. However, only LBA addressing may be used for 

### Block Size

In this document, the block sized used is defined to be 4096 bytes (4KiB or 2^12 bytes). When using Logical Block Addressing for physical media with 512-byte blocks, implementations are responsible for translating from block numbers/sizes in filesystem structures to actual block addresses on the physical media. Block indexes begin from index 0 (byte offset 0), and count.

### Optional Identity Header

At byte offset 512 (within block 0 of the partition), there may be an identity header that identifies the partition as compatible with Lilium FS. This is optional and not required and the interpretation of this header is not required for correct interpretation of the filesystem. However, a malformed Identity Header may indicate that the partition is some other filesystem.

The identity header is defined by the following Structure:
```
struct LiliumFSIdentity: align(512) {
    identifier: Uuid,
    next_header_bytes: u64,
}
```

`identifier` is the UUID `e9ea3705-c42b-54c8-bd4c-8e1901c12f12`. `next_header_bytes` is the number of bytes from the start of the Identity Header to the start of the Primary Header, which is identically the value `3584`.

### Primary Header

The Primary Header is located at block index 1. It identifies the partition or volume as a LiliumFS volume and defines the information necessary for correct interpretation of the filesystem.

The format is defned as follows

```
struct LiliumFSHeader : align(4096) {
    identifier: Uuid,
    header_size: u16,
    volume_name: [u8; 22],
    volume_name_offset: u64,
    volume_id: Uuid,
    object_list_end: u128,
    object_list_count: u64,
    root_object: u64,
    allocation_list_begin: u128,
    allocation_list_length: u64,
    required_features_list: u64,
    optional_features_list: u64,
}
```

`identifier` is the unique identifier for the filesystem format, `1183429f-40f8-5638-baf7-835828aba375`. This UUID is unique to LiliumFS and is unlikely to be present on any other volume.

`header_size` is the total size, in bytes, of the header. It must be at most 4096, and at least 120. This allows extensions that grow the size of the header to be backwards compatible to earlier filesystems.

`volume_name` is the inline name of the volume, if less than 21 bytes (UTF-8 encoded), padded with 0s, or all 0s if the volume name is out of line (when `volume_name_offset` is non-zero). `volume_name_offset` is either `0` or the offset into the `Strings` stream of the root object which is the out of line name of the volume.
`volume_id` is a Unique ID generated when the FS is first created that uniquely identifies it. When the volume is located on a partition of a GPT-formatted disk, this should be the same as the partiton ID.

`object_list_end` points to the block which follows the last block of the object list (typically, this is the last block of the volume), and `object_list_count` is the total count of the objects in the object list (not the number of blocks). 

`root_object` is the index into the object list which is the root directory of the filesystem.

`allocation_list_begin` points to the first block of the allocation list. `allocation_list_length` is the number of blocks used to store the allocation list.

`required_features_list` and `optional_features_list` are bitsets for features that are reserved for future use. Unknown features set in `optional_features_list` must be ignored. Unknown features set in `required_features_list` are an error (the filesystem cannot be used).

At the end of the block on which the Primary Header resides, A 32-byte Sha3-256 hash of the entire header, as defined by `header_size`, is present. This is used as an integrity check.
The primary header and this hash is mirrored to the block immediately following the end of the object list.

### Allocation List

The allocation list is used to track extents for streams it consists of a list of 32 byte entries (with 128 entries per block) defined as follows:

```
struct Allocation : align(32) {
    begin_block: u128,
    length_bytes: u64,
    attributes: u64,
}
```

The `attributes` word is set to 0 in current versions of the filesystem, `begin_block` is the first block of the span, and `length_bytes` is the total size in bytes, of the span. Note that allocations are only granular to the block - if any byte in a block is allocated, the entire block cannot be used for another allocation.

An allocation with a `begin_block` of `0` and a `length_bytes` of `0` is a null (unused) entry. This can be used to easily free an allocation.

The allocation list shall consist of a minimum of 4 entries:

* An null sentinel allocation at entry `0`.
* One with begin_block 0 and length_bytes 8192 to cover the unused block 0 and the primary header,
* One that covers the entire allocation list, and
* One that covers the entire object list.

The above constraints mean that every in-use block can be accounted for by scanning the allocation list.

### Object List

The object list is the primary list of objects in the LiliumFS. An object is an entity on the filesystem that carries data and metadata through streams (Files and Directories are all examples of objects).

The object list grows down from usually the last block of the filesystem (specifically, it ends before the first byte of `object_list_end`). The entries are 64 bytes in size, 0 indexed and count downwards, with the element ending at the high address (thus, object 0 ends at `object_list_end`, object 1 ends at `object_list_end - 64 bytes`, etc.). 
Because the object list grows downward from the end of the filesystem, and the allocation list grows upward from the start, it is unlikely that they will conflict.

Each Entry in the object list is defined as follows:

```
struct Object: align(64) {
    weak_count: u32,
    strong_count: u32,
    object_type: u16,
    flags: u32,
    unused: u8,
    streams_indirection: u8,
    streams_allocation: u64,
    strings_index: u64,
    pad([u64;4])
}
```

`weak_count` is the total number of references to the object (both strong and weak). When this value is `0`, the `Object` is not in use and the entire contents of the entry are undefined.

`strong_count` is the number of strong references to the object. When this value is `0`, the streams of the object can be deallocated.

`object_type` is a hint about the primary purpose and stream of the object:
- `0` (Regular File): The object primarily contains data, to be interpreted by programs opening the file. Regular files should have a "FileData" stream that contains these bytes
- `1` (Directory): The object is primarily a directory that contains other files. Directories should have a "DirectoryContent" stream that contains the files
- `2` (Symlink): The Object Primarily refers to the logical path of another object. In most cases, Symlinks are transparently replacable with the referent path. Symlinks should have a "SymlinkTarget" stream that contains the logical path as a UTF-8 stirng
- `3` (POSIX FIFO): The object is primarily a Named Pipe/FIFO object. 
- `4` (Unix Socket): The object is primarily a Unix Socket. 
- `5` (Block Device): The object is primarily a Block Device. Block Device Files should have a "DeviceId" stream or a "LegacyDeviceNumber" stream.
- `6` (Character Device): The object is primarily a Character Device. Character Device Files should have a "DeviceId" stream or a "LegacyDeviceNumber" stream.
- 65535 (Custom Type): The object has implementation-specific or custom semantics. Custom Type Objects should have a "CustomObjectInfo" stream.
- other values are reserved and implementations MUST not allow access to objects with invalid types. 

`flags` contain flags for the Object. No such flags are currently defined and the field shall be `0`.

`unused` shall be `0`. 

`streams_indirection` is the indirection level for the `Streams` stream. Where `strong_count > 0`, this field shall be at least `1`.

`streams_allocation` is an index into the allocation table that refers to the content of the `Streams` stream (where `streams_indirection > 1`, see the specification for Indirect Streams).

`strings_index` is either `0` or the index into the streams array that refers to the `Strings` stream.

### Streams

Each object stores data and metadata on several streams. These allow implementations to determine the content and meaning of the file.  

Streams are referred to via the `Streams` stream, which itself is located from the object structure.

The `Streams` stream contains an array of 128-byte `StreamDescriptor`s.  
The `StreamDescriptor` type is:

```
struct StreamDescriptor : align(128) {
    name: [u8; 32],
    name_index: u64,
    flags_and_type: u64,
    alloc: u64,
    size: u64,
    inline_content: [u8; 64]
}
```

`name` is the name of the stream if it is at most 32 bytes long, padded with 0 bytes, or all `0` bytes, in which case, `name_index` refers to the name of the stream.
`name_index` is either 0 or the index into the `Strings` stream of the object that refers to the name. The name of the stream determines its behavior.

`flags_and_type` are defined as follows:  

* The bottom 4 bits (indirection) contains the indirection level, where `0` means the the content is present in `inline_content`, `1` means the content is located by `alloc`, and values `2` and above mean that `alloc` points to an indirection array and the content must be resolved by iterating through that many levels of indirections, starting from `alloc`, Up to indirection 15 is permitted
* Bits 4, 5, 6, and 7 are support bits: If Bit 4 (REQUIRED) is set, the implementation shall not permit access to the object if it does not recognize the stream name. If Bit 5 (WRITE_REQUIRED) is set, the implementation shall not permit write access to the object if it does not recognize the stream name unless it also removes that stream at the same time. If Bit 6 (PRESERVE) is set, the implementation shall not remove the stream if it does not recognize the stream name (See Below for an exception). If Bit 7 (STRINGS) is set, then the stream contains structural references to the `Strings` stream.
* Bits 8 through 16 (stype) contain the stream type, of which 8 are presently defined:
    * `0` (UDATA): The stream contains unstructured data that can be read or written to arbitrarily,
    * `1` (SDATA): The stream contains structured data that can be read but must be written to according to the structure,
    * `2` (UMDATA): The stream contains unstructured metadata (comment)
    * `3` (USDATA): The stream contains structured metadata that can be read but must be written to according to the structure,
    * `4` (SECURITY): The stream contains structured metadata that is critical to security,
    * `5` (NDATA): The stream contains no data (size is 0),
    * `6` (DESC): The stream contains structured metadata that describes how to interpret another stream,
    * `7` (INFO): The stream contains structured metadata that describes how to interpret the object.
    * `255` (STREAM_DEFINED): The content and behaviour of the stream cannot be interpreted without recgonizing the stream.
* Bits 48 through 63 (stream_bits) are stream-specific bits. The meaning is defined per stream name and shall be ignored if the implementation does no recognize the stream type


`alloc` is either `0` (when `indirection == 0`) or is the index into the allocation list that provides access to the content of the file (when `indirection > 0`). 

`size` is the total size of the stream, in bytes. 

`inline_content` contains the content of the stream if `indirection == 0`, otherwise the contents are undefined. Thus small amounts of data can be stored directly within the stream descriptor.

#### Indirections

When `indirection > 0`, the `alloc` entry refers to the content of the stream by indexing the allocation list. 

To determine the content of the stream, form a tree as follows:
* The root node of the tree is `alloc`,
* There are `indirection` total levels of node,
* The last level of nodes are all leaf nodes, with all other levels being indirection nodes.
* For each indirection node, populate the next level down of nodes by taking the span referred by the allocation entry pointed to by the node as an array of `u64`, each of which are indexes into the allocation table that point to the next node.

The content of the stream is taken by concatenating each leaf node from left to right.

#### The `Streams` stream

The first entry of the `Streams` array is a reference to the `Streams` stream itself. The properties of this entry is as follows:
* `name` of the stream is `Streams`. The name is always inline and `name_index` is set to `0`.
* `alloc` is identically the value of `streams_allocation` for the object, 
* `type_and_flags` is set as follows:
    * indirection is set to `streams_indirection`
    * The `REQUIRED`, `PRESERVE`, and `STRINGS` bits are all set,
    * `stype` is `INFO`,
    * `stream_bits` are all set to `0`

#### The `Strings` stream

An object may have a `Strings` stream. This allows other streams (including the `Streams` stream) to refer to arbitrary length UTF-8 data without having to encode potentialy long data.

The `Strings` stream is a packed array of UTF-8 strings that are separated by a NUL (`0`) byte. Index `0` in the stream is this byte, and thus an index of `0` into this stream can be treated as a sentinel or a 0-length string.

THe properties of the `Strings` stream are as follows:
* `name` of the stream is `Strings`. The name is always inline and `name_index` is `0`,
* `type_and_flags` is set as follows:
    * The `REQUIRED` and `PRESERVE` bits are all set,
    * `stype` is `DESC`,
    *  `stream_bits` are all set to `0`.

There may be at most one `Strings` stream in an object.

#### Other Streams

An object may have an arbitrary set of streams. The `name` of the stream identifies how to refer to it, and, in some cases, its behaviour. The name can be arbitrary and both implementations and users can define additional stream types. Names consisting solely of ASCII letters, Numbers, and the `_` character are reserved for the Lilium Project.

Note that this RFC does not define a mechanism for arranging for the uniqueness of third-party stream types.

## Security Considerations

<!--If the proposal requires users and/or implementors to take anything into consideration for security reasons, document this here.-->

## ABI Considerations

None

## Prior Art

## Future Direction

<!--
Provide an informative explanation of any future possibilities.
-->

## References

### Normative References

<!--List all documents cited normatively here. 
A Normative Reference is a reference within the main text (Normative Text section, Security Considerations, or Registry Impacts) for the meaningful content within.
For example, if you use definitions from another specification, it would be a normative reference.
-->

### Informative References

<!--Include any documents cited to provide informative context only-->