# Lilium knums IDL

## Summary

The knums Language is a simple Interface Description Language that allows describing system interfaces in a uniform, human-readable and human-writable, and machine-readable manner. The language is may be written by users, in the course of writing RFCs, to describe the expected interface of a subsystem.

## Motivation

Several Languages are anticipated to compile to Lilium, including C, C++, and Rust. Additionally, it is anticipated that the Lilium SCI will be implemented in multiple forms, 
 including from emulation/compatibility layer projects like winter-lily, and the official kernel. To ensure cross-contextual agreement of the API-level definitions of system calls, structures, important constants, and other metadata, it is useful to have a single "source of truth". Where other operating systems rely on C headers for this purpose, the Lilium OS instead uses a simplistic IDL.

## Informative Explanation

The knums language is defined with a syntax similar to Rust. The language is token-based, whitespace insensitive (other than to delimit tokens). 
Line comments begin with `//`, documentation line comments begin with `///`, and file documentation comments begin with `//!`

knum files are structured into folders and files, with each hierarchy level from the build root forming part of the file path (for `use` declarations). 

### Items

The language has 5 main top-level constructs (items):
* `use` declarations, which bring the contents of a specified module into scope,
* `const` items defining constant values of various types
* `fn` items defining system calls
* `struct` and `union` items defining custom types, and
* `type` alias items that allow naming an existing type in a different way.

Additionally, there are "directive" items (led by a `%` sign on their own line). These directives are not intended to be consumed by users, only for automated tooling (for example, `%define_int_types` is used in `types/int.knum` to instruct definition generators to provide support for using integer types).

Each item, other than a directive, may have any number of documentation lines immediately preceeding it. Before any item, including `use` declarations and directives, (but not necessarily before any comments), any number of "file" documentation lines may appear. 


## Normative Text

### Lexical Grammar

The following formal grammar, described in a simplified version of ABNF, defines the lexical format of the knums language

```abnf
; Whitespace Sensitive Grammer

file := [<whitespace>] *(<token> [<whitespace>])
    ; Every file must match the lexical file non-terminal

whitespace := 1*(<White_Space> / <comment>)

newline := %x0A
comment-begin := %x00-09 / %x0B-20 / %x22-2E / %x30-x10FFFF
comment-char := <comment-begin> / "!" / "/"

comment := "//" [<comment-begin> [*<comment-char>]] <newline>

doc-comment := "///" *<comment-char> <newline>
inner-doc-comment := "//!" *<comment-char> <newline>

ident := <XID_Start> *<XID_Continue> ; Except <keyword>

keyword := "use" / "type" / "const" / "mut" / "handle" / "shared_handle" / "struct" / "union" 

octal-digit := %x30-37

digit := %x30-39

hex-digit := <digit> / %x41-46 / %x61-66

uuid := "U{" 8<hex-digit> "-" 4<hex-digit> "-" 4<hex-digit> "-" 4<hex-digit> "-" 12<hex-digit> "}

hex-literal = "0" ("x" / "X") 1*(<hex-digit> / "_")

dec-literal = <digit> *(<digit> / "_")

oct-literal = "0 ("o" / "O") 1*(<octal-digit> / "_")

int-literal = <hex-literal> / <dec-literal> / <oct-literal>

punct := "=" / "*" / "+" / "-" / "^" / "&" / "|" / "<<" / ">>" / "<" / ">" / "->" / "{" / "}" / "[" / "]" / "(" / ")" / "/"

ascii-ident-begin := %x41-5A / %x61-7A / "_"

ascii-ident-continue := <ascii-ident-begin> / %x30-39

direcitve := "%" <ascii-ident-begin> *<ascii-ident-continue> 
    ; Must only be followed by `<White_Space>` or `<comment>` before first `<newline>`

token := <int-literal> / <uuid> / <punct> / <keyword> / <ident> / <doc-comment> / <inner-doc-comment> / <directive>
```

The following conventions are used above: 
* The `XID_Start`, `XID_Continue`, and `White_Space` non-terminals each match a single character belonging to that Unicode Character Class under [Unicode 16.0](https://www.unicode.org/versions/Unicode16.0.0/),
* Whitespace characters in the input are preserved,
* Every input file must match the `<file>` production to lex
* The input is consumed such that matches are formed based on the longest and most specific rule that matches, (for example, `/// <input>` should be treated as a doc-comment, not a comment)
* Any input that matches the `<keyword>` production is not matched by the `<identifier>` production (but may be matched as part of a directive)

### Syntatic Grammar

The following ABNF defines the syntactic grammar:

```abnf

file := *<inner-doc-comment> *<bare-item>

bare-item := <item-struct> / <item-sysfn> / <item-const> / <item-use> / <directive> 

item-struct := <struct-keyword> <ident> [<generics-def>] [":" *<struct-option>] <struct-body>

struct-body := <struct-opaque-body> / <struct-fields>

struct-opaque-body := "opaque" ["(" <type> ")"] ";"

struct-fields := "{" [<struct-field> *("," <struct-field>) ["," [<struct-padding>]]] "}"

struct-field := *<doc-comment> <ident> ":" <type>

struct-padding := "pad" "(" <type> "," <expr> ")"

item-sysfn := "fn" <ident> <fn-signature> "=" <expr> ";"

item-const := "const" <ident> ":" <type> "=" <expr> ";"

item-use := ["inline"] "use" <path> ";"

path := <ident> / <path> "::" <ident>

fn-signature := "(" [<fn-param> *("," <fn-param>) [","] ] ")" "->" <type>

fn-param := [<ident> ":" ] <type>

simple-type := ("(" <type> ")") / <named-type>

type := <simple-type> / <pointer-type> / <fn-pointer-type> / <array-type> / "!"

pointer-type := "*" ("const" / "mut" / "handle" / "shared_handle") <type>

fn-pointer-type := "fn" <fn-signature>

array-type := "[" <type> ";" <expr> "]"

named-type := <ident> [<generics-type>]

generics-type := <generic-list> / <alternate>

generic-list := "<" [<type> *("," <type>) ","] ">"

alternate := "!" <type>

expr := <literal-expr> / <ident> / <unary-expr> / <non-binary-expr>

binary-expr := <expr> <binary-op> <expr>

; Precedence Order is 
; High: `<<`, `>>`
;  | : `&`, `|`, `^`
;  v : `/`, `*`
; Low: `+`, `-`
binary-op := "<<" / ">>" / "&" / "|" / "^" / "/" / "*" / "+" / "-"

unary-expr := <unary-op> <expr>

unary-op := "+" / "-" / "!"

literal-expr := <int-literal> / <uuid> 
```

The following conventions are used:
* All productions defined in the lexical grammar, other than the `file` production, may be referred to by the syntatic grammar and match the same token produced by that grammar
* The contents of the file must match the `<file>` production after stripping all whitespace and comments (other than doc comments and inner doc comments)

### Items

Each file is a collection of items, that represent the public api of the system. There are 6 kinds of items:
* Directives
* Use Definitions
* `const` items
* `fn` items
* `struct` and `union` items
* `type` aliases.

#### Directives

A directive is a `%` prefixed ascii identifier. It must appear on its own line and have nothing other than whitespace preceeding or following it before a newline.

A directive is a command to the processor tool, it does not form part of the public API. 

#### Use Definitions

An item introduced by the `use` keyword specifies a path (a sequence of identifiers separated by `::`). A `use` item makes the definitions in the specified path available for definitions in the current file (See below for the rules for how to find the path of a given file).

Before the `use` keyword, the `inline` contextual keyword may be present. If the `inline` keyword is present, the `use` item is part of the public api of the file, and the items of that file are accessible from any file that imports the current file. Otherwise, the items made available by the `use` item are not guaranteed to be available outside of the current file.

#### `const` items

A `const` item defines an important named constant of a specified type. 

#### `fn` items

An `fn` item defines a system function with a specified signature. It has an expression that represents the system function number within the subsystem it belongs to.

An `fn` item's signature contains a number of parameters (at least 0) which have a type other than `void`, `!`,  or an array type. Parameters also have a name which are informative to users and documentation readers. It also has a return type which is a type other than an array type.

#### `struct`/`union` items

A struct item is introduced by the `struct` keyword, and a union item by the `union` keyword. Both introduce new named types. 

Both types have a body which can be either a braced struct body, or for `struct` types only an `opaque` body. Prior to the body, a number of attributes

A braced struct body contains a number of fields, which have a name and a type. The fields must be initialized to a value of the type. The body may end with the keyword `pad` which contains a type. The type must be an integer type or an array of an integer type. Such a keyword indicates that the type is extended as though by a field of such a type, which must be initialized to `0`.

An opaque body optionally specifies a type field in parenthesis. The type provides a hint about a base type. It shall be valid to cast between pointer types to the base type and the derived type. 
Opaque types cannot be constructed, and may not appear directly in signatures, const items, or `struct` fields.

##### Struct attributes

Prior to the body, one or more attributes may be specified. Attributes are of the form `<ident>(<expr>)`. 

The following attributes are currently recognized:
* `align(N)`, `N` must an expression of type `ulong`, and must be a power of two. Indicates that the type requires alignment to at least `N` bytes.
* `option(ID)`. Inserts a field at the start of the structure type `ExtendedOptionHead` (The file `types::option` must be `use`d to use this struct attribute), and provides sufficient definitions. Must appear on a `struct`. 
* `option_head(N)`: Must appear on a `union`. Inserts a field of an unnamed struct type, containing a field of type `ExtendedOptionHead` (the file `types::option` must be `use`d to use this attribute), and a field of type `[byte; N]`.

#### `type` items

A `type` item introduces a named alias for a specified type.

### Types

#### Integer types 

The types `uN` and `iN` are integer types (valid only for N=`8`, `16`, `32`, `64`, `128`, and `long`, all others integer values for `N` are reserved identifiers for type names).

An integer type `uN` represents a N-bit unsigned integer type with values in `[0, 2^N)`. An integer type `iN` represents an N-bit signed twos-complement integer type with values in `[2^(N-1), 2^(N-12))`. 

The type `ulong` and `ilong` represent integer types that have the same width as a pointer on the current platform. It has the same range and representation as the equivalent `uN` or `iN` type, but is a distinct type. 

To use an integer type, the file `types::int` must be `use`d. 

#### `byte` type

The type `byte` is a byte type. It has the same width as `u8` and can be initialized from values of type `u8`. `byte` may also contain uninitialized values.

#### `char` type

The type `char` is a character type. It has the same width and representation as `u8`, but is a distinct type.

#### Named types

Any identifier may be used as a type. The identifier is only valid in the type position if it refers to a `struct`, `union`, or `type` item in scope, or if it refers to the name of a generic parameter for a containing `struct.

Identifiers of the form `uN`, `iN` (`N` is an integer value), `ulong`, `ilong`, `byte`, `char`, and `void` do not resolve to named types and instead resolve to the specified type.

Named types may be followed by either a generic-arg-list or a replacement type. 

#### `void` type

The type `void` is the void type. It may only appear in return types of function definitions or function pointers. It represents an empty return from a function

#### Array types

An array type is of the form `[<elem>; <len>]` where `elem` is the element type and `len` is the length of the array, an expression of type . The array is layed out with the elements contiguous in memory.

Array types cannot appear in parameters or return types directly.

#### Pointer types

A pointer type is of the form `*<kind> <pointee>` where pointee is the pointee type and `kind` is one of the following keywords: `mut`, `const`, `handle`, or `shared_handle`.

`mut` and `const` represents userspace pointers to mutable or immutable data respectively. 

`handle` and `shared_handle` represents pointers to kernel objects. `shared_handle` represents a pointer to an explicitly shared resource.

`handle` and `shared_handle` pointers may only be used if the `types::hdl` module is `use`d. 

Pointers may be to any type, including arrays, `void`, `!`, or `opaque` structs. Such pointers can appear in any position.

#### Function Pointer Types

A function pointer type is introduced by the `fn` keyword, and is followed by a function signature (like for an `fn` item)

## Security Considerations

None

## ABI Considerations

The knum language allows interfaces to be described as to their language API. When combined with a future RFC defining the system calling conventions and layout definitions, this is sufficient to know the complete ABI of these interfaces as well. This RFC does not define those conventions 

Note that the fact that the ABI of an interface only depends on the aforementioned calling convention RFC, and its knum API, it follows that 

## Prior Art

## Future Directions

* Userspace function `fn` items
    * Defining USI items
* Additional items, expressions, and types may be introduced

## References

### Normative References

* [Unicode 16.0](https://www.unicode.org/versions/Unicode16.0.0/)

