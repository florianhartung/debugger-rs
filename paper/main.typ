#import "@preview/acrostiche:0.5.1": *
#import "styling.typ": setup

#let td = text(red)[TODO]

#show: setup.with(
  title: "Development of a minimal x86-64 Unix debugger\n in Rust",
  authors: (("Florian Hartung", 6622800), ("Marek Freunscht", 9604914))
)

#let acs = (
  "POSIX": "Portable Operating System Interface",
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
Signals are a feature offered by kernels that comply#footnote("Compliance is meant as partial compliance with the specific signal feature as few kernels are actually fully POSIX-compliant") with the #acr("POSIX") #acr("API") to asynchronously send messages and trigger events between processes.
Commonly known signals include SIGTERM, which terminates a process, SIGSTOP, which stops a process, SIGCONT, which continues a stopped process or SIGTRAP, which signals a breakpoint.

Most signals can be caught and handled by the receiving process with a signal handler.
This signal handler, which can be user-defined, defines a routine which processes the signal and handles it accordingly.
If a user-defined signal handler is not present for a specific signal, the default signal handler is invoked.
The only exceptions for this are SIGKILL and SIGSTOP, which cannot be caught and respectively kill and stop a process.

One usecase for signals in a debugger is controlling the execution of the traced process with signals like SIGSTOP, SIGCONT or SIGKILL.
Additionally, the signals which target the traced process will be delivered to the debugger first.
The debugger can then process these signals and decide which to deliver to the traced process.
This includes SIGTRAP, a signal which is sent when a breakpoint is hit in the executable.

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
// TODO maybe talk about instruction stepping: "Dynamic breakpoints" on a smaller scale for every instruction.

// Requirement for OS interaction
#acrpl("OS") usually isolate processes, their resources, memory, etc. from each other.
However debuggers require access to other processes to be able to debug them.
Thus debuggers require #acrpl("OS") to provide some interface through which they can access and debug these other processes.
For Unix and Unix-like systems this is the `ptrace` syscall, which is short for `process trace`.
Even though it is a single syscall, `ptrace` combines various different commands useful for debuggers to trace other processes.

== Symbols
// Compilation <-> Loss of information
When source code is compiled by a compiler to native machine code, a lot of information about the original source code is changed or completely lost.
Such information may include the names of variables and functions or the layout of stack frames.
Debugging these kinds of programs is time-consuming and most of the time not feasible in practice.
// Motivate debug information
To solve this problem, object/executable formats include sections where compilers can store additional debug information.
Debuggers can read this debug information and use to give users meaningful insight into the debugged process.

// Symbols
One crucial category of information compilers produce are symbols.
Symbols relate string names to addresses in the final object/executable file.
For #ac("ELF") files, symbols reside in a symbol table and their string names in an additional string table.
#ac("ELF") symbols can be of different kinds, such as functions (`STT_FUNC`), sections (`STT_SECTION`), globals (`STT_GLOBAL`), etc. @elf.

// are variables & stack frame debug information also symbols?
// - Short paragraph on DWARF#footnote("DWARF is not an acronym, instead it's a backcroynm") for stack frame debug information

= Requirements
As the scope of this debugger implementation is fairly limited, basic requirements for the debugger are defined:

The set of debuggable programs is restricted to x86-64 #ac("ELF") binaries for Unix and Unix-like systems.
Furthermore, the debugger shall provide the following basic functionalities to run and observe other processes:
The debugger must be able to attach to running processes and run new processes.
For inspecting binaries, the debugger must allow the user to list all function symbols contained inside of a given binary.
Setting breakpoints at arbitrary addresses or function symbols must also be allowed.
Watchpoints that trigger on reads or writes at arbitrary addresses are also required.

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
This section explores the implementation details of the various methods needed to fulfil our requirements.

== Attaching to processes
When beginning to debug a process, there are typically two scenarios for a debugger, attaching to a process that is already running and creating a new process.
Both of these have available ptrace #acfp("API") to use with the debugger.

Attaching to a running process can be done with either PTRACE_ATTACH or PTRACE_SEIZE.
When using PTRACE_ATTACH, the attached process is signaled to stop immediately and the debugger should wait until that stop is completed using the `waitpid` syscall.
The user can then set breakpoints or obtain information about the process while it is stopped.
On the other hand, PTRACE_SEIZE does not stop the attached process and gives the debugger a little more flexibility to do so later with PTRACE_INTERRUPT.
PTRACE_SEIZE also allows the debugger to use some other functionality, like PTRACE_LISTEN.
However, in our implementation we use PTRACE_ATTACH because it is sufficient for our use case and the flexibility and complexity of PTRACE_SEIZE is not needed. @ptrace

Another use case is the user wanting to debug an executable that is not already running in some process.
In this case, the debugger can start the executable and initiate the tracing with ptrace.
This is typically done by forking the debugger process, which leaves the programmer in control of what happens in the child process.
After forking the parent, the child process initiates the tracing using PTRACE_TRACEME, which turns the parent process into the tracer @ptrace.
Finally, the child process can be turned into the desired executable by executing an `execl` syscall.
After this `execl` call executes successfully, a SIGTRAP will be sent to the tracee which stops it and leaves the debugger in control @exec.

Both of these cases are supported in our implementation because they are fundamental for a functional debugger.

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
Instruction stepping is vital for the user to have fine grained control over the program execution after hitting a breakpoint.
This allows the user to advance the execution by a single instruction at a time and inspect the program state after each step.

To implement this, ptrace provides the PTRACE_SINGLESTEP #acr("API") @ptrace. 
The tracee will be stopped after executing one instruction.
This is done internally by the kernel, which sets the trap flag in the x86-64 FLAGS status register.
The CPU will generate a trap after execution which yields control back to the debugger @intel-manual.
In order to ensure that the debugger does not initiate invalid ptrace calls while the tracee is still running, a call to `waitpid` is necessary to wait for the tracee to stop. 

= Debugger Usage
- Show 2-3 example programs and the commands used to interact with the debugger

= Outlook
// TODO Missing features
// TODO What could've been done better
#td

= Conclusion
#td

#bibliography("./bib.yml", style: "institute-of-electrical-and-electronics-engineers")
