# RFC Policy

## Summary

The public facing portions of the Lilium OS Project will be developed through a serious of documents, called "Requests for Comment" or "RFC"s, which will provide the basis for the policies of the Lilium OS Project and the normative text for defining the public interface.

## Motivation

A Community Oriented Process for creating and evolving the Lilium OS is desirable. Using a public "Request for Comments" system allows the Leads of the Lilium OS to meaningfully debate the specification while making it easy for community members with their own interests to comment on the development.

## Informative Explanation

The RFC Process is the mechanism for evolution of the Lilium OS. Throughout the lifecycle of the project, documents, known as Requests for Comments or RFCs will be published for review, comment, and consensus. These documents are used to change and evolve the Lilium OS Interfaces itself, a project-level policy, or other major portions of the Lilium OS Project.

While an RFC is "Live", it is open to public review comment, not restricted to members of the project or Core Interest Groups. Likewise, anyone may open an RFC to public comment, subject to policies regarding contributions. After comment, the Lilium Leads, approve or reject the RFC, which is followed by a period for final comment, where outstanding concerns can be raised and addressed.

An RFC is proposed and becomes "Live" by opening a pull request to https://github.com/LiliumOS/rfcs with the rfc text in a new item in the src/rfcs folder.

A Template for RFCs is provided, and can be used to aid in authoring RFCs. The template sets out required, recommended, or suggested parts of the RFC and what questions an RFC should address. It includes explanations of each section.

## Normative Text

### Purpose of an RFC

An RFC is required if it makes any of the following changes:
* The interacts with the primary, user-facing definitions of the Lilium Operating System (kernel or standard USI impl), including by modifying any of the following:
    * Defining a new core SCI Subsystem,
    * Defining a new standard SCI Subsystem (other than as experimental) or labeling a previously-experimental standard SCI Subsystem as no longer experimental,
    * Adding new system calls to a core SCI Subsystem or a non-experimental standard SCI subsystem,
    * Modifying the System Call ABI or the Userspace ABI,
    * Modifying, including by adding to, the Executable Format Specification used by the operating system,
    * Defining a new standard USI interface in an existing USI library or defining a new USI library,
* It establishes a project-wide policy or charters a group within the project designed to carry out the project work in a formal manner,
* Or otherwise if it meaningfully amends any preexisting RFC.

An RFC may also be useful, but is not necessarily required, if the change has wide impact on the project, or is important in a meaningful way.

And RFC is not required for the following:
* To make internal-only changes to the kernel (<https://github.com/LiliumOS/lilium-kernel>), usi (<https://github.com/LiliumOS/lilium-usi>), or winter-lily (<https://github.com/LiliumOS/winter-lily>) or to make changes necessary to implement another RFC,
    * Some internal changes may be useful to put forward as an RFC.
* To create a new experimental standard subsystem or implement them in either the kernel or winter-lily,
* To create extension or non-standard subsystems.

### Lifecycle of an RFC

The following is the Lifecycle of every RFC

0. Pre-Review (Optional): The RFC is brought for informal review and drafting, in an incomplete (or Pre-RFC) form designed for drafting and initial design,
1. Submission: The RFC is submitted to <http://github.com/LiliumOS/rfcs> as a Pull Request. Generally, the RFC must contain the content set forth in the [content](#content) section,
2. Review and Iteration: The Pull Request is reviewed and discussed, with appropriate concerns, design questions, proposed design changes, and appropriate changes are made to address these concerns, 
3. Approval: Once the RFC is reviewed appropriately the Lilium Leads must approve for it to be merged. Approval represents consensus to adopt the RFC.
4. Final Comment Period: After approval the RFC must undergo a 7 day final comment period. Approval may be revoked at any point by any Lead.
4. RFC Number Assignment: The RFC is assigned a number based on its PR Number and the document is renamed accordingly.

An RFC may be explicitly closed also by request of all of the Core Interest Groups - this indicates that there is consensus not to move forward at the current time. A Closed RFC may be reopened or refiled in the future.

After approval and the final comment period, any member of the Project may perform the RFC Number Assignment (if the Number was not previously assigned) and merge the RFC.

### Content

Generally, an RFC must contain at least the following:
* A One to two paragraph summary of the RFC,
* Motivation for the RFC and the underlying proposal,
* An Informative Explanation of the proposal,
* The Normative Text of the proposal.

Additionally, an RFC should specify the following, as applicable:
* Any Security Considerations that may apply to the RFC, both as to users and implementors,
* Any Considerations on the System Call or Userspace Application Binary Interface,
* A description of Prior Art that informed the proposal,
* A description of Future Changes and Directions that can be made in respect to the RFC.

### Copyright Licenses and Notices

Each RFC is provided under a unified license. Everyone who submits an RFC must permit the Lilium OS Project to license the RFC under the unified license in force at the time. 

The License for RFCs is currently [`CC-BY-4.0`] and requires an RFC to change. 
Such a change applies to previous RFCs as well as future RFCs, though previous RFCs will remain available under the previous license as well.

Any Code submitted in an RFC is subject to the MIT License. This include examples and snipets. 
API Definitions are deemed by the Project to not be subject to copyright, due to their inherent functionality, 
If this assement is incorrect, such API signatures are also released under the terms of the MIT License.

The RFC License shall always be an Open Documentation License.

### RFC Template

An RFC Template shall be provided to aid in the authoring of RFCs. The template shall provide sections for the elements of an RFC set forth in the Content section. Use of the template is not required for an RFC, however.
Modifying the template is not an RFC of its own.

## Security Considerations

There are no security considerations for this RFC, as it strictly defines a policy of the Project.

## Prior Art

* [Rust RFC 2](https://rust-lang.github.io/rfcs/0002-rfc-process.html)
* IETF RFC Process

## Future Direction

* This policy does not specify the method for fixing "releases" of the Specification/OS Definition, whether the specification is simply a snapshot of the merged RFCs as a whole at any given time, or some formal stabilization process is required,
    * Likewise, it is not yet specified how or if releases will be "versioned", and how versions will be designated or discovered,
* The Policy also Leaves Open how to modify RFCs for non-technical reasons (such as for editorial purposes),
* While Copyright is addressed by the Policy, Patent considerations are currently omitted. This may need to be addressed at some point in the future,
* Finally, the policy requires RFCs to be approved by the Lilium Project Leads. In the future a proper team may be chartered for this purpose, and certain kinds of RFCs may be delegated to other such teams.

## Normative References

- [`CC-BY-4.0`] - Copyright License by Creative Commons Team


[`CC-BY-4.0`]: <https://creativecommons.org/licenses/by/4.0/legalcode.en>