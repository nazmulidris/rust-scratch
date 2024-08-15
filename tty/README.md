# tty
<a id="markdown-tty" name="tty"></a>

Table of contents

<!-- TOC -->

- [Prerequisite](#prerequisite)
- [Limitations of using TTY in Linux, and why we like userland terminal emulators PTY](#limitations-of-using-tty-in-linux-and-why-we-like-userland-terminal-emulators-pty)
  - [Kernel TTY 👎🏽](#kernel-tty-)
  - [Userland PTY 👍🏽](#userland-pty-)
- [Examples of using PTY in Linux](#examples-of-using-pty-in-linux)
  - [Using redirection to write to another PTY run command in left terminal, see output in right terminal](#using-redirection-to-write-to-another-pty-run-command-in-left-terminal-see-output-in-right-terminal)
  - [Using redirection to read from another PTY type in left terminal, see it in right terminal](#using-redirection-to-read-from-another-pty-type-in-left-terminal-see-it-in-right-terminal)
  - [Breaking things in raw mode.](#breaking-things-in-raw-mode)
- [Processes, sessions, jobs, PTYs, signals](#processes-sessions-jobs-ptys-signals)
  - [Background information knowledgebase](#background-information-knowledgebase)
    - [File descriptors and processes, ulimit, stdin, stdout, stderr, pipes](#file-descriptors-and-processes-ulimit-stdin-stdout-stderr-pipes)
    - [Unix shells that run in terminals to execute built-in and program commands](#unix-shells-that-run-in-terminals-to-execute-built-in-and-program-commands)
      - [What is the relationship between linux shells, subshells, and fork, exec, and wait patterns?](#what-is-the-relationship-between-linux-shells-subshells-and-fork-exec-and-wait-patterns)
      - [Does exec change the current working directory or affect environment variables in the parent?](#does-exec-change-the-current-working-directory-or-affect-environment-variables-in-the-parent)
      - [Then how does the cd command change the current working directory of a shell?](#then-how-does-the-cd-command-change-the-current-working-directory-of-a-shell)
      - [How do subshells work, in the case where I don't the shell's environment to be affected at all?](#how-do-subshells-work-in-the-case-where-i-dont-the-shells-environment-to-be-affected-at-all)
      - [Deep dive of all this information in video format](#deep-dive-of-all-this-information-in-video-format)
    - [Processes, sessions, jobs, PTYs, signals using C](#processes-sessions-jobs-ptys-signals-using-c)
- [What is /dev/tty?](#what-is-devtty)
  - [How is crossterm built on top of stdio, PTY, etc?](#how-is-crossterm-built-on-top-of-stdio-pty-etc)
  - [How is termion built on top of stdio, PTY, etc?](#how-is-termion-built-on-top-of-stdio-pty-etc)
- [List of signals](#list-of-signals)
- [🦀 Sending and receiving signals in Rust](#-sending-and-receiving-signals-in-rust)
  - [Code to receive signals](#code-to-receive-signals)
  - [Code to send & receive signals](#code-to-send--receive-signals)
- [🦀 Communicating with processes in Rust](#-communicating-with-processes-in-rust)
- [🦀 Process spawning in Rust](#-process-spawning-in-rust)

<!-- /TOC -->

## Prerequisite
<a id="markdown-prerequisite" name="prerequisite"></a>

Read all about TTY history and implementation in Linux
[here](https://www.linusakesson.net/programming/tty/) before reading this repo and doing the
exercises here. There is so much background history and information in this article that is a
prerequisite to understanding anything in this repo. You can read them in this repo
[here](./tty_reading/The%20TTY%20demystified.html) (in case the site is not longer available). To
read them in this repo, run the following commands, and you will be able to access this web page on
[localhost:3000](http://localhost:3000):

```bash
cd tty_reading
serve .
```

## Limitations of using TTY in Linux, and why we like userland terminal emulators (PTY)
<a id="markdown-limitations-of-using-tty-in-linux%2C-and-why-we-like-userland-terminal-emulators-pty" name="limitations-of-using-tty-in-linux%2C-and-why-we-like-userland-terminal-emulators-pty"></a>

### Kernel TTY 👎🏽
<a id="markdown-kernel-tty-%F0%9F%91%8E%F0%9F%8F%BD" name="kernel-tty-%F0%9F%91%8E%F0%9F%8F%BD"></a>

To switch to TTYs in Linux, press:

- <kbd>Ctrl + Alt + F3</kbd> to <kbd>Ctrl + Alt + F4</kbd>. To access two TTYs, one on <kbd>F3</kbd>
  and the other on <kbd>F4</kbd>.
- To switch back to the TTY in which the GUI is running, press <kbd>Ctrl + Alt + F2</kbd>.

In the Linux kernel, the TTY driver and line discipline provide basic line editing (and the
implementation of `cooked` or `raw` mode), and there is no
[`UART`](https://en.wikipedia.org/wiki/Universal_asynchronous_receiver-transmitter) or physical
terminal involved. Instead, a video terminal (a complex state machine including a frame buffer of
characters and graphical character attributes) is emulated in software, and
[[video] rendered to a VGA display](https://www.youtube.com/watch?v=aAuw2EVCBBg).

> So if you run `edi` in a TTY, you will see that the font rendering and colors are different than
> in a GUI terminal emulator. However it still runs.

### Userland PTY 👍🏽
<a id="markdown-userland-pty-%F0%9F%91%8D%F0%9F%8F%BD" name="userland-pty-%F0%9F%91%8D%F0%9F%8F%BD"></a>

The (kernel TTY) console subsystem is somewhat rigid. Things get more flexible (and abstract) if we
move the terminal emulation into userland. This is how `xterm` and its clones work. To facilitate
moving the terminal emulation into userland, while still keeping the TTY subsystem (session
management and line discipline) intact, the pseudo terminal or PTY was invented. And as you may have
guessed, things get even more complicated when you start running pseudo terminals inside pseudo
terminals, aka `screen` or `ssh`.

> The primary use case for r3bl code is to run in this terminal emulator environment in userland and
> not the TTY environment supplied by the Linux kernel itself.

## Examples of using PTY in Linux
<a id="markdown-examples-of-using-pty-in-linux" name="examples-of-using-pty-in-linux"></a>

Each terminal in Linux is associated with a PTY (pseudo terminal). This is the device provided by
each terminal emulator program instance (aka process) that is currently running on the system. Use
the following command to get a list of all PTYs on the system.

```fish
ls /dev/pts
```

Here's sample output:

```
crw--w---- nazmul tty  0 B Wed Jul 17 11:36:35 2024  0
crw--w---- nazmul tty  0 B Wed Jul 17 11:38:32 2024  1
crw--w---- nazmul tty  0 B Wed Jul 17 11:38:06 2024  10
crw--w---- nazmul tty  0 B Wed Jul 17 11:23:20 2024  11
crw--w---- nazmul tty  0 B Sun Jul 14 16:19:36 2024  2
crw--w---- nazmul tty  0 B Mon Jul 15 13:22:48 2024  3
crw--w---- nazmul tty  0 B Tue Jul 16 09:58:08 2024  4
crw--w---- nazmul tty  0 B Wed Jul 17 10:34:48 2024  5
crw--w---- nazmul tty  0 B Wed Jul 17 11:30:32 2024  7
crw--w---- nazmul tty  0 B Wed Jul 17 11:36:36 2024  8
crw--w---- nazmul tty  0 B Wed Jul 17 11:30:48 2024  9
c--------- root   root 0 B Sat Jul 13 18:23:41 2024  ptmx
```

So which PTY is associated with the currently open terminal? Run the following command to get the
TTY number of the currently open terminal.

```fish
set my_tty_id (tty)
echo $my_tty_id
```

It will output something like this:

```
/dev/pts/1
```

Each `/dev/pts/*` is a file. And you can read / write / redirect to these files just like any other
file.

For the following examples, let's assume that you have 2 terminal emulator app windows open. One on
the left, and another one on the right.

```
┌────────────────────────────────┐  ┌────────────────────────────────┐
│                                │  │                                │
│    LEFT TERMINAL               │  │    RIGHT TERMINAL              │
│    /dev/pts/1                  │  │    /dev/pts/2                  │
│                                │  │                                │
└────────────────────────────────┘  └────────────────────────────────┘
```

### Using redirection to write to another PTY (run command in left terminal, see output in right terminal)
<a id="markdown-using-redirection-to-write-to-another-pty-run-command-in-left-terminal%2C-see-output-in-right-terminal" name="using-redirection-to-write-to-another-pty-run-command-in-left-terminal%2C-see-output-in-right-terminal"></a>

Let's say you have 2 terminals open, and one has the PTY number `/dev/pts/1` (on the left) and the
other has the TTY number `/dev/pts/2` (on the right).

From the left PTY `/dev/pts/1`, you can write to the right PTY `/dev/pts/2` using the following
command, and you will see "Hello, World!" in the right PTY.

```fish
# Run this in left terminal /dev/pts/1
echo "Hello, World!" > /dev/pts/2 # You will see this in the right terminal /dev/pts/2
```

### Using redirection to read from another PTY (type in left terminal, see it in right terminal)
<a id="markdown-using-redirection-to-read-from-another-pty-type-in-left-terminal%2C-see-it-in-right-terminal" name="using-redirection-to-read-from-another-pty-type-in-left-terminal%2C-see-it-in-right-terminal"></a>

From the right PTY `/dev/pts/2` you can read input from the left PTY `/dev/pts/1` using the
following command.

```fish
# Run this in right terminal /dev/pts/2
cat /dev/pts/1
```

Type the following in the left PTY.

```fish
# Run this in left terminal /dev/pts/1
abcdefgh
```

You will see the following output in the right PTY: `abcdefgh`.

### Breaking things in raw mode.
<a id="markdown-breaking-things-in-raw-mode." name="breaking-things-in-raw-mode."></a>

On the **right** terminal, run the following commands.

```fish
vi &
jobs
```

Here you will see the job number of the `vi` process. And you will see that it is in the background.

If you run `ps l` you will see the states of all the processes that are running. If you run `ps -l`
you will this information on just the processes spawned in the right terminal. For example:

```
F S   UID     PID    PPID  C PRI  NI ADDR SZ WCHAN  TTY          TIME CMD
0 S  1000  540327  540177  0  80   0 - 62854 futex_ pts/8    00:00:01 fish
0 T  1000  554675  540327  0  80   0 -  3023 do_sig pts/8    00:00:00 vi
4 R  1000  554850  540327  0  80   0 -  3478 -      pts/8    00:00:00 ps
```

Now if you bring `vi` to the foreground by running `fg`. The `vi` process is now in raw mode, and
the shell is no longer interpreting the input. It won't know what to do with input that comes in
over `stdin`.

Run `echo "foo" > /dev/pts/2` in the **left** terminal, you will see that the `vi` process gets
messed up, since it doesn't really interpret that input (as it's reading directly from keyboard and
mouse). However, the shell will send that output to `vi` and it's UI will be messed up. The same
thing happens if you use `micro` or `nano`.

```
┌────────────────────────────────┐  ┌────────────────────────────────┐
│    LEFT TERMINAL               │  │    RIGHT TERMINAL              │
│    /dev/pts/1                  │  │    /dev/pts/2                  │
│                                │  │                                │
│                                │  │  > vi &                        │
│                                │  │  > jobs                        │
│                                │  │  > fg                          │
│  > echo "foo" > /dev/pts/2     │  │  > # vi is messed up           │
└────────────────────────────────┘  └────────────────────────────────┘
```

To terminate the `vi` process (or many of them), run `killall -9 vi`. That sends the `SIGKILL`
signal to all the `vi` processes.

## Processes, sessions, jobs, PTYs, signals
<a id="markdown-processes%2C-sessions%2C-jobs%2C-ptys%2C-signals" name="processes%2C-sessions%2C-jobs%2C-ptys%2C-signals"></a>

Let's say in a new terminal emulator program `xterm`, and then you run the following commands in
`fish`:

```fish
cat &
ls | sort
```

What happens here? What sessions and jobs are created? What about the pipe?

There are 4 jobs:

1. The job that runs `xterm` itself.

- This does not have any `stdin`, `stdout`, `stderr` `fd`s associated with it.
- This does not have a PTY associated with it.

2. The job that runs `bash` itself.

- This has `stdin`, `stdout`, `stderr` (from `xterm`), lets say, `/dev/pts/0`.
- This has a PTY associated with it, lets say, `/dev/pts/0`.

3. The job that runs `cat` in the background.

- This has `stdin`, `stdout`, `stderr` (from `xterm`), `/dev/pts/0`.
- This has a PTY associated with it, `/dev/pts/0`.

4. The job that runs `ls | sort` pipeline. This job has 2 processes inside of it which are spawned
   in parallel due to the pipe: 4.1. The process that runs `ls`.
   - This has `stdin`, `stderr` (from `xterm`), `/dev/pts/0`.
   - Due to the pipe `stdout` is set to `pipe0`.
   - This has a PTY associated with it, `/dev/pts/0`. 4.2. The process that runs `sort`.
   - This has `stdout`, `stderr` (from `xterm`), `/dev/pts/0`.
   - Due to the pipe, this has `stdin` set to `pipe0`.
   - This has a PTY associated with it, `/dev/pts/0`.

The basic idea is that every pipeline is a job, because every process in a pipeline should be
manipulated (stopped, resumed, killed) simultaneously. That's why `kill` allows you to send signals
to entire process groups. By default, `fork` places a newly created child process in the same
process group as its parent, so that e.g. a <kbd>^C</kbd> from the keyboard will affect both parent
and child. But the shell, as part of its session leader duties, creates a new process group every
time it launches a pipeline.

The TTY driver keeps track of the foreground process group id, but only in a passive way. The
session leader has to update this information explicitly when necessary. Similarly, the TTY driver
keeps track of the size of the connected terminal, but this information has to be updated
explicitly, by the terminal emulator or even by the user.

Several processes have `/dev/pts/0` attached to their standard input. With these constrains:

1. Only the foreground job (the `ls | sort` pipeline) will receive input from the TTY.
2. Likewise, only the foreground job will be allowed to write to the TTY device (in the default
   configuration).
3. If the `cat` process were to attempt to write to the TTY, the kernel would suspend it using a
   signal.

### Background information (knowledgebase)
<a id="markdown-background-information-knowledgebase" name="background-information-knowledgebase"></a>

The following sections are a deep live of the Linux kernel and how it works with processes, file
descriptors, shells, and PTYs.

#### File descriptors and processes, ulimit, stdin, stdout, stderr, pipes
<a id="markdown-file-descriptors-and-processes%2C-ulimit%2C-stdin%2C-stdout%2C-stderr%2C-pipes" name="file-descriptors-and-processes%2C-ulimit%2C-stdin%2C-stdout%2C-stderr%2C-pipes"></a>

Here's a
[[video] What's behind a file descriptor in Linux? Also, i/o redirection with `dup2`.](https://youtu.be/rW_NV6rf0rM?si=wcEkGPXnXzKeBn_G)
that goes into file descriptors, pipes, and process forking in Linux.

#### Unix shells (that run in terminals to execute built-in and program commands)
<a id="markdown-unix-shells-that-run-in-terminals-to-execute-built-in-and-program-commands" name="unix-shells-that-run-in-terminals-to-execute-built-in-and-program-commands"></a>

##### What is the relationship between linux shells, subshells, and fork, exec, and wait patterns?
<a id="markdown-what-is-the-relationship-between-linux-shells%2C-subshells%2C-and-fork%2C-exec%2C-and-wait-patterns%3F" name="what-is-the-relationship-between-linux-shells%2C-subshells%2C-and-fork%2C-exec%2C-and-wait-patterns%3F"></a>

In Linux, shells, subshells, and the fork-exec-wait pattern are interconnected concepts that play a
crucial role in process management and execution. Here's how they relate to each other:

1. **Shells**: A shell is a command-line interpreter that allows users to interact with the
   operating system. Shells provide a way for users to run commands, launch programs, and manage
   processes. Examples of popular shells in Linux include Bash, Zsh, and Fish.

2. **Fork-Exec-Wait Pattern**: This pattern is commonly used in shell scripting to spawn new
   processes and manage their execution. By forking a new process, executing a different program in
   the child process, and then waiting for the child process to finish, the shell can run multiple
   commands concurrently and coordinate their execution. If the parent does not wait for the child
   process to finish, the child is a zombie process.

   - **Fork**: When a process wants to execute a new program, it creates a copy of itself using the
     `fork()` system call. This creates a new process (child process) that is an exact copy of the
     original process (parent process) at the time of the `fork()` call. It needs to do this since
     `exec()`, which is called next, will swap the program binaries of the process which calls it!
     If it doesn't spawn a child, then the parent will cease to exist in memory after `exec()` is
     called.
   - **Exec**: After forking, the child process uses the `exec()` system call to replace its memory
     space with a new program. This allows the child process to run a different program than the
     parent process. The `exec()` system call loads the new program into the child process's memory
     and starts its execution.
   - **Wait**: After forking and executing a new program, the parent process may need to wait for
     the child process to finish its execution. The parent process can use the `wait()` system call
     to wait for the child process to terminate. This ensures that the parent process does not
     continue its execution until the child process has completed its task.

3. **Subshells**: A subshell is a separate instance of the shell that is spawned to execute a
   command or a group of commands. Subshells are created within the parent shell and can be used to
   run commands in a separate environment without affecting the parent shell.

> You can learn more about each of these system calls on your Linux machine simply by running
> `bash -c "man fork"`, `bash -c "man exec"`, and `bash -c "man wait"`. The `bash -c` is needed only
> if you're running some other shell like `fish` and not `bash`.

The relationship between these concepts is as follows:

- A shell process (the parent) creates a clone of their "self" process using `fork()`, called a
  child process. And then they use `exec()` to replace the memory space of the child process with a
  new program. Then the parent process waits for the child process to finish.
- The fork-exec-wait pattern is a common technique used in shells and subshells to spawn new
  processes, execute programs, and coordinate their execution.
- Shells can create subshells to run commands in a separate environment. For example if you want to
  run `cd` (which is a shell built-in command and not a external "program" command) and you don't
  want this to affect the parent shell, you can run it in a subshell.

Overall, these concepts work together to facilitate process management, execution, and command
interpretation in a Linux environment.

##### Does `exec()` change the current working directory or affect environment variables in the parent?
<a id="markdown-does-exec-change-the-current-working-directory-or-affect-environment-variables-in-the-parent%3F" name="does-exec-change-the-current-working-directory-or-affect-environment-variables-in-the-parent%3F"></a>

Running `exec()` on the child process does not change the current working directory of the parent
process.

When a process calls the `exec()` system call in Linux, it replaces its current image with a new
program. The `exec()` system call loads a new program into the process's memory space and starts its
execution.

Here's how `exec()` affects the current working directory and environment variables:

1. **Current Working Directory**: When a child process calls `exec()`, the current working directory
   of the parent process remains unchanged. The new program loaded by `exec()` will start executing
   with the same working directory as the original process. Therefore, the current working directory
   of the parent process is not affected by the child's `exec()` call.

2. **Environment Variables**: The environment of the new program loaded by `exec()` can be set
   explicitly by the program itself or inherited from the parent process. If the new program does
   not explicitly modify the environment variables, it will inherit the environment variables from
   the parent process. Any changes made to environment variables in the child process after the
   `exec()` call will not affect the environment variables of the parent process.

##### Then how does the `cd` command change the current working directory of a shell?
<a id="markdown-then-how-does-the-cd-command-change-the-current-working-directory-of-a-shell%3F" name="then-how-does-the-cd-command-change-the-current-working-directory-of-a-shell%3F"></a>

The `cd` command is a special command called a "shell built-in" command; there are about ~70 of
these. `echo`, `source` are examples of these "built-in" commands. These commands are built into the
shell itself. It is not a "external executable program" command like `ls`. So a shell does not have
to `fork` and `exec` to run these commands. The shell runs them inside of it's own "parent" process,
which affects "self".

If you think about it, `cd` has to be a built-in command since we know that child processes can't
affect the environment of the parent process, and the current working directory is part of a
process' environment.

> Watch this [video](https://youtu.be/GA2mIUQq48s?si=Sfbpre-MeNXlND_b&t=820) to get an understanding
> of `built-in` commands vs `external executable program` commands.

Let's say you want to `cd` into a folder but you don't want this to affect the parent shell. How do
you do this? This is where subshells come into play. If you're using `fish`, then a subshell is like
running `fish -c` with whatever is typed in between `""`.

##### How do subshells work, in the case where I don't the shell's environment to be affected at all?
<a id="markdown-how-do-subshells-work%2C-in-the-case-where-i-don't-the-shell's-environment-to-be-affected-at-all%3F" name="how-do-subshells-work%2C-in-the-case-where-i-don't-the-shell's-environment-to-be-affected-at-all%3F"></a>

In a Linux shell, a subshell is a separate instance of the shell that is spawned to execute a
command or a group of commands. When a user types a command to execute, the shell creates a subshell
to run that command.

Subshells are useful for various purposes, such as:

1. Running commands in a separate environment without affecting the parent shell.
2. Running commands in parallel to improve performance.
3. Running commands that need to be isolated from the parent shell.

Subshells are typically created using parentheses `()` in `fish` or the `$(...)` syntax in `bash`.
For example, when you run a command within parentheses like this:

```bash
(command1; command2)
```

The commands `command1` and `command2` will be executed in a subshell. Once the commands finish
executing, the subshell exits, and the parent shell continues its operation. If you run the `cd ..`
command in a subshell, it won't change the current working directory of the shell!

Subshells are used to manage sessions and jobs and pipelines. Things like foreground and background
jobs are managed using subshells. And signals are sent to processes using subshells in a pipeline.

> Watch this [video](https://youtu.be/N8kT2XRNEAg?si=iiv6i3mO6Lxi8qb1&t=60) to get an understanding
> of subshells, signals, jobs, pipelines, etc.

##### Deep dive of all this information in video format
<a id="markdown-deep-dive-of-all-this-information-in-video-format" name="deep-dive-of-all-this-information-in-video-format"></a>

Here's a
[[video playlist] Unix terminals and shells](https://www.youtube.com/playlist?list=PLFAC320731F539902)
that goes into details about shells, subshells, forking, exec (command), and wait works.

#### Processes, sessions, jobs, PTYs, signals using C
<a id="markdown-processes%2C-sessions%2C-jobs%2C-ptys%2C-signals-using-c" name="processes%2C-sessions%2C-jobs%2C-ptys%2C-signals-using-c"></a>

Here are some videos on forking processes, zombies, and signals in C:

- [[video] Create new process in C w/ `fork()`](https://www.youtube.com/watch?v=ss1-REMJ9GA)
- [[video] Send signals to processes in C w/ `kill()`, `signal()`, `sigaction()`](https://www.youtube.com/watch?v=83M5-NPDeWs)
- [[video] Zombie processes in C](https://www.youtube.com/watch?v=xJ8KenZw2ag)
- [[video] Stop process becoming zombie in C](https://www.youtube.com/watch?v=_5SCtRNnf9U)

## What is /dev/tty?
<a id="markdown-what-is-%2Fdev%2Ftty%3F" name="what-is-%2Fdev%2Ftty%3F"></a>

`/dev/tty` is a special file in Unix-like operating systems that represents the controlling terminal
of the current process. It is a synonym for the controlling terminal device file associated with the
process.

The controlling terminal is the terminal that is currently active and connected to the process,
allowing input and output interactions. It provides a way for processes to interact with the user
through the terminal interface.

The `/dev/tty` file can be used to read from or write to the controlling terminal.

In each process, `/dev/tty` is a synonym for the controlling terminal associated with the process
group of that process, if any. It is useful for programs or shell procedures that wish to be sure of
writing messages to or reading data from the terminal no matter how output has been redirected. It
can also be used for applications that demand the name of a file for output, when typed output is
desired and it is tiresome to find out what terminal is currently in use.

1. Definition from
   [IEEE Open Group Base Specifications for POSIX](https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap10.html).
2. You can see it used in `crossterm` crate
   [here](https://github.com/crossterm-rs/crossterm/blob/master/src/terminal/sys/file_descriptor.rs#L143).
3. Here's more info about this on
   [baeldung.com](https://www.baeldung.com/linux/monitor-keyboard-drivers#devtty).

### How is crossterm built on top of stdio, PTY, etc?
<a id="markdown-how-is-crossterm-built-on-top-of-stdio%2C-pty%2C-etc%3F" name="how-is-crossterm-built-on-top-of-stdio%2C-pty%2C-etc%3F"></a>

The [`crossterm`](https://github.com/crossterm-rs/crossterm) crate is built on top of Tokio's
[`mio`](https://docs.rs/mio/latest/mio/guide/index.html) crate, which uses Linux
[`epoll`](https://man7.org/linux/man-pages/man7/epoll.7.html) to work with file descriptors in an
async manner.

- Here's [`mio`'s `Poll`](https://docs.rs/mio/latest/mio/struct.Poll.html) using `epoll` under the
  hood.
- Here's an [example](https://docs.rs/mio/latest/mio/guide/index.html) of `mio` using Linux `epoll`
  in order to read from a file descriptor in an async manner.

> Linux `epoll` is able to work with `stdio` file descriptors (ie, `stdin`, `stdout`, `stderr`), as
> well as other file descriptors (network and file system). However, for throughput and performance
> (by reducing context switching and being efficient with buffers that hold IO data), Linux
> [`io_uring`](https://en.wikipedia.org/wiki/Io_uring) might be more suitable.

Here are some links to learn more about how `crossterm` works with `PTY`s and `stdio`:

- [Get a file descriptor for the TTY `tty_fd()`](https://github.com/crossterm-rs/crossterm/blob/master/src/terminal/sys/file_descriptor.rs#L143).
  It uses [`rustix::stdio::stdin()`](https://docs.rs/rustix/latest/rustix/stdio/fn.stdin.html) by
  default and falls back on `/dev/tty/`.
- This `fd` is used by
  [`UnixInternalEventSource`](https://github.com/crossterm-rs/crossterm/blob/master/src/event/source/unix/mio.rs#L35)
  which creates a `mio::Poll` object for the `fd`. This `Poll` object uses `epoll` under the hood.
  The
  [`EventSource` trait impl for `UnixInternalEventSource`](https://github.com/crossterm-rs/crossterm/blob/master/src/event/source/unix/mio.rs#L72)
  is used to actually
  [read](https://github.com/crossterm-rs/crossterm/blob/master/src/terminal/sys/file_descriptor.rs#L75)
  the bytes from the `fd` (using
  [`rustix::io::read()`](https://docs.rs/rustix/latest/rustix/io/fn.read.html)).
- Once a `Poll` has been created, a
  [`mio::Poll::registry()`](https://docs.rs/mio/latest/mio/struct.Registry.html) must be used to
  tell the OS to listen for events on the `fd`. A
  [source and interest must be registered](https://docs.rs/mio/latest/mio/guide/index.html#2-registering-event-source)
  with the registry next:
  - The `fd` [implements](https://docs.rs/mio/latest/mio/unix/struct.SourceFd.html) the
    [`Source` trait](https://docs.rs/mio/latest/mio/event/trait.Source.html) which
    [allows](https://docs.rs/mio/latest/mio/event/trait.Source.html#implementing-eventsource) `mio`
    to listen for events on the `fd`.
  - An `Interest::READABLE` must also be "registered" with the `registry`. For eg, for `stdin`, this
    tells the OS to listen for input from the keyboard, and wake the `Poll` when this is ready.
  - A `Token` is supplied that can be used when polling for events to see if they're available on
    the source. This happens in the
    [`loop`](https://docs.rs/mio/latest/mio/guide/index.html#3-creating-the-event-loop) that calls
    `poll()` to fill an `Event` buffer. If an event in this buffer matches the `Token`, then the
    `fd` is ready for reading.

You can see all the steps (outlined above) in action, in the following crates:

- [Guide in `mio` docs](https://docs.rs/mio/latest/mio/guide/index.html).
- [`mio.rs` file in `crossterm`](https://github.com/crossterm-rs/crossterm/blob/master/src/event/source/unix/mio.rs).
- This [PR](https://github.com/nazmulidris/crossterm/pull/1) in my fork of `crossterm` has
  `println!` traces so you can see how `mio` is used under the hood by `crossterm` to read from
  `stdin`.

### How is termion built on top of stdio, PTY, etc?
<a id="markdown-how-is-termion-built-on-top-of-stdio%2C-pty%2C-etc%3F" name="how-is-termion-built-on-top-of-stdio%2C-pty%2C-etc%3F"></a>

Here's a [PR](https://github.com/nazmulidris/termion/pull/1) to explore the examples in `termion`
crate. This is a beautifully simple and elegant crate that is much simpler than `crossterm`. It
simply uses the standard library and a few other crates to get bytes from `stdin` and write bytes to
`stdout`. It does not use `mio`, and neither does it support `async` `EventStream`. There is an
"async mode", which simply spawns another thread and uses a channel to send events to the main
thread.

## List of signals
<a id="markdown-list-of-signals" name="list-of-signals"></a>

Here are the reference docs on signals:

- [gnu libc termination signals](https://www.gnu.org/software/libc/manual/html_node/Termination-Signals.html)
- [gnu libc job control signals](https://www.gnu.org/software/libc/manual/html_node/Job-Control-Signals.html)

Here is a list of all the signals that a process might get:
[signals](https://www.linusakesson.net/programming/tty/#signal-madness:~:text=using%20a%20signal.-,Signal%20madness,-Now%20let%27s%20take).

You can also get a list of them using `kill -l`. It is different for `fish` and `bash`. However,
under the hood, the Linux kernel uses the same signal numbers for all shells.

<!-- cSpell:disable -->

````fish
$ fish -c "kill -l"
HUP INT QUIT ILL TRAP ABRT BUS FPE KILL USR1 SEGV USR2 PIPE ALRM TERM STKFLT
CHLD CONT STOP TSTP TTIN TTOU URG XCPU XFSZ VTALRM PROF WINCH POLL PWR SYS

```bash
$ bash -c "kill -l"
 1) SIGHUP	 2) SIGINT	 3) SIGQUIT	 4) SIGILL	 5) SIGTRAP
 6) SIGABRT	 7) SIGBUS	 8) SIGFPE	 9) SIGKILL	10) SIGUSR1
11) SIGSEGV	12) SIGUSR2	13) SIGPIPE	14) SIGALRM	15) SIGTERM
16) SIGSTKFLT	17) SIGCHLD	18) SIGCONT	19) SIGSTOP	20) SIGTSTP
21) SIGTTIN	22) SIGTTOU	23) SIGURG	24) SIGXCPU	25) SIGXFSZ
26) SIGVTALRM	27) SIGPROF	28) SIGWINCH	29) SIGIO	30) SIGPWR
31) SIGSYS	34) SIGRTMIN	35) SIGRTMIN+1	36) SIGRTMIN+2	37) SIGRTMIN+3
38) SIGRTMIN+4	39) SIGRTMIN+5	40) SIGRTMIN+6	41) SIGRTMIN+7	42) SIGRTMIN+8
43) SIGRTMIN+9	44) SIGRTMIN+10	45) SIGRTMIN+11	46) SIGRTMIN+12	47) SIGRTMIN+13
48) SIGRTMIN+14	49) SIGRTMIN+15	50) SIGRTMAX-14	51) SIGRTMAX-13	52) SIGRTMAX-12
53) SIGRTMAX-11	54) SIGRTMAX-10	55) SIGRTMAX-9	56) SIGRTMAX-8	57) SIGRTMAX-7
58) SIGRTMAX-6	59) SIGRTMAX-5	60) SIGRTMAX-4	61) SIGRTMAX-3	62) SIGRTMAX-2
63) SIGRTMAX-1	64) SIGRTMAX
````

<!-- cSpell:enable -->

Here are some important ones.

1. `SIGHUP`

- Default action: Terminate
- Possible actions: Terminate, Ignore, Function call
- `SIGHUP` is sent by the UART driver to the entire session when a hangup condition has been
  detected. Normally, this will kill all the processes. Some programs, such as `nohup` and `screen`,
  detach from their session (and TTY), so that their child processes won't notice a hangup.

2. `SIGINT`

- Default action: Terminate
- Possible actions: Terminate, Ignore, Function call
- `SIGINT` is sent by the TTY driver to the current foreground job when the interactive attention
  character (typically <kbd>^C</kbd>, which has ASCII code 3) appears in the input stream, unless
  this behavior has been turned off. Anybody with access permissions to the TTY device can change
  the interactive attention character and toggle this feature; additionally, the session manager
  keeps track of the TTY configuration of each job, and updates the TTY whenever there is a job
  switch.

3. `SIGQUIT`

- Default action: Core dump
- Possible actions: Core dump, Ignore, Function call
- `SIGQUIT` works just like SIGINT, but the quit character is typically <kbd>^\</kbd> and the
  default action is different.

4. `SIGPIPE`

- Default action: Terminate
- Possible actions: Terminate, Ignore, Function call
- The kernel sends `SIGPIPE` to any process which tries to write to a pipe with no readers. This is
  useful, because otherwise jobs like `yes | head` would never terminate.

5. `SIGCHLD`

- Default action: Ignore
- Possible actions: Ignore, Function call
- When a process dies or changes state (stop/continue), the kernel sends a `SIGCHLD` to its parent
  process. The `SIGCHLD` signal carries additional information, namely the process id, the user id,
  the exit status (or termination signal) of the terminated process and some execution time
  statistics. The session leader (shell) keeps track of its jobs using this signal.

6. `SIGSTOP`

- Default action: Suspend
- Possible actions: Suspend
- This signal will unconditionally suspend the recipient, i.e. its signal action can't be
  reconfigured. Please note, however, that `SIGSTOP` isn't sent by the kernel during job control.
  Instead, <kbd>^Z</kbd> typically triggers a `SIGTSTP`, which can be intercepted by the
  application. The application may then e.g. move the cursor to the bottom of the screen or
  otherwise put the terminal in a known state, and subsequently put itself to sleep using `SIGSTOP`.

7. `SIGCONT`

- Default action: Wake up
- Possible actions: Wake up, Wake up + Function call
- `SIGCONT` will un-suspend a stopped process. It is sent explicitly by the shell when the user
  invokes the `fg` command. Since `SIGSTOP` can't be intercepted by an application, an unexpected
  `SIGCONT` signal might indicate that the process was suspended some time ago, and then
  un-suspended.

8. `SIGTSTP`

- Default action: Suspend
- Possible actions: Suspend, Ignore, Function call
- `SIGTSTP` works just like `SIGINT` and `SIGQUIT`, but the magic character is typically
  <kbd>^Z</kbd> and the default action is to suspend the process.

9. `SIGTTIN`

- Default action: Suspend
- Possible actions: Suspend, Ignore, Function call
- If a process within a background job tries to read from a TTY device, the TTY sends a `SIGTTIN`
  signal to the entire job. This will normally suspend the job.

10. `SIGTTOU`

- Default action: Suspend
- Possible actions: Suspend, Ignore, Function call
- If a process within a background job tries to write to a TTY device, the TTY sends a `SIGTTOU`
  signal to the entire job. This will normally suspend the job. It is possible to turn off this
  feature on a per-TTY basis.

11. `SIGWINCH`

- Default action: Ignore
- Possible actions: Ignore, Function call
- As mentioned, the TTY device keeps track of the terminal size, but this information needs to be
  updated manually. Whenever that happens, the TTY device sends `SIGWINCH` to the foreground job.
  Well-behaving interactive applications, such as editors, react upon this, fetch the new terminal
  size from the TTY device and redraw themselves accordingly.

## 🦀 Sending and receiving signals in Rust
<a id="markdown-%F0%9F%A6%80-sending-and-receiving-signals-in-rust" name="%F0%9F%A6%80-sending-and-receiving-signals-in-rust"></a>

| crate                                     | recv | send  |
| ----------------------------------------- | ---- | ----- |
| https://docs.rs/tokio/latest/tokio/signal | 🟢   | 🔴    |
| https://crates.io/crates/ctrlc            | 🟢   | 🔴    |
| https://crates.io/crates/signal-hook      | 🟢   | 🟢 \* |
| https://docs.rs/nix/latest/nix/           | 🟢   | 🟢    |

> \*: Via
> [`signal_hook::low_level::raise`](https://docs.rs/signal-hook/latest/signal_hook/low_level/fn.raise.html).

### Code to receive signals
<a id="markdown-code-to-receive-signals" name="code-to-receive-signals"></a>

`tokio` has limited handling of signals. You can only receive certain signals, not send them. Here's
an [example](https://github.com/nazmulidris/rust-scratch/blob/main/tty/src/receive_signal.rs#L18) of
how to receive signals using `tokio`.

Other choices to receive signals:

- [`ctrlc`](https://crates.io/crates/ctrlc)
- [`signal-hook`](https://crates.io/crates/signal-hook)

### Code to send & receive signals
<a id="markdown-code-to-send-%26-receive-signals" name="code-to-send-%26-receive-signals"></a>

Here's an
[example](https://github.com/nazmulidris/rust-scratch/blob/main/tty/src/send_and_receive_signal.rs)
of using `signal-hook` and `signal-hook-tokio`

## 🦀 Communicating with processes in Rust
<a id="markdown-%F0%9F%A6%80-communicating-with-processes-in-rust" name="%F0%9F%A6%80-communicating-with-processes-in-rust"></a>

In `tokio` a good place to start is
[`tokio::process`](https://docs.rs/tokio/latest/tokio/process/index.html) which mimics the
`std::process` module.

Here are code examples of how to communicate with processes in Rust asynchronously (in this repo):

- [Example of running `echo` process](https://github.com/nazmulidris/rust-scratch/blob/main/tty/src/async_command_exec_1.rs)
- [Example of piping input to `cat` process programmatically](https://github.com/nazmulidris/rust-scratch/blob/main/tty/src/async_command_exec_2.rs)
- [Example of programmatically providing input into `stdin` and getting output from `stdout` of a process](https://github.com/nazmulidris/rust-scratch/blob/main/tty/src/async_command_exec_3.rs)
- [Example of programmatically piping the output of one process into another](https://github.com/nazmulidris/rust-scratch/blob/main/tty/src/async_command_exec_4.rs)

This example is in the `r3bl_terminal_async` repo:

- [Example of using `r3bl_terminal_async` to send commands to a long running `bash` child process](https://github.com/r3bl-org/r3bl-open-core/blob/main/terminal_async/examples/shell_async.rs)

## 🦀 Process spawning in Rust
<a id="markdown-%F0%9F%A6%80-process-spawning-in-rust" name="%F0%9F%A6%80-process-spawning-in-rust"></a>

Here's the [procspawn crate](https://crates.io/crates/procspawn) that we can use for this.

- [Example of using `procspawn` to spawn
  processes](https://github.com/nazmulidris/rust-scratch/blob/main/tty/src/procspawn.rs)
