# TinyGuard : A customisable compile-time checker for libraries

Are you writing some code that will be used by others?
Well, probably, it's not often that you write code just for yourself.

However, due to language designs and limitations, you can't always enforce the correct usage of your code/API.
Well, you could with a lot of runtime checks, assertions and what not, but that adds boilerplate that you have to write and maintain.

TinyGuard aims to provide simple compile-time checks for the users of your code, checks designed by you, the library author.

## Current state of the project

Does nothing concrete for now, just setting up the project.

## Why should I use TinyGuard?

Compile-time checks are a nice way to avoid debugging cryptic error messages over tiny mistakes that can be easily overlooked, so you can integrate your own that are specific to your library, that the compiler or other tools can't provide.

All developers code and think about the design of their libraries in different ways. TinyGuard allows you to help other developers use your code in the way you intended, by providing additional compile-time checks that you define, like a tiny guard on the shoulder of the user that tells them what's wrong.

It also protects you from yourself, as you can also forget how to use your own code.

## How to use it

TODO

## Where it's bad

TinyGuard is indeed tiny, it's not and will never be an all powerful static analysis tool. It won't ever be as developped as Rust's borrow checker or LLVM's Clang Static Analyzer.
It will most likely struggle with complex flow control like loops and recursion, and not be able to provide the same level of guarantees as those tools.

## Where it's good

Being tiny also means that it's much easier to integrate into your codebase, and it's much easier to write new checks for it.
For small projects and for novice developers, using a custom written clang plugin or a full blown static analysis tool can be overkill and intimidating (I know it was for me), so TinyGuard can be a good starting point.

Also, since it's very simple and broad, it is not limited to a specific language or compiler. As long as you can generate an AST, you can use some features of TinyGuard.

It's also possible to use it to look out for bad data formats.

## How does it work?

TODO

## Is this actually useful?

First, I did learn a lot about parsing and ASTs, so that's a plus.

Second, I'm going to actually answer that with what made me want to create TinyGuard in the first place.
I have a library that I'm working on, about analysing stream-graphs (graphs that evolve over time), that is written in C.

For some specific metrics that could be computed, I needed to query all the nodes that were present in the graph at a certain time. However, making this query efficient required pre-computing some data structures.
However, since the graphs analysed could be huge, pre-computing that data structure could take a lot of time and memory, which would end up being wasted if the user didn't actually need that specific metric.
So, I decided to make that computation on a separate function, that the user would need to call first before computing the metric.
However, the user could forget to call that function, and this could lead to crashes or incorrect results.
We all know how explicit C errors can be, right?

I could've added a runtime check that prevents that, but not only would you have loaded that huge graph for nothing, but the best error message you would get would be that you forgot to call that function somewhere.
I could've added a lazy evaluation mechanism, but I wanted the user to be aware of the cost of computing that metric, because performance is a concern when analysing bigger graphs. Also since it can support multi-threading, that would mean having to add locks and what not, which would add even more overhead.
Also, since computing that structure is heavy, I provided a way to save it to a file, so that the user could load it later, and avoid recomputing it. It would be a bit weird to the saving explicit, but the loading implicit.

With TinyGuard, I can just verify that the user called that function before computing the metric, and if they didn't, I can give them a nice error message that tells them exactly what and where is the mistake, with additional hints on how to fix it.
This avoids wasting the user's time with debugging segmentation faults and needing to find the documentation to understand what went wrong.

The typical program that uses my library would be quite small and wouldn't need complex flow control, so simple compile-time checks would be enough to avoid most of the common mistakes.