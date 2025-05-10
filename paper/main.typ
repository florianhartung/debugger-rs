#import "@preview/acrostiche:0.5.1": *
#import "styling.typ": setup

#let td = text(red)[TODO]

#show: setup.with(
  title: "Development of a minimal x86-64 Unix debugger\n in Rust",
  authors: (("Florian Hartung", 6622800), ("Marek Freunscht", 9604914))
)

#let acs = (
  "ELF": "Executable and Linkable Format",
  "ASLR": "Address space layout randomization",
  // DWARF is no an acronym
)
#init-acronyms(acs)
#let acs = acs.keys().map(x => (x, ac(x))).to-dict()

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
This section describes fundamentals for processes on Unix/Unix-like systems and debuggers useful for later development.

== Lifecycle of a process in Unix/Unix-like systems 
Methods of creating processes
- fork
- exec @exec

- What happens when a process is created?
- #acs.ASLR

- Memory maps & sections?

== Signals

== Debuggers
// TODO present commands such as gdb's `break` and then present what happens internally
- What does a debugger do on an abstract level [gdb manpage]
- Interactions with operating systems
- Features of debuggers
  - Breakpoints 
  - Instruction stepping
  - Register + memory access/modification
  - Relating symbols to source code

== Symbols
- Debug information produced by compilers
- Types of symbols for #acs.ELF binaries
- DWARF#footnote("DWARF is not an acronym, instead it's a backcroynm") for stack frame debug information
- Debuggers use this information to directly relate source code 

- Look at ELF symbol table & string table for example

// are variables & stack frame debug information also symbols?

= Requirements
// TODO: Unix & Unix-like systems
// TODO: x86-64 architecture
1. attaching to processes
2. setting breakpoints at fixed addresses
3. reading data from memory, the stack and registers.
...#td

= Design
- We split the project into a core debugger and a CLI for modularity and ease of development
  - CLI design
  - Core design
    - Debugger loop
    - Signal
- Hardware debug registers 
  - Hardware debug registers: method for debugging that required kernel privileges
- ptrace on a high level: a syscall for monitoring other processes
- Using Hardware debug registers through ptrace
- We choose ptrace for our debugger design

= Implementation
#td

== Attaching to processes
- PTRACE_ATTACH vs fork + PTRACE_TRACEME

== Setting breakpoints
- Software vs hardware breakpoints
  - PTRACE_POKETEXT (Writing int3 into program flow) or hardware debug registers
  - One or both?

== Reading memory & registers
- PTRACE_PEEKTEXT, PTRACE_POKEDATA & PTRACE_GETREGS

== Instruction Stepping
- PTRACE_SINGLESTEP

= Debugger Usage
- Show 2-3 example programs and the commands used to interact with the debugger

= Conclusion
#td

= Outlook
// TODO Missing features
// TODO What could've been done better
#td


#bibliography("./bib.yml", style: "institute-of-electrical-and-electronics-engineers")
