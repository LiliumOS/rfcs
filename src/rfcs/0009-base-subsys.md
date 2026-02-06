# Base Subsystem

## Summary

The Base Subsystem is one of Four core subsystems in Lilium. It represents fundamental and general operations for all programs, that generally 

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

### Overview

The base subsystem is comprised of the following modules:

* `base::types::str`
* `base::types::slice`
* `base::option`
* `base::types`
* `base::error`
* `base::hdl`
* `base::ctx`
* `base::info`
* `base::config`
* `base::exception`
* `base::syscall`

### `base::types`

The `base::types` module is a re-export module. It reexports all definitions in `base::types::str`, `base::types::slice`, and `base::option`


```
inline use base::types::str;
inline use base::types::slice;
inline use base::option;
```

### `base::types::str`

The `base::types::str` module defines types exposed for strings.

#### `KStr`

`KStr` is a structure type that is used to pass text to the kernel. 

`buf` is a pointer to a UTF-8 string that is **not null terminated**. `len` is the length of the buffer. 

**Associated Errors**

When a `KStr` is used by a system function:

* If `len` bytes starting from `buf` does valid UTF-8, the system function returns `INVALID_STRING`,
* If `buf` does not point to readable memory for `len` bytes, or another memory access error occurs, the system function returns `INVALID_MEMORY`.

**Definition**

```
struct KStr {
    buf: *const char,
    len: ulong
}
```

#### `KStrBuf`

`KStrBuff` is a structure type that is used to return text from the kernel to userspace. 

`buf` is a pointer to (userspace) memory that will be populated by the system function. `len` is the length of the buffer.

`KStrBuf` will always appear in system interfaces by mutable indirection (e.g. behind a `*mut`). 
When a system function succesfully returns, `len` is set to the size of the string written to `buf`. If a system function unsucessfully returns (typically with `INVALID_LENGTH` error), `len` is either the same value as on entry and buf is not modified, or `len` is set to the length of the string the system function attempted to write to the buffer. If there error is `INVALID_LENGTH`, then buf is guaranteed to be written in such a manner that (where `orig_len` represents the value of `len` at entry):
* The first `n<=orig_len` bytes written to `buf` contains valid UTF-8, and
* The bytes in `n..orig_len`, if non-empty, form a valid prefix of some valid UTF-8 character.

(That is, when `len` is overwritten and the function returns `INVALID_LENGTH`, the buffer is overwritten with valid UTF-8 that is truncated at `orig_len`)

Where `len< orig_len`, the bytes between `len..orig_len` are undefined.

**Associated Errors**

When `KStrBuf` is used by a system function:

* If `len` is insufficient to write the required string, `INVALID_LENGTH` is returned (`len` is always overwritten in this case),
* If `buf` is not writable for N bytes, where N is the number of bytes written by the system function (max of `len` bytes), `INVALID_MEMORY` is returned,
* If `buf` is not writable for `len` bytes, or another memory access error is detected, `INVALID_MEMORY` may be returned.

**Definition**

```
struct KStrBuf {
    buf: *const char,
    len: ulong
}
```

### `base::types::slice`

The `base::types::str` module defines types exposed for arrays of fixed size data.

#### `KSlice<T>`

A `KSlice<T>` is a const pointer to a fixed size buffer of type `T`. 

`ptr` is a pointer to an array of `len` elements of type `T`. 
`len` is the number of elements

**Associated Errors**

When `KSlice<T>` is used by a system function:

* If `ptr` cannot be read for `len` elements of type `T`, or another memory access error occurs, the system function returns `INVALID_MEMORY`

**Definition**

```
struct KSlice<T> {
    ptr: *const T!void,
    len: usize,
}
```

#### `KSliceMut<T>`

A `KSliceMut<T>` is a mutable pointer to a fixed size buffer of type `T`. The elements may be written, and/or read by the system function.

`ptr` is a pointer to an array of `len` elements of type `T`. 
`len` is the number of elements

**Associated Errors**

When `KSliceMut<T>` is used by a system function:

* If `ptr` cannot be read or written for `len` elements of type `T`, or another memory access error occurs, the system function returns `INVALID_MEMORY`

**Definition**

```
struct KSlice<T> {
    ptr: *const T!void,
    len: usize,
}
```

#### `KSliceBuf<T>`

`KSliceBuf<T>` is a structure type that is used to return array elements of type `T`. 

`buf` is a pointer to (userspace) memory that will be populated by the system function. `len` is the length of the buffer in a count of elements of type `T`.

`KSliceBuf<T>` will always appear in system interfaces by mutable indirection (e.g. behind a `*mut`). 
When a system function succesfully returns, `len` is set to the size of the string written to `buf`. If a system function unsucessfully returns (typically with `INVALID_LENGTH` error), `len` is either the same value as on entry and buf is not modified, or `len` is set to the length of the string the system function attempted to write to the buffer. 

Where `len< orig_len`, the elements between `len..orig_len` are undefined.

**Associated Errors**

When `KSliceBuf<T>` is used by a system function:

* If `len` is insufficient to write the required string, `INVALID_LENGTH` is returned (`len` is always overwritten in this case),
* If `buf` is not writable for N elements, where N is the number of elements written by the system function (max of `len` bytes), `INVALID_MEMORY` is returned,
* If `buf` is not writable for `len` elements, or another memory access error is detected, `INVALID_MEMORY` may be returned.

**Definition**

```
struct KSliceBuf<T> {
    buf: *const T!void,
    len: ulong
}
```

### `base::option`

The `base::option` module defines the `ExtendedOptionHead` type and `FLAG_OPTION_IGNORE` constant defined by [RFC 8].

### `base::hdl`

The `base::hdl` module types routines relevant to handles and shared handles:


**Overview**

```
inline use types::hdl;

pub struct WideHandle<H> {
    hdl: *handle H!Handle,
    pad([ulong; (16 / __SIZEOF_POINTER__) - 1])
}

fn IdentifyHandle(hdl: *handle Handle) -> SysResult = 0;
fn ShareHandle(hdl: *handle Handle) -> SysResult2<*shared_handle Handle> = 1;
fn UpgradeSharedHandle(hdl: *shared_handle Handle) -> SysResult2<*handle Handle> = 2;
fn ReleaseSharedHandle(hdl: *shared_handle Handle) -> SysResult = 3;

fn CheckHandleRight(hdl: *handle Handle, right: KStr) -> SysResult = 8;
fn DropHandleRight(hdl: *handle Handle, right: KStr) -> SysResult = 9;
fn DropAllHandleRights(hdl: *handle Handle) -> SysResult = 10;
fn RecheckPermissions(hdl: *handle Handle) -> SysResult = 11;
fn GrantHandleRight(hdl: *handle Handle) -> SysResult = 12;
```

#### struct `WideHandle<H>`

A `WideHandle<H>` is a 0-padded handle pointer that is the same width as wide as a `Uuid`. Any non-null handle stored in a `WideHandle<H>` is guaranteed not to have the same representation as a valid UUID.

Note: Where pointers are 128-bit, the handling of `WideHandle<H>` and `HandleOrId<H>` is not yet decided.

**Definition**

```
struct WideHandle<H> {
    hdl: *handle H!Handle,
    pad([ulong; (16 / __SIZEOF_POINTER__) - 1])
}
```

#### fn `IdentifyHandle`

`IdentifyHandle` is used to dynamically determine the type of a handle.
For a non-error result, the bottom 16 bits contains the type, and the top 15 bits contains the subtype (if applicable).

**Errors:**

* `INVALID_HANDLE`: If the input is not a live handle value on the current thread.

**Definition:**

```
fn IdentifyHandle(hdl: *handle Handle) -> SysResult = 0;
```



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