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

### Auxillary Vector

The Auxillary Vector is used by the kernel to communicate information to the dynamic linker, and by both the kernel and the dynamic linker to communicate information to the USI implementation. The data in the auxillary vector may be accessed by programs directly, but this is usually unnecessary unless you are writing highly advanced system software.

The Auxillary Vector is an array of 2-word values, which consists of one of the following types (defined in the knums language):

```

union AuxvValue {
    at_ptr: *mut void,
    at_data: ulong,
    at_fn: fn()->void, // Or some other signature
}

struct AuxvEnt {
    at_type: ulong,
    at_value: AuxvValue,
}
```

`at_type` of `0` (`AT_NULL`) is reserved. It does not appear in the auxillary vector (instead, a `0` word terminates the auxillary vector, but `at_value` is not guaranteed to be present).

`at_type` is either `AT_IGNORE` (1), in which case `at_value` is undefined, or a value specified below (which specifies which of `at_ptr`, `at_data`, or `at_fn` is used)

```
const AT_NULL: ulong = 0;
const AT_IGNORE: ulong = 1;
const AT_PAGESZ: ulong = 6;
const AT_BASE: ulong = 7;
const AT_PLATFORM: ulong = 8;

const AT_SECURE: ulong = 23;
const AT_BASE_PLATFORM: ulong = 24;
const AT_RANDOM: ulong = 26;

const AT_LILIUM_INIT_HANDLES: ulong = 64;
const AT_LILIUM_INIT_HANDLES_LEN: ulong = 65;
const AT_LILIUM_EXECHDL: ulong = 66;
```

* `AT_PAGESZ`: Contains the page size of the architecture in `at_data`,
* `AT_BASE`: Contains a pointer to the base of the program interpreter in `at_ptr`. When an executable is executed without the dynamic linker, `AT_BASE` instead contains the base address of the executable
* `AT_PLATFORM`: Contains a pointer to a string containing the architecture of the program being executed in `at_ptr`.
* `AT_SECURE`: `at_data` is set to `1` if the process is executed in a secure context (and thus both the dynamic linker and USI implementation may be able to trust some inputs, such as those in environment variables)

### Efficient Data Lookup

The following specification is optional, and may not be implemented. If it is implemented, it is done so in the following way. 

At process initialization, a system register is populated with a value indicated the Process Initialization Capabilities Word. Which register is used is system abi dependant. If this specification is not implemented, the register must contain the value 0 at process initialization. Otherwise, bit 0 is set to 1 and all other bits are reserved and must be set to `0` except to indicate conformance with a future initialization specification published by RFC. 

If this specification is implemented, two system registers (identified by the system abi) are used to communicate the address of the beginning of the environment array and the auxillary vector. If not implemented, these registers contain undefined values and cannot be relied upon. In any case, the stack layout above must still be provided (thus, a program that is not aware of this specification can find the auxv and envp pointers by scanning the stack frame until terminators are found, or by using argc to offset the argv array). 

#### `x86_64`

The Process Initialization Capabilities Word is `rax`. `r12` contains a pointer to `envp[0]`, `rbx` contains a pointer to `auxv[0]`.

#### `i686`

The Process Initialization Capabilities Word is `eax`. Additionally, `edx` is reserved to store high bits of the Capabilities Word. `esi` contains a pointer to `envp[0]`, `ebx` contains a pointer to `auxv[0]`

## Security Considerations

<!--If the proposal requires users and/or implementors to take anything into consideration for security reasons, document this here.-->

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