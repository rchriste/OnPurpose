# On Purpose: Time Management

Hi my name is Russ Christensen and I have come up with a time management system that I call _On Purpose: Time Management_. I have been diagnosed with Autism and I consider time management as one of my biggest problems. I have spent the last few years rethinking the problem space and experimenting with what information is required to make a decision of what to do now. _On Purpose_ is a time management system. In order to implement this system I have written a console application in Rust. _On Purpose_ is meant to be a program that is used inside your habit loop of deciding what to do. It is meant to be used consistently throughout the day to get guidance on what to do next. Because I work for Microsoft and personally use Microsoft products _On Purpose_ is meant to integrate with the Microsoft platform. Meaning Microsoft Teams, Azure Dev Ops, OneNote, Outlook, Microsoft To Do, and OneDrive. For privacy and security reasons there is no web service rather it is a standalone console application that will eventually sync information between machines through OneDrive.

## Principals of _On Purpose: Time Management_:
* Everything we do is done because of a motivational reason
* To pick between what can be done now: urgency, importance, your mood, and your routine should be taken into account
* Inside the same motivational reason items can be prioritized against each other and stack ranked
* Prioritizing between motivational reasons should consider your mood and your routine, but it is otherwise an ephemeral (in the moment) decision
* Urgency can be classified into various levels, but prioritization inside the same level is an ephemeral decision
* Items should be reviewed periodically: the plan, the importance, and the urgency

## Existing solutions
The problem that I had with existing self-help time management solutions is almost no one seems to stick with these systems over the course of many years. Especially when someone faces a crises these other system are more commonly abandoned rather than leaned into and relied on. Another signal that the ideal solution doesn't yet exist is the fact that these existing time management self-help systems might be more convenient to carry out with software, but software is not strictly required. Imagine using a modern word processor for a couple years and then returning to a typewriter; and yet people return to a simple bullet list all the time after trying out the time management software of today.

## Interested?
If so I want to warn you that _On Purpose_ is currently an unpaid hobby project for me and while this remains true I am the primary customer. At this stage the application is rough with bugs and usability problems. It is more as a proof of concept and research project than it is a serious commercial or open source project. 

I am currently working towards implementing my vision to something that I consider showable. If you want to build it and try it out then be my guest but otherwise I recommend waiting.

## Installing On Purpose

If you want to try it now you will need to compile it and use the Rust tool `Cargo install` to install it. As of today I expect _On Purpose_ to work on both Windows and Linux but I expect the Windows side to eventually be more fully featured as I do have plans to integrate with various Windows API in time. I will mention that setting up the Surreal DB build dependency is more of a pain in Windows proper than the convenient steps you can follow inside Windows' Linux WSL layer. But Windows is what I am currently using.

### Compiling On Purpose

Compiling On Purpose requires the Rust toolchain and it requires installing various things as well so the Surreal DB dependencies can compile. These other things are things like LLVM and some GNU tools. This is required because I use [Surreal DB](https://github.com/surrealdb/surrealdb) as an embedded database that persists data to disk.

* [Install Rust from here](https://rustup.rs)
* [Instructions for how to install the SurrealDB dependencies are here](https://github.com/surrealdb/surrealdb/blob/main/doc/BUILDING.md)

If you want to be able to just type `on_purpose` from a console window then you can install _On Purpose_ by doing `cargo install --path console` then as further changes are checked in you can do a `git pull` and rerun the cargo install command to update to the latest version.

Because it takes a while to build I will generally use the older version of _On Purpose_ while the new one compiles and then after I get an error that the file is in use I will close the _On Purpose_ program and rerun the cargo install command a second time to install the updated binary.

### Using On Purpose with Windows Terminal

In order for the Emoji and Unicode char to display properly you need to enable the new "Atlas" rendering engine. Go to Settings -> Rendering -> Engine and turn on `Use the new Text Render ("AtlasEngine")`
