# Elf Statically Linked Library Format

## Summary

The Elf Statically Linked Library Format is a semi-modern static linking format that uses relocation infrastructure that already exists for ELF files on Lilium.

## Motivation

Static Libraries on most elf platforms (and indeed, most platforms) are provided using archive files, with special additional metadata. 
The Archive format itself is limited, highly defined by legacy support, and has limited ability to control its use.


Due to the nature of these libraries, and backwards compatibility considerations, this leads to a number of surprising effects:

* Linking a static library does not, by default, link all objects that were combined to produce it, only the ones that have symbols referenced,
* As a result of the above, symbols that are expected to appear in the output, and initialization/finalization functions, won't get included normally,
* Additionally, it is not possible to "link" an archive against other libraries, either static or dynamic, without including the contents of a static library directly in the archive.

## Informative Explanation

Static Library files are a special type of ELF File for the Lilium OS. They obey similar (though different) basic rules as regular relocatable object files (as though they are simply a "merged" or partially-linked object file), but are treated more like shared objects by link editors (such as interacting with the `--as-needed` flag and `AS_NEEDED` linker script directives). 
Such Libraries have more consistent behaviour with other parts of the toolchain, and less surprising behaviour than legacy archive-based static libraries.

There are two types of Static Libraries: RVA (Relative Virtual Address) Unresolved and RVA Resolved.

RVA Unresolved libraries act like normal object files and are split into several sections that get merged into the final artifact according to normal rules.

RVA Resolved libraries resolve the addresses between sections of the library, and requires being treated as a single opaque blob. This allows many relocations within the library against local symbols to be processed when making the library, instead of when linking against it. However, RVA Resolved Libraries are restricted in how they can be used, only allowing being linked into Executables, Dynamic Libraries, and other RVA Resolved Libraries.

Static Libraries also allowing carrying dynamic linking information (unlike existing static libraries) that cause the resulting binary link against the same shared object without an explicit mention on the link line of the executable.

The adoption of Static Libraries includes a soft phase-out of archive-based static libraries, which may be removed later.

## Normative Text

### Changes to the ELF Header

When targeting the Lilium Elf OSABI (according to RFC 5), the following additional values are supported for the `e_type` field:

| `e_type`                   | Value    | Description                 |
|----------------------------|----------|-----------------------------|
| `ET_LILIUM_STATIC_LIBRARY` | `0xFE00` | Statically Linkable Library |

An `ET_LILIUM_STATIC_LIBRARY` shall be produced and handled as defined herein.

### Format Specification

An `ET_LILIUM_STATIC_LIBRARY` has two format modes, Relative Virtual Address (RVA) Unresolved and RVA Resolved modes, indicated by the presence or absence of a program header.

In RVA Unresolved Mode, the object shall be treated by the link editor as an `ET_REL` file, except for the considerations defined in [Recommended Treatment for Static Libraries](#static-linking-recommendations). Where the `e_phoff` member or the `e_phnum` member of the Elf header is `0`, the file is in RVA Unresolved Mode. From the `.dynamic` section, `DT_NEEDED` entries and the contents of the `DT_STRTAB` must be copied into the output file. `DT_NEEDED` entries must be adjusted to the `DT_STRTAB` of the complete file, and may be deduplicated.

In RVA Resolved Mode, the object shall be handled by a link editor in a hybrid form:

* The Link Editor must assign a base address within the full link image for each RVA Resolved Mode static library
* All `PT_LOAD` and `PT_TLS` segments and the contents must be copied into the output, with the physical and virtual addresses updated by the base address of the library,
* If any section covered by a `PT_LOAD` or `PT_TLS` segment is omitted from the output file, all sections from the file must be omitted, and if any section covered by a `PT_LOAD` or `PT_TLS` file is included in the output file, all such sections must be included,
* The Relative Virtual Address of any two sections in the file covered by a `PT_LOAD` or `PT_TLS` segment must be preserved,
* On psABIs that support `R_*_RELATIVE` and `R_*_IRELATIVE` relocations, these must be resolved according to the target relocation format, using the base address of the relocation image. If the result is a shared object and position independant executables, these must be adjusted to `R_*_RELATIVE` and `R_*_IRELATIVE` relocations using the base address within the full image,
* All other relocation types must be handled the same way as the same relocation type for an `ET_REL` file,
* `DT_NEEDED` entries and the contents of the `DT_STRTAB` must be copied into the output file,
* The considerations defined in [Recommended Treatment for Static Libraries](#static-linking-recommendations) apply when linking RVA Resolved static libraries.

Only an RVA Resolved Mode `ET_LILIUM_STATIC_LIBRARY` file may have a program header, and the program header may only contain the following segment types:

* `PT_LOAD`,
* `PT_TLS`,
* `PT_GNU_STACK`, which must set exactly `PF_R | PR_W` and may be ignored,
* `PT_NOTE`.

The file header and program header for an `ET_LILIUM_STATIC_LIBRARY` must not be contained within an `ET_LOAD` segment in the memory image of the static library. The determination of whether or not to include these headers in the memory image is controlled by the final link step to an executable or shared object.

Link Editors should reject files that violate constraints put on RVA Resolved Mode file Program Headers. If it does not reject such a file that it is given on the link line, the behaviour is unspecified.

An `ET_LILIUM_STATIC_LIBRARY` in RVA Resolved Mode cannot be an input to a partial/relocatable link to an `ET_REL` file. An object in RVA Unresolved Mode may be. A Link Editor should diagnose an attempt to do so regardless of the Mode of the file. When linking an `ET_LILIUM_STATIC_LIBRARY` into another `ET_LILIUM_STATIC_LIBRARY`, the result must be an RVA Resolved Mode file if any input file is in RVA Resolved Mode.

### Recommended Treatment for Static Libraries {#static-linking-recommendations}

When a link editor links against any `ET_LILIUM_STATIC_LIBRARY`, regardless of whether it is RVA Resolved or RVA Unresolved, it should discard the library according to the rules it uses for shared objects/dynamic libraries. More specifically:

* If the same named static library appears more than once on the link line, all but one copy of the library should be discarded,
* If the static library is subject to as-needed mode (the `--as-needed` flag (conv. `--no-as-needed`) on GNU-style link editors), it should be discarded if a symbol defined by it is not referred to by any object, any object included from an archive, and any static library that is included in the link output. If two or more as-needed static circularily refer to symbols defined in the other libraries, all such libraries must be included or discarded together, and must be included if there is at least one reference to any symbol by an object file, or no-as-needed static library.

When including a static library with a `DT_NEEDED` entry, the link editor should diagnose an issue if it cannot find a library referred to by any `DT_NEEDED` entry in:

* The paths normally searched by the link editor for shared objects,
* The paths set in the rpath (not the runpath) of the resulting elf file, 
* Any paths set by the rpath-link option of the link editor, and
* Any other link editor specific search locations.

### Producing Static Libraries

A Link Editor producing a static library shall do the following steps:

* Combine the sections of each input file according to the rules for a relocatable/partial link,
* Where multiple `GRP_COMDAT` groups with the same control symbol are present across the input files, choose one such group and discard the sections from all other groups,
* Resolve relocations of symbols within the newly combined sections,
* Set the Binding of any STB_GLOBAL or STB_WEAK symbol that is locally defined and STV_HIDDEN or STV_INTERNAL to STB_LOCAL,
* Additionally, set the binding of non-exported STB_GLOBAL or STB_WEAK symbols that are locally defined to STB_LOCAL. The manner to indicate which symbols are exported and not-exported is link-editor dependant, and may not be supported (in which case all symbols that are at least STV_PROTECTED are exported),
* If required, emit a `.dynamic` section according to [Dynamic Linking](#dynamic-linking).

A Link Editor producing a static library in RVA Resolved must additionally do the following:
* Assign addresses for each section,
* Emit `PT_LOAD` and `PT_TLS` segments to contain each `SHF_ALLOC` and `SHF_TLS` section, other than the `.dynamic` section,
* Resolve relocations of symbols other than weak symbols and `STV_DEFAULT` symbols.

#### Weak Symbols

When an locally-defined STB_WEAK symbol is marked STV_PROTECTED, the correct behaviour for the symbol is not clear. In the current version of this specification, the STB_WEAK symbol is preserved, and is not duplicated into a local non-weak symbol. The symbol may be overriden by other symbols defined in the same link target (executable or shared object). The Link Editor should diagnose if such a symbol is emitted, unless it refers to a section within a `GRP_COMDAT` group. The behaviour specified here should not be treated as ABI stable.

#### Object Form

Static Libraries that are in RVA Unresolved mode can be emitted as an ET_REL file. Other than not being subject to the special treatment as link inputs, they behave identically as linker inputs. 
Link Editors and Object Transformation tooling may provide an option to convert a static library to an object file. This should only be done by explicit request of the user.

RVA Resolved Static Libraries cannot be converted to an ET_REL file.

### Dynamic Linking

When producing a static library file, a link editor may be instructed to link shared object files. The treatment of these files should be the same as for executable and dynamic library linking. When this occurs, a `.dynamic` section must be emitted to contain a `DT_STRTAB` and `DT_NEEDED` entries for those files. Other entries must not be produced.

For RVA Resolved library files, a PT_DYNAMIC segment must not be generated, and the `.dynamic` section must not be covered by any PT_LOAD segment. This allows the section to be incorporated into 

### Archives

Archive static libraries are deprecated by this RFC, but will not be disabled for Lilium Link Editors. Link Editors targetting Lilium must support Archives as a form of static linking. This may be phased out by future RFCs and toolchain updates.

This does not rule out the adoption of a future archive-like static library format with a more comprehensive format.

## Security Considerations

Handling `ET_LILIUM_STATIC_LIBRARY` files incorrectly can produce executable files that contain memory safety vulnerabilities.

Like all ELF Files, linking arbitrary static libraries into executables and running them can perform arbitrary code execution without the execution security context limits and process namespacing scope.

## ABI Considerations

The handling of static libraries, both producing them, and linking against them, generally presents an ABI Boundary.

Particularily, the handling of RVA Resolved Static Libraries requires novel consideration that cannot be removed later without breaking existing library files and tooling.

## Prior Art

* Prior gABI proposal: <https://groups.google.com/g/generic-abi/c/sT25-xfX9yc>,
* Binutils Fork that partially supports that gABI proposal: <https://github.com/eyalitki/binutils-gdb/tree/static_bundle>

## Future Direction

* Phase-out of Archive Format
* Future Support of a different static linking format
* Collaboration with other OSABIs, psABIs, or gABI to support a more generic version of this same format
* Resolving the STB_WEAK/STV_PROTECTED issue and providing more comprehensive support for that symbol combination.

## References

### Normative References

* ELF gABI: <https://gabi.xinuos.com/>
* Lilium RFC 5: <https://github.com/LiliumOS/rfcs/pull/5>
