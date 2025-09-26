# Target Names

## Summary

We define canonical names for Lilium targets on toolchain compilers.

## Motivation

Target tuples have historically been a point of contention between various toolchain authors, with different toolchains asserting the names they feel are most accurate and descriptive. This leads to avoidable discrepancies when multiple toolchains are needed in a project, for instance using a GCC autoconf script but then feeding into an LLVM-derived toolchain, such as Clang or rustc. At the Lilium project, we ran into this issue, and we decided to try to fix it, hence the RFC


## Informative Explanation

When toolchains are built to cross-compile, it uses a 3 or 4 component target name to determine what system it should generate code for. This includes knowing how to find library files and header files, the machine code to emit, or the object format used for linking or execution. Other information can be derived from the target name (such as ABI or whether or not dynamic linking is supported). Programs may also use this information to determine what interfaces are available on the system, and how it should interact with the OS.

The canonical form of a target has 3 or 4 components, the last two of which delineate the operation system and environment for compilation. Lilium uses `lilium` as its canonical kernel name for the third component, and the name of the USI implementation is used for the environment. When compiling for the kernel, the environment `kernel` is used. This allows toolchains and programs to know when it is building for Lilium and for what USI it is building.

## Normative Text

### General Style of a target name

A canonical target name is made up of three primary components, `<arch>-<vendor>-<system>` where `system` is either a single component on its own, or an `<kernel>-<env>` pair. For general purposes, we define only the `system` part of the tuple, and the other components are primarily defined by their respective authorities.

### OS Name

The canonical operating system/kernel name is `lilium`. The kernel component shall appear together with an environment that specifies the userspace implementation. 
In the future the kernel component may be decorated with a version number, but this is not currently defined.

### Environment

The environment shall specify the userspace implementation targetted by the toolchain, or use one of the names below. The name `init` is reserved for future use.

* `std` - the standard USI provided by the Lilium project,
* `kernel` - target for use in building kernel mode artifacts, such as kernel modules and the Lilium kernel itself,
* `none`/`elf` - No userspace/direct system calls only.

Additionally, we reserve the name of any of these appended with `x32` for ILP32 interfaces on x86-64 and other targets.

### Vendor

The vendor is not specified by the Lilium Project. However, for consistency between toolchains, it is recommended that the vendor `pc` be used for x86_64 and x86-32 (i686/i786) targets.

### x86 architecture names

The use of i686 as a target name is recommended to refer the Intel 686/P6 Microarchitecture (beginning with the Pentium Pro). The i786 architecture name should be instead used when a Pentium 4 Minimum is required. This is consistent with the results of reading `arch_version` on 32-bit x86 targets.

### Non-canonical Targets

Non-canonical targets may be supported by toolchains in different form than canonical ones, including by using alias names for various components, or omitting otherwise required components. Lilium does not support any alias names for the kernel name or any of the standard environments. Additionally, it does not support omitting either the kernel name or an environment. Lilium targets may omit the vendor field or use aliases for either the vendor or architecture component as supported by the toolchain. These aliases are not defined herein.

## Security Considerations

None.

## ABI Considerations

Artifacts built against different environment components are not expected to be ABI compatible and linking such objects together may not function properly.

## Prior Art

## Future Direction

* Future environment names may be assigned for additional compilation configurations,
* The Lilium project may coordinate environment names between USI implementations,
* The `init` environment may be defined in the future

## References

### Normative References

None
