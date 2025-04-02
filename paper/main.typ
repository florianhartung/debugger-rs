#import "styling.typ": setup

#let td = text(red)[TODO]

#show: setup.with(
  title: "Development of a minimal x86-64 Unix debugger in Rust",
  authors: (("Florian Hartung", 6622800), ("Marek Freunscht", 9604914))
)

= Introduction
// Motivation/ Context
- Debuggers are programs that provide tools for developers to run, monitor and modify the state of other processes
- Historically, they have been used primarily to find and eliminate bugs that are inherently present in almost all software
- Another use case for debuggers is foreign code analysis, sometimes needed for reverse engineering or pentesting

- Debuggers come in many shapes and forms:
  - Some debuggers are architecture-specific or support multiple architectures such as ARM, x86, x86-64 or PowerPC
  - Some only work with types of programming languages such as languages compiled to machine code, Java or Python just to name a few.
// Goal
- In this work we present the bare minimum required to implement a debugger for low-level programs compiled to machine code.
- We limit our debugger to Unix and Unix-like systems on the x86-64 architecture.
- We want these features:
  - Attach to processes of programs
  - Breakpoints at fixed addresses
  - Reading memory at given addresses
  - Reading register contents
  - ...?
// Structure overview
- First we present important fundamentals:
  - Software execution lifecycle in Unix
  - Explore workings and techniques of debuggers with specific examples
- Then we present our debugger:
  - Requirements
  - Design/ architecture
  - Implementation
  - Verification and validation
  - and finally its usage

= Fundamentals
#td

== Processes and operating systems
#td

== Debuggers
#td

= Developing the debugger
#td

== Requirements
// TODO: Unix & Unix-like systems
// TODO: x86-64 architecture
#td

== Design
// TODO we split the project into a core debugger and a CLI
#td

== Implementation
#td

=== Attaching to processes
#td

=== Setting breakpoints
#td

=== Reading memory & registers
#td

== Verification & validation
#td

== Using the debugger
#td

= Conclusion
#td

= Outlook
// TODO Missing features
// TODO What could've been done better
#td


#bibliography("/bib.yml", style: "institute-of-electrical-and-electronics-engineers")