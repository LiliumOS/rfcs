# Subsystems

## Summary

Subsystems are the core of the Lilium Kernel System interface. System Calls and error constants are divided into discrete subsystems that allow a program to interact with discrete segments of the OS.

## Motivation

The Organization of an Operating System kernel can be complicated, particularily at the level of Userspace-Kernel interaction (System Calls, Constants), and most processes will not interact with most of the system calls defined by a given OS. Additionally, special purpose system calls may only need to be defined by the kernel itself if the kernel is needed to coordinate the system call numbers, and could be defined externally if coordination was achieved otherwise.

## Informative Explanation

The Lilium Kernel exposes its interfaces in several groups, known as subsystems. This allows for more efficient organization of both the kernel and userspace libraries, as well as extensibility. 

Subsystems can come from various places:
* They can be defined by the Lilium Project itself - including core subsystems,
* They can be provided by a kernel vendor, 
* They can be provided in third-party modules that you install.

Other than core subsystems, which are always available, the presence of any subsystem is not guaranteed - programs must query for their precence and what subsystem number to refer to them as and, if necessary, attempt loading them.

Both System Calls and Errors can be defined by subsystems. Other features may also be presented from subsystems. This allows for easy coordination of these critical constants, even when 3rd party modules are loaded.

## Normative Text

### Subsystem Number; System Functions; Error Codes

System Function are divided into subsystems, which are assigned a subsystem number. The subsystem number is a runtime-assigned value that is less than 2^20. Certain subsystem numbers are reserved, as defined below.

A System Function Number stores the subsystem number in the top 20 bits of the Value, and the Function within the subsystem in the bottom 12 bits, forming a 32-bit value. Raw Error Numbers are also divided by subsystem, using the bottom 8 bits to store the per-subsystem error number, the next 20 bits is the System Function Number, and the top 4 bits are reserved. The Error Code returned from System Functions is the negative of the Raw Error Number.

The Subsystem definition assigns the set of System Functions within the subsystem, the Function Number, the Signature, the expected name, and behaviour (including return codes). 
The Subsystem definition likewise assigns the Raw Error Numbers within a subsystem.

### Subsystem Loading/Sources

Subsystems fall into three loading categories:
* Core,
* Predefined, or
* Loadable.

Core Subsystems are critical subsystems that are always available to all processes running on a given version of Lilium. These subsystems have fixed reserved numbers that are constants and thus can be relied upon and/or hardcoded into processes. There are few Core Subsystems that divide standard tasks of a program on the Lilium Operating System, interact with readily available elements of the Operating System, or perform management tasks that may not be possible with loadable subsystems. 
Core subsystems are almost always built into the kernel image and primarily exist for organizational purposes.

Predefined Subsystems are subsystems made available by a kernel that are not core subsystems. These are usually, but not necessarily, built into the kernel image itself, and are made available to all processes, but do not necessarily have a fixed subsystem number. APIs provided by core subsystems are provided to query the subsystem number of predefined (and loaded) subsystems. Generally different processes will observe the same subsystem number, but this is not guaranteed, and different kernel boots may observe different subsystem numbers (even if neither the kernel nor the list of predefined subsystems changes).

Loadable Subsystems are subsystems that are not made available by default and must be loaded explicitly. Loadable subsystems do not have fixed subsystem numbers, and they are assigned one when the kernel loads them in a process.

Kernels may impose an upper limit on the number of Loadable Subsystems which may be loaded in a given process.

Subsystems may be provided as one of three source categories:
* Standard,
* Extension, or
* Module.

Standard Subsystems are defined by the Lilium Project by RFC. They are considered widely available but may not always be available on all kernels.

Extension Subsystems are defined by the kernel vendor. Multiple kernel vendors may collaborate to define equivalent extension subsystems.
Module or 3rd Party subsystems are special ELF shared objects loaded into the kernel runtime. The support for Module subsystems is per-kernel, and Modules defined and built against one kernel are not guaranteed to be supported by any other, or by different versions of the same kernel.

Other than Core Subsystems, which are always Standard subsystems, Subsystems from any of these categories may be either Predefined or Loadable.

Some Standard Subsystems may be Experimental. Experimental Subsystems are not fixed and may evolve in incompatible ways before they are finalized. Generally, Experimental Subsystems should be Loadable, rather than predefined.

### Core Subsystems

Core Subsystems are a specifically designated set of Standard Subsystems, with a predefined subsystem number, which are guaranteed to be available on a given Lilium Kernel Version. Core Subsystems are defined by RFC (like all standard subsystems). These subsystems define critical system interfaces, vocabulary types and errors, are are generally more widely applicable, or are commonly used by most general-purpose programs.

### USI Libraries

Standard Subsystems are generally called through a usi library which contains its name prefixed with `usi-`. For example, the `foo` subsystem would be called through `usi-foo`. The library `usi` will link against routines for all core subsystems, in addition to support routines for the compiler, dynamic linker, or Structure Exception Unwinding.

### Options

Many subsystems define "Option" Parameters. These represent extension points to system functions that may be extended in ABI compatible ways. 
Different Options may be defined:

* By the definition of the subsystem,
* By future versions of the same subsystem,
* By different kernel or subsystem vendors,
* By other subsystems,
* By other kernel modules (that aren't subsystems),
* In some cases, in userspace.

Every option type begins with a 32-byte aligned data structure defined as if the following:
```
struct ExtendedOptionHead : align(32) {
    ty: Uuid,
    flags: u32,
    pad([u32; 3])
}

const OPTION_FLAG_IGNORE: u32 = 0x0000_0001;
```

`ty` refers to a unique identifier that identifies the option. The NIL UUID is reserved. Otherwise, any valid UUID may be used (See [IETF RFC 9562] for accepted processes for generating UUIDS - note that the use of v8 UUIDs is not recommended).

`flags` contains flags for defining the behaviour of the option. The top 16 bits is reserved per type. 
Flag `OPTION_FLAG_IGNORE` controls the behaviour of unrecognized option values. All other flags must be 0.

When the `flags` sets any reserved bits, or when the padding is non-zero, the system function should return an error. The error `INVALID_OPTION` is defined in the base subsystem for this purpose.
When `ty` is not recognized by the subsystem or by any other source, the flag `OPTION_FLAG_IGNORE` controls the behaviour. If this flag is set, the option shall be ignored by the subsystem implementation. Otherwise, the subsystem should error with `INVALID_OPTION`. Any deviation of a should qualifier in this paragraph shall be only as documented by the subsystem definition.

## Security Considerations

Loading subsystems may allow executing arbitrary code or establishing system calls with security holes. Kernels should be cautious about allowing arbitrary subsystems to be loaded by unprivileged processes.

## ABI Considerations

This defines the precise ABI for system functions in terms of system function number and `SysResult` error codes. and how they relate to the set of loaded subsystems. Additionally, it defines the layout (and thus ABI on each platform) of a common vocabulary type.

## Prior Art

* Windows uses Subsystems to characterize the behaviour of programs, how they ought to be loaded, and what modules ought to be loaded into the default process image.

## Future Direction

* The set of standard subsystems may be defined
* The `ExtenedOptionHead` type may be extended with new flags or with new fields.

## References

### Normative References

### Informative References

* [IETF RFC 9562]

[IETF RFC 9562]: https://www.rfc-editor.org/rfc/rfc9562.html