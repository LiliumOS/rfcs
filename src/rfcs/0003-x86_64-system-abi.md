# x86_64 System ABI

## Summary

The Interaction with the Lilium System Calls and userspace libraries has both an API and an ABI. This document describes the ABI of those parts of Lilium on x86_64 specifically.

## Motivation

Toolchains compiling to Lilium on x86_64, as well as hand-written assembly, need to be able to produce code that can interact with system libraries as well as system functions, using standardized conventions.

## Informative Explanation

The x86_64 Architecture is one of the primary architectures targetted by the Lilium Operating System. To esnure compatibility of compiled programs, we define an set of conventions for the ABI, 
The conventions consists of the Calling Convention for both Userspace and System Calls from the Kernel, as well as layouts and representations of language types and certain vocabulary types in the standard library,

## Normative Text

### Userspace ABI

Except as specified below, all system libraries in userspace obey the [x86-64 psABI]. 

The C `main` function, and function pointers passed to system libraries, is expected to obey this ABI, and may have the following signatures:

* `int main(void)`
* `int main(int argc, char** argv)` or `int main(long argc, char** argv)`
* `int main(int argc, char** argv, char** envp)` or `int main(long argc, char** argv, char** envp)`

Regardless of whether `argc` is defined as `int` or `long`, it will contain the same value, unless the value cannot be represented as `int`. 

(Note: Signatures using `long argc` are non-standard and are not portable)

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

An Error Code is a negative value always (`-err` is the error value). `-err` encodes an 8-bit per-subsystem error code in the lower 8 bits and the 16-bit subsystem number. All other bits of `-err` are `0` (`1` for `err`).

### Lilium Specific psABI 

#### `long double`

On x86_64, `long double` is 8-bytes in size and has an alignment of 8.  It has a 53-bit Mantisa, 11-bit exponent, and an exponent bias of 1023[^1].

When classifying parameters/return values, `long double` is classified as a single eightbyte of class SSE, and `_Complex long double` is classed as two eightbytes each with class SSE (equivalent to `struct __complex_long_double { long double real; long double imm;}`).

`__fp80` may be defined by the toolchain, and has the standard definition and ABI. It is not Layout or ABI compatible with `long double`. 

[^1]: This is exactly the same as the `double` type. `long double` is not equivalent to `__fp80` on Lilium.

#### `fenv_t`

The `fenv_t` type defined in the header `<fenv.h>` is a single 32-bit value with class INTEGER. The contents of the bits are equivalent to the layout of the `mxcsr` register. `fegetenv` stores the register into the memory pointed to by its parameter, `fesetenv` loads the register from its parameter.

### x32/ILP32

The x32 ABI defined in the [x86-64 psABI] is not supported by either userspace system libraries or system functions. 
Non-standard userspace system libraries may implement support, but require a special program loader, and may require additional support to ensure pointers (especially handles) are restricted on 32-bit. Programs written or compiled to expect x32 support cannot make use of direct system calls without adjusting ABI on the caller side.

## Security Considerations

Violation of the ABI Requirements can lead to undefined behaviour, including pointer access violations that can lead to memory corruption or invalid memory leaks. 
Toolchains, users writing manual assembly code, and implementors of userspace system libraries must take care in matching the ABI to avoid security vulnerabilities arising due to imrpoper ABI handling.

## ABI Considerations

This document defines the ABI of both System Calls and Userspace Libraries on x86_64.

## Prior Art

* [x86-64 psABI]

## Alternatives

* The [x86-64 psABI] can be adopted verbatim without changes:
  * This would require `long double` to be 128-bit and use x87 ABI, which requires additional system library support without substantial benefit, conversion costs, and memory usage.
  * Additionally, the system call ABI must still be modified, as `rcx` (used as the 4th parameter by the psABI) is used by the `syscall` instruction to store the return address
* The [Win64 ABI] could be used instead
  * Win64 would be more constraining, and also requires modification to the ABI for system calls (as `rcx` used for the first parameter needs to be switched to `r10`)
  * Further, the ABI would need additional parameters to support a max of 6 eightbytes of parameters, and there would be limited support for `SysResult2<T>`.


## Future Direction

* x32 support.

## References

### Normative References

* [x86-64 psABI] Sys-V psABI for x86_64

### Informative References

* [Win64 ABI] the Windows ABI for x86-64.

[x86-64 psABI]: https://gitlab.com/x86-psABIs/x86-64-ABI
[Win64 ABI]: https://learn.microsoft.com/en-us/cpp/build/x64-calling-convention?view=msvc-170