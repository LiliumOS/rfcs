# x86_64 System ABI

## Summary

The Interaction with the Lilium System Calls and userspace libraries has both an API and an ABI. This document describes the specific 

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

### Userspace ABI

Except as specified below, all system libraries in userspace obey the [x86-64 psABI]. 

The C `main` function is expected to obey this ABI, and may have the following signatures:

* `int main(void)`
* `int main(int argc, char** argv)` or `int main(long argc, char** argv)`
* `int main(int argc, char** argv, char** envp)` or `int main(long argc, char** argv, char** envp)`

(Note: It is recommended, but not strictly required, that all userspace code obey this ABI).

### System Call ABI

The System Call ABI uses a modify form of the calling convention from the [x86-64 psABI]. It uses the same type layouts as the Userspace ABI.

System Calls (defined by the kernel) are invoked by the `syscall` instruction. The system function number 

The following changes apply:
* On entry, `eax` contains the system function number. The top 32-bits of `rax` must be clear.
* On exit:
    * If the function returns `void`, both `rax` and `rdx` are undefined
    * If the function returns `SysResult`, `rax` contains the return value - the negative error number on error, positive or `0` if succesful, `rdx` is undefined.
    * If the function returns `SysResult2<T>` (`T` must be up to 8 bytes in size, and have class INTEGER), `rax` contains the error or `0` if successful, and `rdx` contains the value if successful (`rdx` is undefined if `rax` contains an error)
    * If the function returns any other type, that type must be at most one eightbyte and have class INTEGER. The value is in `rax` and `rdx` is undefined.
* There may be at most 6 eightbytes of parameters, each of either class MEMORY or INTEGER
* The fourth INTEGER eightbyte for parameters (including pointers for MEMORY types) is passed in `r10`, not in `rcx`. If the system function uses fewer than 4 eightbytes, `r10` is not used for the `syscall` (caller saved/volatile).
* Varargs are not supported.

#### System Function Number and Error Numbers

The System Function Number is a 32-bit value that describes the calling sequence. The bottom 12 bits contains the system function number within the subsystem, bits 12 through 27 (inclusive) contain the 16-bit subsystem number.  Bits 28-31 (inclusive) are reserved and contain `0`.

An Error Code is a negative value always (`-err` is ). `-err` encodes an 8-bit per-subsystem error code in the lower 8 bits and the 16-bit subsystem number. All other bits of `-err` are `0` (`1` for `err`).

### Lilium Specific psABI 

#### `long double`

On x86_64, `long double` is 8-bytes in size and has an alignment of 8.  It has a 53-bit Mantisa, 11-bit exponent, and an exponent bias of 1023[^1].

When classifying parameters/return values, `long double` is classified as a single eightbyte of class SSE, and `_Complex long double` is classed as two eightbytes each with class SSE (equivalent to `struct __complex_long_double { long double real; long double imm;}`).

`__fp80` may be defined by the toolchain, and has the standard definition and ABI. It is not Layout or ABI compatible with `long double`. 

[^1]: This is exactly the same as the `double` type. `long double` is not equivalent to `__fp80` on Lilium.

### x32/ILP32

The x32 ABI defined in the [x86-64 psABI] is not supported by either userspace system libraries or system functions.

## Security Considerations

Violation of the ABI Requirements can lead to undefined behaviour, including pointer access violations that can lead to memory corruption or invalid memory leads. 

## ABI Considerations

This document defines the ABI of both System Calls and Userspace Libraries on x86_64.

## Prior Art

* [x86-64 psABI]

## Future Direction

* x32 support.

## References

### Normative References

* [x86-64 psABI] Sys-V psABI for x86_64

### Informative References

<!--Include any documents cited to provide informative context only-->

[x86-64 psABI]: https://gitlab.com/x86-psABIs/x86-64-ABI