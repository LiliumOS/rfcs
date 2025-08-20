# RFC Template

## Summary

Lilium uses an extended version of the ELF format for both userspace executables and dynamic linking/loading, as well as for kernel modules.

## Motivation

The Lilium OS requires an executable format for executing 

## Informative Explanation

<!--Provide an informative explanation of proposal. 
This is intended to be read by someone who wishes to understand the proposal but may not have advanced technical background.
This section is intended for:
* People using the Lilium Operating System as a Software Developer
* People looking to understand the Lilium Operating System
* People looking to understand the Lilium Project as a whole

This section is not normative-->

## Normative Text

### Executable/Linkable Format

Executable Files and Shared Object modules on Lilium are defined by the [generic-abi], with extensions defined as below.

### OS ABI

Lilium supports the use of `OSABI_SYSV` (0) and `OSABI_LILIUM` (TODO) ELF Files. Both are treated identically if 
`OSABI_LILIUM` should be preferred if the binary contains any of the extensions used herein, except that for compatibility with the GNU and LLVM toolchains, the use of `DT_GNU_HASH` together with `DT_HASH` is supported on `OSABI_SYSV`, as is the use of `PT_GNU_STACK`.

### Constraints on Executable/Loadable Files

The following constraints are placed on any executable (including position independant executables) and shared objects. No support is provided for any ELF File that violates these constraints:

* Any Executable that uses the default interpreter (`/lib/ld-lilium-<arch>.so`) must be position independant. 
* Any External Dynamic Symbol must only contain valid UTF-8 bytes.
* `PF_W | PF_X` segments cannot be loaded. Attempting to load such a segment results in an error.
    * Additionally, a `PF_W` segment cannot occupy the same page as a `PF_X` segment. If a `PF_W` segment immediately follows a `PF_X` segment (or vice versa), the second segment shall begin no earlier than the next 4096 byte boundary (note that this is trivially true if the first segment ends exactly on this boundary)
* `PT_TLS` segments cannot be executable. Thread-local program code should be mapped into memory by the program, using thread-private memory maps if necessary.
* The `pt_align` of any PT_LOAD segment must not be less than 4096.

### OS Specific Program Header types

The ELF Specification defines an OS Specific Range beginning at `PT_LOOS` (0x60000000), and ending at `PT_HIOS` (0x6FFFFFFF). 
We assign this range for Lilium toolchains and ELF loaders as follows:

| Name                    | Value        |
|:-----------------------:|--------------|
| `PT_LOOS`               | `0x60000000` |
| `PT_GNU_EH_FRAME`       | `0x6474e550` |
| `PT_GNU_STACK`          | `0x6474e551` |
| `PT_GNU_RELRO`          | `0x6474e552` |
| `PT_LILIUM_LOKERNEL`    | `0x6FE00000` |
| `PT_LILIUM_HIKERNEL`    | `0x6FEFFFFF` |
| `PT_HIOS`               | `0x6FFFFFFF` |

The behaviour of each segment is described in subsections below.

#### Exception Handling.

`PT_GNU_EH_FRAME` is recognized for compatibility with GNU and LLVM toolchains. It defines the exception handling table for the module. 

The format is as defined for `.eh_frame_hdr` as specified by [LSB 5.0 Core (.eh_frame)].

#### Stack Description

`PT_GNU_STACK` is recognized for compatibility with GNU and LLVM toolchains. The program header has no behaviour on Lilium, other than to be validated as follows:

* `p_memsz` must be 0
* `p_flags` must not set `PF_X`

#### Dynamic Relocation Protection

`PT_GNU_RELRO` is recognized for compatibility with GNU and LLVM toolchains. If it the program header is present, the dynamic linker may disable write access to any memory region that resides within the segment after applying dynamic relocations. If this behaviour is implemented, the dynamic linker must act as though the module being loaded defines the `DT_NOW` dynamic tag. 
It is deprecated to have a `PT_GNU_RELRO` header without one of the following in the dynamic section:
* a `DT_NOW` dyanmic tag,
* a `DT_FLAGS` dynamic tag that sets `DF_NOW`,
* A `DT_FLAGS_1` dynamic tag that sets `DF_1_NOW`

#### Kernel Specific Range

The range of tags starting with `PT_LILIUM_LOKERNEL` (0x6FE00000) and ending with `PT_LILIUM_HIKERNEL` (0x6FEFFFFF) is reserved for use by the kernel and by kernel modules. These tags will be defined in a future RFC. Userspace loaders, including the kernel loader, must not load any module that defines one of these program headers.

### OS Specific Section Types

The ELF Specification defines an OS Specific Range beginning at `SHT_LOOS` (0x60000000), and ending at `SHT_HIOS` (0x6FFFFFFF). 

The following sections are used by static linkers for Lilium

| Name                           | Value        |
|:------------------------------:|--------------|
| `SHT_LOOS`                     | `0x60000000` |
| `SHT_LILIUM_REQUIRE_SUBSYSTEMS`| `0x60000000` |
| `SHT_LILIUM_LOKERNEL`          | `0x6FE00000` |
| `SHT_LILIUM_HIKERNEL`          | `0x6FEFFFFF` |
| `SHT_HIOS`                     | `0x6FFFFFFF` |

#### `.lilium.require-subsystems`

The special section `.lilium.require-subsystems` (of type `SHT_LILIUM_REQUIRE_SUBSYSTEMS`) may provide a mechanism for communicating to the dynamic loader that a specified subsystem is required to be loaded. The Dynamic Loader will then make appropriate system calls when loading the module (this occurs after calling `DT_PREINIT_ARRAY` entries in an executable, but prior to calling `DT_INIT_ARRAY` entries). See [`DT_LILIUM_REQUIRE_SUBSYSTEMS`] for the behavior of dynamic linkers that process this section. Both `SHF_OS_NONCONFORMING` and `SHF_ALLOC` must be set.

The `sh_link` entry is a section index that refers to a `SHT_STRTAB` section that is `SHF_ALLOC`. `sh_entsize` is the length of the entries. For `ELFCLASS32`, only a `sh_entsize` of 4 is supported. For `ELFCLASS64`, `sh_entsize` may be 4 or 8. The section contains an array of offsets (byte offsets) into the string table mentioned by `sh_link`. If the entry is not `0` (an empty string), then the string is a name of a subsystem to pass to `OpenSubsystem`. 

When a `SHT_LILIUM_REQUIRE_SUBSYSTEMS` section is processed by a link editor, the entries must be adjusted so that they have the correct offsets after string tables are concatenated. When linking `.lilium.require-subsystems` into an executable or shared objects, this is the dynamic string table that will be pointed to by `DT_STRTAB`. 

How other sections of type `SHT_LILIUM_REQUIRE_SUBYSTEMS` are handled during linking is not specified. The link editor may include them in `DT_LILIUM_REQUIRES_SUBSYSTEMS`. 

### OS Specific Dynamic Tags

| Name                           | Value        | `d_un`  | Executable | Shared Object |
|:------------------------------:|--------------|---------|------------|---------------|
| `DT_LOOS`                      | `0x6000000D` | N/A     | N/A        | N/A           |
| `DT_LILIUM_HASHENT`            | `0x6000000D` | `d_val` | Optional   | Optional      |
| `DT_LILIUM_HASH`               | `0x6000000E` | `d_ptr` | Optional   | Optional      |
|`DT_LILIUM_REQUIRE_SUBSYSTEMSSZ`| `0x6000000F` | `d_val` | Optional   | Optional      |
|`DT_LILIUM_REQUIRE_SUBSYSTEMS`  | `0x60000010` | `d_ptr` | Optional   | Optional      |
| `DT_LILIUM_LOKERNEL`           | `0x6FE00000` | N/A     | Disallowed | N/A           |
| `DT_LILIUM_HIKERNEL`           | `0x6FEFFFFF` | N/A     | Disallowed | N/A           |
| `DT_HIOS`                      | `0x6FFFF000` | N/A     | N/A        | N/A           |
| `DT_GNU_HASH`                  | `0x6FFFFEF5` | `d_ptr` | Optional   | Optional      |

#### `DT_LILIUM_HASH`

`DT_LILIUM_HASH` and `DT_LILIUM_HASHENT` are defined as reserved for future use. They describe an alternative to `DT_HASH` for dynamic symbol table. 

### `DT_LILIUM_REQUIRE_SUSBYSTEMS`

The `DT_LILIUM_REQUIRE_SUBSYSTEMS` contains a pointer to an array of offsets into the `DT_STRTAB`, with `DT_LILIUM_REQUIRE_SUBSYSTEMSSZ` defining the total size of the array in bytes. 
On ELFCLASS64 only, the top 2 bits encodes the entry size, where `00` is size 4, and `01` is size 8, with other size values being reserved. On ELFCLASS32, only entry size is encoded.

When loading a module with `DT_LILIUM_REQUIRE_SUBSYSTEMS`, the dynamic linker will, for each entry, try to load the corresponding kernel subsystem as though by calling `OpenSubsystem`. If an error occurs, the dynamic linker will refuse to load the module (and may result in a fatal error the loading function returning an error result). The dynamic loader may elide a particular call to `OpenSubsystem` if it knows the subsystem is already loaded (for example, by keeping a cache of loaded subsystems, or when a named subsystem is known to always be loaded on the current kernel).

The subsystems are loaded by the dynamic linker before any code in the module is executed, except that when it is attached to an executable, `DT_PREINIT_ARRAY` elements are executed prior to `DT_LILIUM_REQUIRE_SUBSYSTEMS`.

### Kernel Module Range

The kernel module range is between `DT_LILIUM_LOKERNEL` and `DT_LILIUM_HIKERNEL`. There is currently no definition for the tags in this range, except that they obey the `DT_ENCODING` rule.

Userspace dynamic modules must either ignore tags in this range, or error upon loading a module with these tags.

### `DT_GNU_HASH` {#dt-gnu-hash}

[`DT_GNU_HASH`]: #dt-gnu-hash

`DT_GNU_HASH` is supported as an alternative for `DT_HASH`, defined for compatibility with preexisting GNU and LLVM Toolchains. `DT_GNU

> **NOTE:**
> THe GNU_HASH format is almost completely undocumented, and

#### Hash Algorithm

The algorithm used for this section is described by the following rust function:

```rust
pub fn gnu_hash(name: &str) -> u32 {
    let mut v = 5381u32;
    for b in name.bytes() {
        v = v.wrapping_shl(5).wrapping_add(v).wrapping_add(i as u32);
    }
    v
}
```

#### Symbol Lookup

The lookup algorithm is roughly as follows: (Adapted partially from <https://flapenguin.me/elf-dt-gnu-hash>, but contains additional info, and may yet still be incomplete)
The structure of the whole [`DT_GNU_HASH`] tag is as follows:

```rust
#[repr(C)]
pub struct ElfGnuHashTable {
   pub head: ElfGnuHashHeader,
   pub bloom: [usize; head.bloom_size], // `usize` is the appropriate `ElfX_Size` type - or the size type for the Elf Class
   pub buckets: [u32; head.nbucket],
   pub chain: [u32],
}
```

To lookup a symbol with name `foo`, we compute the `hash` of `foo` using [`hash::gnu_hash(foo)`][crate::resolver::hash::gnu_hash].
We can then check this hash against the `bloom` filter as follows:

```rust
    let bloom_ent = (hash / usize::BITS) % head.bloom_size; // Again, this is actually `ElfX_Size` where `X` is the current ELFCLASS
    let bloom_pos1 = hash % usize::BITS;
    let bloom_pos2 = (hash >> head.bloom_shift) % usize::BITS; //
    let bloom_val = head.bloom[bloom_ent as usize];
    (bloom_val & (1 << bloom_pos1)) && (bloom_val & (1 << bloom_pos2))
```

Note that testing both bits will not guarantee that the hash is in the table if true, but if either bit is false, the symbol is definitely not in the table.
We then take the symbol index to start checking from by `buckets[hash % head.nbuckets]`.
This may be less than `head.symoffset`. If it is `0` then the symbol is absent from the table.
> It is not yet know what the behaviour of symbols in `1..head.symoffset` is, or if these values are allowed to appear.
> Implementations are recommended to treat these values the same as `0`

The chain index is taken by subtracting `head.symoffset` from this index. The top 31 bits of this chain entry is the top 31 bits of the hash value of the corresponding symbol.
The hash is compared ignoring the lower bit. If they match, the symbol index can be looked up in the dynamic symbol table and a name comparison can be done.
If either the hash comparison or the name comparison fails, the least significant bit of the chain determines the following behavior:

* If the last bit is `0`, the next symbol in the bucket can be checked. This is the subsequent entry in both in the symbol table and in the chain array (unlike `DT_HASH`, a pointer is not followed).
* If the last bit is `1`, this is the last entry in the current bucket, and the symbol is not present in the table.\

#### Symbol Table Format

The support the ordering requirements set by the chain array and the bucket array, the following constraints are placed on the dynamic symbol table (accessible from `DT_SYMTAB`):

* All symbols in the hashtable must be contiguous,
* The layout of the symbols that belong to the hashtable in the symbol table exactly corresponds to the layout of the chain array, in particular:
   * The symbols are grouped by which bucket entry they fall into and,
   * They are ordered such that the corresponding entry in the chain array has the value corresponding to the hash of the symbol name.
Note that the requirement only applies to symbols that belong in the hashtable (which are all symbols starting from `head.symoffset`).

## Security Considerations

Loading ELF Files can present a number of security risks. Failure to correctly load an ELF File can lead to memory safety issues, arbitrary code execution, and security vulnerabilities. 

Additionally, allowing code execution from memory regions that are often used for arbitrary data (including data from the user, or data from remote systems) can be the source of shell code vulnerabilities. ELF Loaders, including the kernel loader, on Lilium must not produce a Writable and Executable memory region, including for the call stack, and should reject any request to do so from a loaded binary. 

ELF Loaders, including the kernel loader, and especially the loader for any kernel modules, should use effective exploit mitigation techniques, such as ASLR, when possible to do so. Userspace ELF interpreters should not load executables or shared objects loaded at runtime (either via DT_NEEDED or via runtime loading operations) at a consistent base address, and kernels should not load position independant executables at a consistent base address. 
Position-dependant executables must be loaded with a base address of 0, as they may depend on that base address internally. Use of position-dependant executables on Lilium is unsupported for those that use the default interpreter, and are deprecated when no interpreter is used.

As an exploit mitigation, it is recommended to implement support for the `PT_GNU_RELRO` program header.

Kernel Modules are shared objects loaded in the context of the kernel. The interface that the kernel exposes is not a public part of the API and may depend on the kernel and kernel version. Users creating kernel modules should use appropriate techniques for validating support (Not defined in this RFC), and kernels should validate those techniques appropriately. Users loading kernel modules should take care to ensure only trusted modules are loaded, as loading an untrusted kernel module can present extraordinary risks to system security, privacy, stability, and performance. This includes verifying that the module is designed to function on the in-use kernel.


## ABI Considerations

The ELF Format, and its constraints, forms a part of the OS-specific ABI of Lilium. 

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

* [generic-abi] The System V Generic ABI
* [LSB 5.0 Core (.eh_frame)] the Exception Handler Frame Specification for Linux Standards Base 5.0

[LSB 5.0 Core (.eh_frame)]: https://refspecs.linuxfoundation.org/LSB_5.0.0/LSB-Core-generic/LSB-Core-generic/ehframechpt.html
[generic-abi]: https://www.sco.com/developers/gabi/latest/contents.html

### Informative References

<!--Include any documents cited to provide informative context only-->