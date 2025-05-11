#import "@preview/acrostiche:0.5.1": *
#import "styling.typ": setup

#let td = text(red)[TODO]

#show: setup.with(
  title: "Development of a minimal x86-64 Unix debugger\n in Rust",
  authors: (("Florian Hartung", 6622800), ("Marek Freunscht", 9604914))
)

#let acs = (
  "ELF": "Executable and Linkable Format",
  "ASLR": "Address Space Layout Randomization",
  "PIE": "Position Independent Executable",
  "API": "Application Programming Interface",
  // DWARF is no an acronym
  "REPL": "Read-eval-print loop",
  "OS": "Operating System"
)
#init-acronyms(acs)

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

== Process lifecycle in Unix/Unix-like systems 
Processes are programs during runtime.
They contain information such as the program counter, registers, variables, open files or page metadata@modern-os.

// Process creation
In Unix and Unix-like systems, there exist multiple methods for creating new processes, however the most common one is the `fork` syscall@fork.
A process may use this syscall to create an identical clone of themselves.
Followed by diverging control flow inside this process' program code, the newly created child process can then behave differently form the original parent process.
Often the child process then calls a syscall of the `exec` family (or the underlying `execve` directly), which replaces its current memory layout with a completely new one from a specified program.

// ASLR & memory mappings
During this `exec` syscall, the #acr("ASLR") technique is initialized, provided that the executed program is a #acr("PIE").
#acr("ASLR") randomizes the starting address of memory section such as the text, heap, or stack sections to prevent certain attacks, which exploit the normally fixed memory layout.
The mappings of memory sections are stored by the kernel and made available at `/proc/<PROC_ID>/maps` for every process by its process id `<PROC_ID>`.

== Signals
- What are signals?
- What kinds of signals? Name examples
- Signal handlers (`wait` syscalls??)
- Why do we need them?

== Debuggers
// Abstract level
Debuggers are programs that can attach themselves to other processes and then monitor and modify them.
They are mainly being used by developers for debugging programs, i.e. identifying and tracing bugs/errors in these programs.
Although there are other use cases such as reverse engineering.

// Usage & common commands
Widely-known debuggers, namely `gdb` or `lldb`, are terminal #acrpl("REPL") with a notable overlap of common commands between them.
// Create process & execution
Commands such as `run` or `continue` are used to create and attach to a new process or resume execution of a process the debugger is already attached to.
// Breakpoints
Breakpoints can be set via commands such as `break` or `breakpoint set` for arbitrary code addresses or addresses of symbols.
When the program counter of the process then reaches a previously set breakpoint, execution is preempted and control transferred to the debugger allowing further user input to happen.
// Watchpoints
Watchpoints are similar to breakpoints allowing execution preemption, however they trigger on memory reads and writes of set addresses instead of execution.
// Read/write program state
While execution of a process is halted, the process state including but not limited to call frames or variable contents can be read and modified.

// Requirement for OS interaction
- #acrpl("OS") usually isolates processes
- However debuggers require various types of access to other processes
- Thus #acrpl("OS") also have to provide some kind of method for one process to debug another
  - For Unix/Unix-like systems this is ptrace (short for: process trace), which is a syscall for a family of different request types
  - Also methods to circumvent the OS exist? Hardware breakpoints

- Interactions with #acrpl("OS")
- Common features of debuggers
  - Breakpoints: Halting execution for later continuation
    // TODO present commands such as gdb's `break` and then present what happens internally
  - Instruction stepping: "Dynamic breakpoints" on a smaller scale for every instruction.
  - Register + memory access/modification: Read/write data
  - Relating symbols to source code: Useful feature for user

== Symbols
- Debuggers only know addresses by default, however this makes usage hard

- Compilers improve debugging experience by producing additional information useful to a debugger
- Types of symbols for #acr("ELF") binaries: functions, ...
- Debuggers use this information as annotations to relate parts of machine code to its original source code
- Look at ELF symbol table & string table for example

// are variables & stack frame debug information also symbols?
- Short paragraph on DWARF#footnote("DWARF is not an acronym, instead it's a backcroynm") for stack frame debug information

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
There are two main methods to set a breakpoint inside a process that is currently running, software breakpoints and hardware breakpoints.

Software breakpoints use the int3 instruction of the x86-64 instruction set, which triggers an exception inside the processor #cite(<intel-manual>).
When this exception is encountered, a trap occurs, transferring control to the operating system, which in the case of Unix-like systems will send a SIGTRAP to the running process.
To set a software breakpoint in the tracee process, the debugger may write the `int3` instruction directly into the executable text segment of the tracee with the PTRACE_POKETEXT #acr("API") #cite(<ptrace>).
The first byte of the instruction at the breakpoint address can be overwritten with `int3`, which has a one-byte opcode (0xCC). 
When this breakpoint is hit during execution, the debugger has to write back the first byte of the instruction which was in the breakpoint address initially.
Execution of the process is then resumed for one instruction, after which the `int3` instruction is re-inserted into the breakpoint, so that it can be hit again.

In contrast, hardware breakpoints use the hardware debug registers that are a part of the x86-64 architecture.
These registers make it possible to specify breakpoints at 4 different addresses inside the process.
However, the breakpoints that are stored in hardware registers are more powerful than software breakpoints, as they can also be triggered on memory reads and writes, as opposed to just execution.
This may be specified in the debug control register (DR7) for each address individually, which is another debug register. 
While the direct access of these registers via the `mov` instruction requires a privileged process, they can also be accessed with the PTRACE_WRITE_USER #acr("API") #cite(<ptrace>), that allows the tracer to write fields of the `user` struct#footnote[see glibc source: #link("https://sourceware.org/git/?p=glibc.git;a=blob;f=sysdeps/unix/sysv/linux/x86/sys/user.h"))] in the tracee process, which we use in our implementatio. #cite(<intel-manual>)

Our debugger implements both types of breakpoint, as they have unique strengths and weaknesses.
Software breakpoints are used as the primary execution breakpoint mechanism, because there is no limit to the number of software breakpoints.
Hardware breakpoints, on the other hand, are more flexible in the functionality that they provide, as they can be used to monitor memory access rather than just execution.
However, the number of hardware breakpoints is limited to 4 by the processor architecture.
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
