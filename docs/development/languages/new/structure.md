# How to structure Fuchsia support for a language

This document describes the structure languages typically when supporting
Fuchsia.

## System calls

The lowest level of Fuchsia support in a language provides access to the
[Zircon system calls](/docs/reference/syscalls/).
Exposing these system calls lets programs written in the language interact with
the kernel and, transitively, with the rest of the system.

Programs cannot issue system calls directly. Instead, they make system calls by
calling functions in the [vDSO](/docs/concepts/kernel/vdso.md),
which is loaded into newly created processes by their creator.

The public entry points for the vDSO are defined in
[//zircon/vdso](/zircon/vdso/).
This file is processed by the [kazoo](/docs/concepts/kernel/vdso.md#kazoo_tool)
tool.

## Async

The vast majority of Fuchsia programs act as *servers*. After startup, they wait
in an event loop to receive messages, process those messages (potentially by
sending messages to other processes), and then go back to sleep in their event
loop.

The fundamental building block for event loops in Fuchsia is the
[port](/docs/reference/kernel_objects/port.md)
object. A thread can sleep in a port using
[`zx_port_wait`](/docs/reference/syscalls/port_wait.md).
When the kernel wakes up the thread, the kernel provides a *packet*, which is a
data structure that describes why the kernel woke up the thread.

Typically, each thread has a single port object in which it sleeps, which a
significant amount of code written in your language will need to interact with.
Rather than expose the port directly, language mantainers usually provide
a library that abstracts over a port and provides asynchronous wait operations.

Most asynchronous wait operations bottom out in
[`zx_object_wait_async`](/docs/reference/syscalls/object_wait_async.md). Typically, the `port` and `key`
arguments are provided by the library and the `handle` and `signals`
arguments are provided by the clients. When establishing a wait, the clients
also typically provide an upcall (e.g., a closure) for the library to invoke
when the wait completes, at which point the library uses the `key` to recover
the upcall (e.g., from a hash table).

No additional kernel object is needed to wake a thread up from another thread.
You can wake up a thread by simply queuing a user packet to the thread's port
using
[zx_port_queue](/docs/reference/syscalls/port_queue.md).

### Examples

* [async](/zircon/system/ulib/async)
  (C and C++)
* [fuchsia-async](/src/lib/fuchsia-async/) (Rust)
* [zxwait](https://fuchsia.googlesource.com/third_party/go/+/HEAD/src/syscall/zx/zxwait/) (Go)

## FIDL

The Zircon kernel itself largely provides memory management, scheduling, and
interprocess communication. Rather than being provided directly by the kernel,
the bulk of the system interface is actually provided through interprocess
communication, typically using [channels](/docs/reference/kernel_objects/channel.md).
The protocols used for interprocess communication are defined in
[Fuchsia Interface Definition Language (FIDL)](../fidl/README.md).

FIDL support for a language typically involves two pieces:

1. A language-specific backend for the FIDL compiler that generates code in the
   target language.
2. A support library written in the target language that is used by the code
   generated by the FIDL compiler.

These pieces are usually not built into the language implementation or runtime.
Instead, the libraries are part of the developer's program and versioned
independently from the language runtime. The stable interface between the
program and the language runtime should be the *system calls* rather than the
FIDL protocols so that developers can pick the versions of their FIDL
protocols and the version of their language runtimes independently.

In some cases, the language runtime might need to use FIDL internally. If that
happens, prefer to hide this implementation detail from the developer's program
if possible in your language. The developer might wish to use newer versions of
the same FIDL protocols without conflicting with the version used internally by
the language runtime.

### FIDL compiler backend

The [FIDL compiler](/tools/fidl/fidlc/)
has a single frontend that is used for all languages and multiple backends that
support a diverse assortment of languages. The frontend produces a
[JSON intermediate format][json-ir]
that is consumed by the language-specific backends.

You should create a new backend for the FIDL compiler for your language. The
backend can be written in whatever language you prefer. Typically, language
maintainers choose either Go or the target language. Official backends are named
`fidlgen_<lang>` and can be found in [//tools/fidl](/tools/fidl).

### Generated code

The generated FIDL code varies substantially from one language to another.
Typically the generated code will contain the following types of code:

* Data structure definitions that represent the data structures defined in the
  [FIDL language][fidl-language].
* A codec that can serialize and deserialize these data structure into and from
  the [FIDL wire format][fidl-wire-format].
* Stub objects that represent the server end of a FIDL protocol. Typically,
  stub object have a *dispatch* method that deserializes a message read from a
  Zircon channel and perform an indirect jump into an implementation of the
  method specified by the message's *ordinal*.
* Proxy objects that represent the client end of a FIDL protocol. Typically,
  method calls on proxy objects result in a message being serialized and
  sent over a Zircon channel. Typically, proxy object have a *dispatch* for
  event messages similar to the dispatch method found in stubs for request
  messages.

Some languages offer multiple options for some of these types of generated code.
For example, a common pattern is to offer both *synchronous* and *asynchronous*
proxy objects. The synchronous proxies make use of
[`zx_channel_call`](/docs/reference/syscalls/channel_call.md)
to efficiently write a message, block waiting for a response, and then read the
response, whereas asynchronous proxies use
[`zx_channel_write`](/docs/reference/syscalls/channel_write.md),
[`zx_object_wait_async`](/docs/reference/syscalls/object_wait_async.md),
and
[`zx_channel_read`](/docs/reference/syscalls/channel_read.md)
to avoid blocking on the remote end of the channel.

Generally, we prefer to use *asynchronous* code whenever possible. Many FIDL
protocols are designed to be used in an asynchronous, feed-forward pattern.

### Support library

When designing the generated code for your language, pay particular attention to
binary size. Sophisticated program often interact with a large number of FIDL
protocols, each of which might define many data structures and protocols.

One important technique for reducing binary size is to factor as much code as
possible into a FIDL *support library*. For example, the C bindings, all the
serialization and deserialization logic is performed by a routine in a support
library. The generate code contains only a table that describes the wire format
in a compact form.

Typically, the support library is layered on top of the async library, which
itself has no knowledge of FIDL. For example, most support libraries contain a
*reader* object, which manages the asynchronous waiting and reading operations
on channels. The generated code can then be restricted to serialization,
deserialization, and dispatch.

 * [C](/zircon/system/ulib/fidl)
 * [C++](/sdk/lib/fidl/cpp/)
 * [Rust](/src/lib/fidl/rust/fidl)
 * [Dart](https://fuchsia.googlesource.com/topaz/+/HEAD/public/dart/fidl/)
 * [Go](https://fuchsia.googlesource.com/third_party/go/+/HEAD/src/syscall/zx/fidl/)

## POSIX-style IO

POSIX-style IO operations (e.g., `open`, `close`, `read`, and `write`) are
layered on top of FIDL. If your language has C interop, you can use the
[FDIO library](/sdk/lib/fdio),
which translates familiar POSIX operations into the underlying `fuchsia.io` FIDL
protocol. If your language does not have C interop, you will need to interface
directly with `fuchsia.io` to provide POSIX-style IO.

You can recover the underlying Zircon handles for file descriptors using [`lib/fdio/unsafe.h`](/sdk/lib/fdio/include/lib/fdio/unsafe.h).
Typically, languages have a tiny library that layers on top of the async library
to perform asynchronous waits on file descriptors. This library typically
provides a less error-prone interface that abstracts these "unsafe" FDIO
functions.

<!-- xrefs -->
[json-ir]: /docs/reference/fidl/language/json-ir.md
[fidl-language]: /docs/reference/fidl/language/language.md
[fidl-wire-format]: /docs/reference/fidl/language/wire-format