# Efficient Process Initialization

## Summary

When processes are initialized, or a when program entry point is given control from the dynamic linker, certain information is passed into it. To faciliate 

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

### General Stack Frame Layout

In most system ABIs on Lilium, process initialization (performal when the kernel gives control either directly to the program entry point, or to the dynamic linker, or when the dynamic linker gives control to the program entry point) sets up a stack frame as follows (from first on the stack to last, usually highest address to lowest):
* An Auxillary Vector, terminated at the high address with a `0` word, which consists of an array of the `AuxvEnt` type defined below,
* An environment array, terminated at the high address with a `0` pointer, which consists of an array of pointers to `char*`s that point to strings of each environment (of format `<key>=<value>`)
* An argument array, terminated at the high address with a `0` pointer, which consists of an array of pointers to `char*`s that point to strings of each argument
* An word that contains the length of the argument array.

(The above is required to be implemented by all kernels and all dynamic linkers, unless the specific System ABI defines a generic format)

Prior to the above layout, the stack may contain data placed there by the kernel and/or the dynamic linker. The exact layout of this data is unspecified, and the data may be placed elsewhere.

Every string (including strings accessed via the auxillary vector) passed in program initialization contains valid UTF-8 and are null terminated.

### Auxillary Vector

The Auxillary Vector is used by the kernel to communicate information to the dynamic linker, and by both the kernel and the dynamic linker to communicate information to the USI implementation. The data in the auxillary vector may be accessed by programs directly, but this is usually unnecessary unless you are writing highly advanced system software.

The Auxillary Vector is an array of 2-word values, which consists of one of the following types (defined in the knums language):

```

union AuxvValue {
    at_ptr: *mut void,
    at_data: ulong,
    at_fn: fn()->void, // Or some other signature
    at_hdl: *handle Handle,
}

struct AuxvEnt {
    at_type: ulong,
    at_value: AuxvValue,
}
```

`at_type` of `0` (`AT_NULL`) is reserved. It does not appear in the auxillary vector (instead, a `0` word terminates the auxillary vector, but `at_value` is not guaranteed to be present).

`at_type` is either `AT_IGNORE` (1), in which case `at_value` is undefined, or a value specified below (which specifies which of `at_ptr`, `at_data`, or `at_fn` is used). The maximum `at_type` value used for userspace processes is 95. Additionally, it is guaranteed that 2 entries will have the same `at_type` value (other than `AT_IGNORE`).

```
const AT_NULL: ulong = 0;
const AT_IGNORE: ulong = 1;
const AT_PAGESZ: ulong = 6;
const AT_BASE: ulong = 7;
const AT_PLATFORM: ulong = 8;
const AT_ENTRY: ulong = 9;

const AT_SECURE: ulong = 23;
const AT_BASE_PLATFORM: ulong = 24;
const AT_RANDOM: ulong = 26;
const AT_EXECFN: ulong = 31;

const AT_LILIUM_INIT_HANDLES: ulong = 64;
const AT_LILIUM_INIT_HANDLES_LEN: ulong = 65;
const AT_LILIUM_EXECHDL: ulong = 66;
```

* `AT_PAGESZ`: Contains the page size of the architecture in `at_data`,
* `AT_ENTRY`: Contains the address of the entry point of the program.
* `AT_BASE`: Contains a pointer to the base of the program interpreter in `at_ptr`. When an executable is executed without the dynamic linker, `AT_BASE` instead contains the base address of the executable
* `AT_PLATFORM`: Contains a pointer to a Null Terminated string containing the architecture of the program being executed in `at_ptr`.
* `AT_SECURE`: `at_data` is set to `1` if the process is executed in a secure context (and thus both the dynamic linker and USI implementation may be able to trust some inputs, such as those in environment variables). Other values are reserved for future use (but values greater than `1` should be treated the same as `1` by programs and loaders).
* `AT_BASE_PLATFORM`: Contains the same string as `AT_PLATFORM` from the kernel. If the loader implements online emulation (for example, i686 on x86-64 or A32 on Aarch64), The `AT_BASE_PLATFORM` string set by the loader refers to the architecture of the system (which may be different from program in this case).
* `AT_RANDOM`: `at_ptr` points to 16 random bytes, with at least the following properties[^random-properties]:
    * There is a 2^-64 probability that any two processes accross any system will have the same value, and
    * If the bytes are consumed by a Cryptographic Hash Function and used to generate a value that is at least 128-bits long, the resulting hash has a 50% chance of containing at least 64 bits of enthropy.
* `AT_EXECFN`: `at_ptr` is a pointer to the absolute file path used to execute the file. Guaranteed to be present (when executing from the kernel) if `AT_LILIUM_EXECHDL` is not present.
* `AT_LILIUM_INIT_HANDLES`: `at_ptr` is a pointer to an array of handles valid on the initial thread that refer to the same objects with the same capabilities as were passed to the `init_handles` array of the `CreateProcess` call. By convention, `at_ptr[0]` contains the stdin handle, `at_ptr[1]` contains the stdout handle, and `at_ptr[2]` contains the stderr handle. If this element is present, then the next element is an `AT_LILIUM_INIT_HANDLES_LEN` element.
* `AT_LILIUM_INIT_HANDLES_LEN`: `at_data` is the length of the array passed by `AT_LILIUM_INIT_HANDLES`. If present, the preceeding element is the associated `AT_LILIUM_INIT_HANDLES` array.
* `AT_LILIUM_EXECHDL`: `at_hdl` is a handle that refers to the executable file on the filesystem. 

[^random-properties]: The combination of the two properties is such that the value in `AT_RANDOM` is suitable for seeding a CSPRNG.


### Enhanced Process Initialization

The following specification is optional, and may not be implemented. If it is implemented, it is done so in the following way. 

At process initialization, a system register is populated with a value indicated the Process Initialization Capabilities Word. Which register is used is system abi dependant. If this specification is not implemented, the register must contain the value 0 at process initialization. Otherwise, bit 0 is set to 1 and all other bits are reserved and must be set to `0` except to indicate conformance with a future initialization specification published by RFC. 

If this specification is implemented, two system registers (identified by the system abi) are used to communicate the address of the beginning of the environment array and the auxillary vector. If not implemented, these registers contain undefined values and cannot be relied upon. In any case, the stack layout above must still be provided (thus, a program that is not aware of this specification can find the auxv and envp pointers by scanning the stack frame until terminators are found, or by using argc to offset the argv array). 

#### `x86_64`

The Process Initialization Capabilities Word is `rax`. `r12` contains a pointer to `envp[0]`, `rbx` contains a pointer to `auxv[0]`.

#### `i686`

The Process Initialization Capabilities Word is `eax`. Additionally, `edx` is reserved to store high bits of the Capabilities Word. `esi` contains a pointer to `envp[0]`, `ebx` contains a pointer to `auxv[0]`

## Security Considerations

The auxiliary vector can present security issues if misused/set incorrectly by either the kernel, the loader, or the program:

* `AT_SECURE` is required to communicate if the process environment (which may include the resolution root address) cannot be relied upon for security. Loaders, the USI, and some system libraries must respect the value of `AT_SECURE` when set. Additionally, the Loader must propagate `AT_SECURE` accurately to the program, in order to ensure any security features are not tampered with. 
  * A loader or system library should not rely on the contents of any environment variable when `AT_SECURE` is set. Notably, this includes paths like `LD_PRELOAD` or `LD_LIBRARY_PATH` for determining objects to load
  * A loader or system library should load and execute, or otherwise rely on, any file not owned by `SYSTEM` (or the current primary principal, if that is not also `SYSTEM`), This includes known paths such as files in `/lib` or `/etc/ld-lilium.so.conf`
  * A system library or program should not rely on any file not owned by `SYSTEM` or the current primary principal if it uses that file for a security purposes, even if that file is located within a well-known system directory
  * Unlike other systems, the resolution root directory (called `chroot` on posix) is not necessarily reliable, as unprivileged code may establish a local resolution root
  * `AT_EXECFN` should likewise not be treated as reliable by loaders, as it may be a hardlink that changes its destination between being read by the kernel and by the loader. `AT_LILIUM_EXECHDL` *must* always be set by the kernel when it invokes an interpreter for an `AT_SECURE` binary
* `AT_RANDOM` may be used to initialize psuedo-random number generators that are relied upon for high quality randomness. The kernel and the loader (if it sets its own `AT_RANDOM` entry) should generate a high quality byte source for it (see the two requirements for the value). If the loader uses `AT_RANDOM` as a source of randomness (for example, to seed ASLR), it should generate a new `AT_RANDOM` value for the program. `AT_RANDOM` should only be used at process startup (typically by the USI).
  * Programs that use `AT_RANDOM` should not use it as a source of random data on its own, rather it should be used to seed a PRNG in userspace that is high enough quality for the use of the value. 
  * It is not guaranteed that `AT_RANDOM` is uniformly distributed on its own, only that it can produce ~64 bits of enthropy when passed through a Cryptographic Hash Algorithm, and that it is unlikely to be repeated (2^-64 probability).


## ABI Considerations

<!--
If this proposal impacts either the Userspace or System Application Binary Interface, 
-->

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