# LiliumOS RFCS

Defines the design decisions of the Lilium project, including accepted API interfaces and definitions.

## Copyright

The RFCs in this repository are licensed under the [`CC-BY-4.0`][src/CC-BY.md] license.

### API Policy Disclaimer

As part of these RFCs, human readable and machine-parseable definitions of public APIs for LiliumOS are defined, which are intended to be useful for software written against the LiliumOS. It is believed that, subject to this policy, these public APIs are not subject to copyright, and can be used freely without license. 
In the event that this legal assessment is wrong, we expressly and irrevocably waive all copyrights to these public APIs to the greatest extent permitted by applicable law, and in the event that any such copyrights survive this waiver, we further grant a worldwide, irrevocable, permission to all persons to copy, distribute, publish, modify, perform, and prepare derivatives without limitation of public APIs covered by this waiver, subject to the condition that any such person agree to the following warranty disclaimer, to the greatest extent permitted by law, in respect to all such uses of the public APIs

> THIS API IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL
WARRANTIES WITH REGARD TO THIS API INCLUDING ALL IMPLIED WARRANTIES
OF MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE
FOR ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY
DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN
AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT
OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS API.

For the purposes of this Policy, a "public API" refers to an element of the interface definition, such as the name of a constant, type, module, structure or union field, or function, or the value of a constant or system call, that would be semantically relevant to any link editor, compiler, or to runtime execution of the program for satisfying a functional purpose of the use of that interface, where duplication is needed to make that interface element compatible with any such program designed and developed against the interface definition, and to maintain the expected functionality of that interface. It does not include the following:
* The specific form or manner in which the definition is expressed,
* The name of function parameters, specific directives, or anything else that is not required for compatibility with other software,
* Any comment, documentation, or other prose element explaining the functionality of the interface in any particular way,
* Example code for implementing or using the interface

Further, the above Policy does not waive, disclaim, or license any trademark, trade dress, patent, or industrial design/design patent rights that may be applicable to the name or functionality of a specific interface.

## CI

CI setup is provided under .github/workflows/mdbook.yml. The configuration will build both a github page and epub artifacts on push to the `dist` branch, or on workflow_dispatch. You can modify the file accordingly if you wish to use another branch for page builds (like `main`). 
You need to set up github pages on github actions (and configure the environment appropriately).

Third party and other git host workflows are not yet implemented (PRs welcome for standard CI systems).

## Required Tools

To run locally you need:
* cargo, which can be installed via rustup
* mdbook, which can be installed by `cargo install mdbook@^0.4.37` or <https://github.com/rust-lang/mdBook/releases>, with a version compatible with 0.4.37.
* mdbook-fiction-tools, which can be install by `cargo install --git https://github.com/chorman0773/mdbook-fiction-tools>`. 

Once tools are installed, you can run `mdbook build` or `mdbook serve` as usual for an mdbook and view it 
