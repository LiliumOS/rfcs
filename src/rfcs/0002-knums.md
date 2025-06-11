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

## Security Considerations

None

## ABI Considerations

The knum language allows interfaces to be described as to their language API. When combined with a future RFC defining the system calling conventions and layout definitions, this is sufficient to know the complete ABI of these interfaces as well. This RFC does not define those conventions 

Note that the fact that the ABI of an interface only depends on the aforementioned calling convention RFC, and its knum API, it follows that 

## Prior Art

## Future Direction

<!--
Provide an informative explanation of any future possibilities.
-->

## References

### Normative References

* [Unicode 16.0](https://www.unicode.org/versions/Unicode16.0.0/)


### Informative References

<!--Include any documents cited to provide informative context only-->

