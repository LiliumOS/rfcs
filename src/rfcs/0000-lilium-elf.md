# RFC Template

## Summary

Lilium uses an extended version of the ELF format for both userspace executables and dynamic linking/loading, as well as kernel modules.

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