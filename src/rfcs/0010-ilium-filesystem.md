# Lilium Filesystem

## Summary

The Lilium Filesystem is a filesystem designed specifically to work with the Lilium IO subsystem filesystem APIs. It exposes features that are specifically tailored to the Lilium OS, though may function on other operating systems as well.

## Motivation

The Lilium OS is built arround varying design axioms regarding security and extensibility. These designs can be served partially, but not entirely, by existing filesystems like ext4, zfs, and NTFS. 
Additionally some design constraints (such as using UUIDs for security principals, and a unified user/group system) make system structures of existing filesystems require adaptation for use on Lilium, thus making it unsuitable for use for system files.

Finally, a freshly designed filesystem has the opportunity to make design choices that are more useful now than when previous ones were designed, and be more optimal on modern systems.

## Informative Explanation

The Lilium Filesystem is made up of a series of o

## Normative Text

### GUID Partition Table

When a partition in a GUID Partition Table is formatted for LiliumFS, the partition type ID `2355710d-7e9e-5b2c-811f-52ad19c67e0b` shall be used unless a specific purpose requrires another type.

The top 16 bits of the partition attributes are presently reserved and must be 0.

### MBR Partition Table

There is no specific support specified for LiliumFS partitions allocated in MBR format. However, only LBA addressing may be used for LiliumFS if that support is provided by a third party.

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

The format is defined as follows. It is currently 128 bytes in size, though it takes up the entirety of block 1.

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
    header_flags: u64,
}
```

`identifier` is the unique identifier for the filesystem format, `1183429f-40f8-5638-baf7-835828aba375`. This UUID is unique to LiliumFS and is highly unlikely to be present on any other volume.

`header_size` is the total size, in bytes, of the header. It must be at most 4096, and at least 128. This allows extensions that grow the size of the header to be backwards compatible to earlier filesystems.

`volume_name` is the inline name of the volume, if less than 21 bytes (UTF-8 encoded), padded with 0s, or all 0s if the volume name is out of line (when `volume_name_offset` is non-zero). `volume_name_offset` is either `0` or the offset into the `Strings` stream of the root object which is the out of line name of the volume.
`volume_id` is a Unique ID generated when the FS is first created that uniquely identifies it. When the volume is located on a partition of a GPT-formatted disk, this should be the same as the partiton ID.

`object_list_end` points to the block which follows the last block of the object list (typically, this is the last block of the volume), and `object_list_count` is the total count of the objects in the object list (not the number of blocks). Indecies into the object list count backwards from this position, `object_list_end` also points past the end of the first entry in the object array.

`root_object` is the index into the object list which is the root directory of the filesystem.

`allocation_list_begin` points to the first block of the allocation list. `allocation_list_length` is the number of blocks used to store the allocation list.

`required_features_list` and `optional_features_list` are bitsets for features that are reserved for future use. Unknown features set in `optional_features_list` must be ignored. Unknown features set in `required_features_list` are an error (the filesystem cannot be used).

At the end of the block on which the Primary Header resides, A 32-byte Sha3-256 hash of the entire header, as defined by `header_size`, is present. This is used as an integrity check.
The primary header and this hash is mirrored to the block immediately following the end of the object list.

`header_flags` is a list of flags for the whole filesystem. These flags do not indicate features but instead global state. One such flag is provided:

* 0x0000_0000_0000_0001 (`temp_data`): If this flag is set, the contents of the filesystem are not considered to be preserved between boots/mounts. All objects other than the root object may be deallocated when the filesystem is mounted (unless mounted read-only), and the root directory may be implicitly cleared on mount. This flag is suitable for temporary filesystems that may need to store large data (which is unsuitable for a ramfs). 

### Allocation List

The allocation list is used to track extents for streams it consists of a list of 32 byte entries (with 128 entries per block) defined as follows:

```
struct Allocation : align(32) {
    begin_block: u128,
    length_bytes: u64,
    in_use: u32,
    attributes: u32,
}
```

The `in_use` flag is set to 1 if the allocation is in use, that is: 

* It is referred to, directly or indirectly, by a stream, or
* It is one of the special entries described below, other than the null entry.

Otherwise it is set to 0.

The `attributes` word is set to 0 in current versions of the filesystem, `begin_block` is the first block of the span, and `length_bytes` is the total size in bytes, of the span. Note that allocations are only granular to the block - if any byte in a block is allocated, the entire block cannot be used for another allocation.

An allocation with a `begin_block` of `0` and a `length_bytes` of `0` is a null (unused) entry. This can be used to easily free an allocation.

The allocation list shall consist of a minimum of 4 entries:

* An null sentinel allocation at entry `0` (begin_block=0, length_bytes=0, in_use=0).
* One with begin_block 0 and length_bytes 8192 to cover the unused block 0 and the primary header,
* One that covers the entire allocation list, and
* One that covers the entire object list.

The above constraints mean that every in-use block can be accounted for by scanning the allocation list.

### Object List

The object list is the primary list of objects in the LiliumFS. An object is an entity on the filesystem that carries data and metadata through streams (Files and Directories are all examples of objects).

The object list grows down from usually the last block of the filesystem (specifically, it ends before the first byte of `object_list_end`). The entries are 64 bytes in size, 1 indexed and count downwards, with the element ending at the high address. The nth object in the list begins at byte `object_list_end*4096 - n*64`.
Because the object list typically grows downward from the end of the filesystem, and the allocation list grows upward from the start, it is unlikely that they will conflict.

Object indecies do not use the 0 value as the first element is at index 1. This value is an error but may be used for specific purposes in later versions.

Each Entry in the object list is defined as follows:

```
struct Object: align(64) {
    weak_count: u32,
    strong_count: u32,
    flags: u32,
    object_type: u16,
    __unused: u8,
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
- `1` (Directory): The object is primarily a directory that contains other files. Directories should have a "DirectoryContent" stream that contains the list of files
- `2` (Symlink): The Object Primarily refers to the logical path of another object. In most cases, Symlinks are transparently replacable with the referent path. Symlinks should have a "SymlinkTarget" stream that contains the logical path as a UTF-8 stirng
- `3` (POSIX FIFO): The object is primarily a Named Pipe/FIFO object. This object type has no associated stream
- `4` (Unix Socket): The object is primarily a Unix Socket. This object type has no associated stream
- `5` (Block Device): The object is primarily a Block Device. Block Device Files should have a "DeviceId" stream or a "LegacyDeviceNumber" stream.
- `6` (Character Device): The object is primarily a Character Device. Character Device Files should have a "DeviceId" stream or a "LegacyDeviceNumber" stream.
- 65535 (Custom Type): The object has implementation-specific or custom semantics. Custom Type Objects should have a "CustomObjectInfo" stream.
- other values are reserved and implementations MUST not allow access to objects with invalid types. 

`flags` contain flags for the Object. No such flags are currently defined and the field shall be `0`.

`__unused` shall be `0`. 

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
* Bits 4, 5, 6, and 7 are support bits: If Bit 4 (REQUIRED) is set, the implementation shall not permit any access to the object if it does not recognize the stream name. If Bit 5 (WRITE_REQUIRED) is set, the implementation shall not permit write access to the object if it does not recognize the stream name unless it also removes that stream at the same time. If Bit 6 (PRESERVE) is set, the implementation shall not remove the stream if it does not recognize the stream name (See Below for an exception). If Bit 7 (STRINGS) is set, then the stream contains structural references to the `Strings` stream.
* Bits 8 through 16 (stype) contain the stream type, of which 8 are presently defined:
  * `0` (UDATA): The stream contains unstructured data that can be read or written to arbitrarily,
  * `1` (SDATA): The stream contains structured data that can be read but must be written to according to the structure,
  * `2` (UMDATA): The stream contains unstructured metadata (comment)
  * `3` (SMDATA): The stream contains structured metadata that can be read but must be written to according to the structure,
  * `4` (SECURITY): The stream contains structured metadata that is critical to security,
  * `5` (NDATA): The stream contains no data (size is 0),
  * `6` (DESC): The stream contains structured metadata that describes how to interpret another stream,
  * `7` (INFO): The stream contains structured metadata that describes how to interpret the object.
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

Every implementation of the filesystem must support the `Streams` stream.

The `Streams` stream cannot fit into `inline_content` by definition, as it contains at least one entry that is larger than 64 bytes. Thus, indirection is always >=1.

There is exactly one `Streams` stream on a file, which must be the first entry (index 0) of the `Streams` array.

#### The `Strings` stream

An object may have a `Strings` stream. This allows other streams (including the `Streams` stream) to refer to arbitrary length UTF-8 data without having to encode potentialy long data.

The `Strings` stream is a packed array of UTF-8 strings that are separated by a NUL (`0`) byte. Index `0` in the stream is this byte, and thus an index of `0` into this stream can be treated as a sentinel or a 0-length string.

THe properties of the `Strings` stream are as follows:
* `name` of the stream is `Strings`. The name is always inline and `name_index` is `0`,
* `type_and_flags` is set as follows:
    * The `REQUIRED` and `PRESERVE` bits are all set,
    * `stype` is `DESC`,
    *  `stream_bits` are all set to `0`.

There may be at most one `Strings` stream in an object. If one is present, the `strings_index` field of the object specifies the index (not offset) into the `Streams` array that references the `Strings` stream. Otherwise this field is `0`.

#### Other Streams

An object may have an arbitrary set of streams. The `name` of the stream identifies how to refer to it, and, in some cases, its behaviour. The name can be arbitrary and both implementations and users can define additional stream types. Names consisting solely of ASCII letters, Numbers, and the `_` character are reserved for the Lilium Project.

Note that this RFC does not define a mechanism for arranging for the uniqueness of third-party stream types.

### Required Stream types

Several Stream types are defined that are not critical to interpreting the filesystem itself, but are necessary for any implementation to support regardless. A Correct implementation must defined support for at least the following stream types (in addition to the types identified previously):

* `FileData`
* `DirectoryContent`,
* `SymlinkTarget`,
* `SecurityDescriptor`.

#### The `FileData` stream

The `FileData` stream is a stream that encapsilates the contents of a file. It has the following stream properties:

* `name` of the stream is `FileData`
* `type_and_flags` is set as follows:
  * `REQUIRED` is set. 
  * `stype` is 0 (`UDATA`)
    * Note: depending on the file, the stream may contain data used by an application program that may be structured from the point of view of the program. Regardless, `stype` 1 (`SDATA`) shall not be used (as the file content is unstructured from the perspective of the filesystem and filesystem implementation)
  * All `stream_bits` are 0

There may be at most one `FileData` stream on an object.

#### The `DirectoryContent` stream

The `DirectoryContent` stream is a stream that lists the files contained in a directory. It has the following stream properties:

* `name` of the stream is `DirectoryContent`
* `type_and_flags` is set as follows:
    * `REQUIRED` and `STRINGS` are set
    * `stype` is `SDATA`
    * All `stream_bits` are 0

The stream content is an array of the following 64-byte structures. Note that the array may be empty:

```
struct DirectoryContentEntry {
    name_ref: u64,
    flags: u64,
    object_ref: u64,
    name_bytes: [u8; 32],
    pad([u8;8])
}
```

`name_ref` is either 0 or a offset into the `Strings` stream of the object referring to the name of the directory entry. If this is zero, then `name_bytes` contains the name as UTF-8 string padded with null bytes, which may be up to (and including) 32 bytes long. The name may not contain any of the following characters:

* `/`
* `\`
* `$`
* `\0`

Additionally, the name may not be any of the following strings:

* `.`
* `..`

Note: An empty file name is permitted, but typically cannot be accessed. 

A directory cannot contain more than one file with a given name string. This is whether or not the name is inlined (in `name_bytes`) or outlined (referenced by `name_ref`).

`flags` is a list of directory entry flags, which are the following:

* 0x0000_0000_0000_0001 (`weak`): The entry references the specified object weakly - only the `weak_count` field of the object structure is incremented by the reference instead of both `weak_count` and `strong_count`,
* 0x0000_0000_0000_0002 (`hidden`): The entry should be hidden from display/listing in user interfaces unless it is requested to be shown explicitly,
* 0x0000_0000_0000_0004 (`system`): UIs should display the file as a system file/system resource.
* 0xFFFF_0000_0000_0000: These bits are reserved for future optional use and should not be set. Any unknown bits that are set in this list must be ignored.

All other bits must not be set, and are an error if they are set.

`object_ref` refers to the index (not offset) into the objects array of the filesystem

There may be at most one `DirectoryContent` stream on an object.

#### The `SymlinkTarget` stream

The `SymlinkTarget` stream provides a string reference to the target of a symbolic link object. It has the following stream properties:

* `name` of the stream is `SymlinkTarget`
* `type_and_flags` is set as follows:
    * `REQUIRED` is set
    * `stype` is `SDATA`
    * All `stream_bits` are 0

The content of the stream is a UTF-8 string contains a path. Resolving the symlink stream performs ordinary path resolution, using the symlink object as the starting point for relative paths.

Note: The string is emitted directly into the stream in all cases - the stream never references the `Strings` stream.

There may be at most one `SymlinkTarget` stream on an object.

#### The `SecurityDescriptor` stream

The `SecurityDescriptor` stream is the primary means to establish file access control. The first 64 bytes of the file is an security header, and then is followed by a list of 64 byte security descriptor entries up to the stream length. 

The Stream has the following stream properties:

* `name` of the stream is `SecurityDescriptor`
* `type_and_flags` is set as follows:
    * `REQUIRED` is set. `STRINGS` must be set if the stream has any entries with `permission_name_ref` set to a non-zero value
    * `stype` is `SECURITY`
    * All `stream_bits are 0


The security header has the following structure:

```
use types::uuid;
struct SecurityHeader : align(64) {
    object_owner: Uuid,
    security_flags: u64,
    implied_mode: SecurityMode,
    pad([u8; 39])
}
```

`object_owner` is either the UUID of the principal that owns the object, or the FULL (all 1s) UUID if there is no owner.

`security_flags` is set as follows:
* `0x0000_0000_0000_0001` (force_security): Overrides any other present security stream, such as `LegacySecurityDescriptor`
* `0x0000_0000_0000_0004` (imply_default_only): Use `implied_mode` only for permissions in the default domain.

All other bits are reserved and must not be set. Setting any bit is an error.

`implied_mode` is the default mode for most permissions (see [Permission Defaults]) that aren't referenced by a `SecurityEntry`. The `SecurityMode` enum is given below.

The remaining entries in the stream have the `SecurityEntry` structure:
```
use types::uuid;

enum SecurityMode : u8 {
    INHERIT = 0,
    FORBID = 1,
    DENY = 2,
    ALLOW = 3,
}

struct SecurityEntry : align(64) {
    principal: Uuid,
    permission_domain: Uuid,
    stream_ref: u64,
    permission_bits: u64,
    flags: u64,
    mode: SecurityMode,
    pad([u8;7])
}
```

`principal` is either the UUID of the principal or the entry, or the FULL (all 1s) UUID if this is a default entry.

`permission_domain` is the domain the permissions are taken from for the entry. The NIL UUID Represents the default permission domain. Which is defined by this document. 
`permission_bits` contains the actual bitmask of permissions, of which bits 0-62 have meaning defined by the domain. Bit 63 implies all defined bits for the domain.

`stream_ref` is either `0` or the index into the `Streams` stream that refers to the stream to which the permission pertains. If `stream_ref` is `0`, then the permission applies to the entire object unless overriden by a different permission option.

`mode` is the security mode to apply if the entry matches:
* INHERIT means that the mode is taken by a recursive search applied to the parent object, In the case of the implied mode, only the implied_mode of the security descriptor is inherited, not the mode of the specific action performed.
* FORBID means that the permission is denied. FORBID overrides previous ALLOWs (even if they are equally or less specific), and cannot be overriden by a later ALLOW or INHERIT,
* DENY means the permission is denied unless a prior ALLOW/INHERIT is more specific, or a later ALLOW or INHERIT is more specific,
* ALLOW means the permission is allowed unless a prior DENY/INHERIT is more specific, or a later DENY or INHERIT is more specific (or any FORBID applies)

`flags` contains flags about the permission entry:
* 0x0000_0000_0000_0001 (`order_mode`): Determines how entries applies when matched. If unset, search continues for a more specific entry. If set, the following applies:
    * For a DEFAULT (`principal` set to the FULL UUID) Or Wildcard (`permission_name`=`*`), only definite entries are continued to be searched for: subsequent DEFAULT and Wildcard entries are ignored,
    * For a specific entry (`principal` set to any other UUID, whether a user or a group), the search stops if the entry matches (even if it doesn't affect the permission mode). Only prior entries are considered. Note that this is effectively applies to any FORBID entry (which cannot be overriden) even if unset.

All other bits are reserved and must not be set.

##### Default permission Domain

The default permission domain (NIL or all 0s uuid) is for default actions. The following actions are defined. Other bits are unused and should be set to 0 and ignored:

* Bit 0 (`Read`): Read Access to the stream/object
* Bit 1 (`Write`): Write Access to the stream/object
* Bit 2 (`Execute`): Allow Code Execution
* Bit 3 (`SearchDirectory`): Allow path search through the stream/object
* Bit 12 (`AdvisoryLock`): Allows taking a non-mandatory (soft) lock
* Bit 13 (`MandatoryLock`): Allows establishing a mandatory (hard) lock
* Bit 14 (`BypassLock`): Allows overriding a mandatory (hard) lock, subject to lock priority.

Additionally, the following actions are defined for entries that set `stream_ref` to `0`. If `stream_ref` is non-zero, the bits are unused and are ignored.

* Bit 8 (`ReadSecurityDescriptor`): Allow reading `SECURITY` streams
* Bit 9 (`WriteSecurityDescriptor`): Allow writing to or creating `SECURITY` streams, other than the security header or fields that indicate object ownership
* Bit 10 (`TakeOwnership`): Allows writing to the security header or fields that indicate object ownership

##### Permission Check Algorithm

Let `A` be the Action bit being checked and D be the Action Domain being checked, 
`S` being the stream index being checked or 0 if the action is ambient, `P` be the primary principal of the actor, `G` be a (potentially-empty) implementation-defined set of secondary principals, let `O` be the object owner, and let `C` be the context parent object. Let `F` be an uninitialized security entry that is updated with the most specific found entry.

For each permission entry in order, perform the following actions:

1. Check the entry as follows, in any order, to determine if the entry matches. If it does not match, the entry is ignored:
    * The entry does not match if `principal` is not the FULL UUID, and is not equal to either `P` or any element of `G`,
    * The entry does not match if `permission_domain` is not `D`,
    * The entry does not match if neither bit 63 nor bit A of `permission_bits` are set,
    * The entry does not match if `stream_ref` is neither 0 nor `S`,
    * otherwise the entry matches.
2. If the entry matches, let `f` be that entry. If `F` is unset, let set it to `f` and continue to the next entry. Otherwise, choose either `F` or `f`, and set `F` to the choice, according to the following rules:
    1. If `order_mode` is set in `F`, and either `principal` in `f` is the FULL Uuid, or Bit A of `permission_bits` in `f` is not set, choose `F`,
    2. If `mode` in `f` is `FORBID`, choose `f`,
    3. If `principal` in `F` is the FULL Uuid, and the `principal` in `f` is not, choose `f`,
    4. If `principal` in `F` is not equal to `O`, and the `principal` in` f` is, choose `f`,
    5. If Bit A of `permission_bits` is not set in F but is set in `f`, choose `f`,
    6. If `mode` in `F` is `INHERIT`, and `mode` in `f` is not, choose `f`,
    7. Else, choose `F`.
3. If `order_mode` is set in `f`, and both `principal` is the FULL Uuid, and Bit A of `permission_bits` in `f` is set, stop the search here.
4. Stop the search if `mode` in `F` is `FORBID`.
5. Else, continue to the next permission entry.

After all entries are searched, perform the following steps, letting `M` be the `mode` field of `F`, if `F` is set:

1. If no entry is matched, the behaviour depends on the action domain, action, and the security header
    * If `D` is the default action domain:
        1. If `A` is `Execute` (Bit 2), the action is denied
        2. If `P` is equal to `O`, the action is allowed
        3. If `A` is between bits 8 and 10 inclusive, the action is denied
        4. Else, the action is checked by setting `M` to be `implied_mode` from the security header.
    * If `D` is any other domain:
        1. If `P` is equal to `O`, the action is allowed
        2. If `(A, D)` belongs to an implementation defined set of security-sensitive permissions, the action is denied.
        3. If the `imply_default_only` flag of the security header is set, the action is denied.
        4. Else, the action is checked by setting `M` to be `implied_mode` from the security header
2. If `M` is `ALLOW`, the action is allowed.
3. If `M` is `DENY` or `FORBID`, the action is denied.
4. If `M` is `INHERIT` and there is a context parent object, repeat the permission check for the action `A` in domain `D` against the object `C` (`S` is ignored), using the same `P` and `R` values. 
5. If there is no context parent object, the action is denied.

### Recommended Implementation for Operating Systems

#### Conventions Regarding Paths

Paths in this document are Byte strings that consist of a sequence of characters that do not contain `\0`. Paths can be split using a path separator components. In this document `/` is assumed to be the path separator. On some operating systems (such as Windows), this may be `\` instead.

For an absolute path, There may be an implementation defined filesystem prefix, which has a implementation-specific form (On Windows, this is the drive letter or a UNC prefix). After that, there is an implementation-defined filesystem path prefix indicated by the mount point (`/` for unix root filesystems). All components after the prefix and the path prefix must be valid UTF-8, and the previous segments may be any implementation-defined superset of UTF-8.

A component in a path can be split into a stem and a stream specifier. If a `$` appears in the path component, everything before the `$` is the stem, and everything after is the stream. If no `$` appears in a component, the entire component is the stem and there is no stream specifier. Empty components are disallowed and have implementation-specific semantics (for example, on unix, an empty component is discarded). Stream specifiers are only guaranteed to be supported on intermediate components

Paths can also be described in another character set, according to operating system conventions. 

Paths can also be relative, and start with neither a prefix nor a path prefix and immediately begin with a component. Whether or not these are supported are implementation-defined.

#### Minimum Set of Operations

The following basic access modes are supported:
```
enum AccessMode {
    Read,
    Write,
    Append,
}
```

`Read` means open the file for reading only. `Write` means to open the file for writing. `Append` means to open the for writing and position the file at the end of the stream. 
For unstructured streams (`UDATA` or `UMDATA`), `Write` truncates the file to position 0. Otherwise, it is equivalent to `Append`.

Every operating system that implements the filesystem must, at a minimum, provide the following operations with the `Mode` enum defined above. Note that the names and :

* `OpenStream(path, stream, mode)`: Opens a specified stream of a specified path using path resolution. Returns a handle to the stream
* `OpenFile(path, mode)`: Opens the specified path using path resolution. If `path` designates an object of type `File`, this is equivalent to `OpenStream(path, "FileData", mode)`. If `path` designates an object of type `Directory`, this is equivalent to `OpenStream(path, "DirectoryContent", mode)`. Other object types are conditionally supported.
* `CreateFile(path)`: Same as `OpenFile(path, Write)`, but only functions on `File` objects, and creates the file with a `FileData` stream if it is not found.
* `CreateDirectory(path)`: Same as `OpenFile(path, Write)`, but only functions on `Directory` objects, and creates the file with a `DirectoryContent` stream if it not found.

For each component of `path` other than the last, each of these operations performs a `Search` permission check on each path stream found. 
Additionally:

* `OpenStream` performs a `Read` permission check against the specified stream, if it is not a `SECURITY` stream for a `Read` access, and a `Write` permission check against the specified stream if it is not a `SECURITY` stream for `Write` or `Append` access. For a `SECURITY` stream, a `ReadSecurityDescriptor` or `WriteSecurityDescriptor` check respectively is performed instead.
* `OpenFile` performs the same permission check as the equivalent `OpenStream` operation
* `CreateFile` and `CreateDirectory` both perform a `Write` check against the stream of the last path component

## Security Considerations

Filesystems are an integral part of data and program security, particularily filesystems upon which critical system software or the kernel are loaded from. Correct implementation of the filesystem, particularily elements marked as security features, is critical to preserving system security. 

Implementations must correctly read and write all standard data structures. However, it should not assume those structures for validity or for security purposes, where the filesystem may be provided from untrusted data (mounted by an unprivileged user). Instead, all data structures must be validated as they are read against constraints imposed by the filesystem specification, physical disk and partition limits, and the implementation. Additionally, handling of certain stream types must be defensive against valid but malicious stream content.

## ABI Considerations

None

## Prior Art

* [FAT Filesystems](https://en.wikipedia.org/wiki/File_Allocation_Table)
* [NTFS](https://learn.microsoft.com/en-us/windows-server/storage/file-server/ntfs-overview)
* [Ext 2](https://wiki.osdev.org/Ext2), [Ext 3](https://wiki.osdev.org/Ext3), [Ext 4](https://wiki.osdev.org/Ext4)

## Future Direction

* Storage could be expanded in several ways:
  * Special storage types like file encryption, or compression can be added,
  * Copy-on-write support could be added for sharing large identical spans of data
* Additional stream types can be defined
  * For example, streams for more efficient directory lookup, file metadata, or additional default stream types
* Additional permission types can be defined
* Journaling support can be added

## References

### Normative References

* [RFC 9562: Universally Unique Identifiers](https://www.rfc-editor.org/rfc/rfc9562)


### Informative References
