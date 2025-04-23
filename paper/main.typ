#import "styling.typ": setup

#let td = text(red)[TODO]

#show: setup.with(
  title: "Development of a minimal x86-64 Unix debugger\n in Rust",
  authors: (("Florian Hartung", 6622800), ("Marek Freunscht", 9604914))
)

= Introduction
// Motivation/ Context
Debuggers are programs that provide tools for developers to run, monitor and modify the state of other processes.
Historically, they have been used primarily to find and eliminate bugs, which are inherent in almost all software developed.
Another prominent use case for debuggers is foreign code analysis, sometimes needed during reverse engineering or pentesting.
Due to the vast amount of different technologies and languages existing, debuggers must adapt, which is why they come in many shapes and forms:
Some debuggers are architecture-specific or support only only a specific set of architectures such as ARM, x86, x86-64 or PowerPC.
Other debuggers may require the use of specific programming languages, such as languages that compile to machine code, Java or Python just to name a few.

// Goal
In this work, we present the bare minimum required to implement a debugger for low-level programs that compile to machine code.
We limit our debugger to Unix and Unix-like systems out of personal preference and the x86-64 architecture because of its widespread use.
Also we choose Rust as the programming language for our debugger, due to its advantages regarding memory safety and modern approach to software.
The debugger is required to implement a certain amount of base features we identify as necessary for a minimum viable product.

// Structure overview
First, we present important fundamentals such as the lifecycle of a process in Unix/Unix-like systems.
Then we explore the workings and techniques used by popular debuggers through specific examples.
The next section presents our debugger with its initial requirements, development process and application in a test scenario.


= Fundamentals
#td

== Lifecycle of a process in Unix/Unix-like systems 
#td

== Debuggers
// TODO present commands such as gdb's `break` and then present what happens internally
#td

= Developing the debugger
#td

== Requirements
// TODO: Unix & Unix-like systems
// TODO: x86-64 architecture
1. attaching to processes
2. setting breakpoints at fixed addresses
3. reading data from memory, the stack and registers.
...#td

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


#bibliography("./bib.yml", style: "institute-of-electrical-and-electronics-engineers")
