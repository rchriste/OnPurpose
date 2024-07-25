# On Purpose: Time Management

Hi my name is Russ Christensen and I have come up with a time management system that I call _On Purpose: Natural Time Management_. I have been diagnosed with Autism and time management experimentation and programming is my current hobby. I have spent the last few years experimenting with various ideas with the goal of writing a program that helps me decide what to do right now. _On Purpose_ is the name that I have given this experimentation. _On Purpose_ is a time management system that is meant to reflect how we naturally operate. All to often I have heard or read great guidance on how to manage my time, but the guidance is very principal based with implementation details left the reader. _On Purpose_ is meant to fill this void.

I envision _On Purpose_ as the personal or individualized view. When it comes to groups or companies there are many existing systems and programs that help plan, implement, and track work. _On Purpose_ on the other hand is for the individual. It aspires to integrate with these existing programs and help that person track and balance all that they need to keep track of. This includes inside and outside of work, meaning the ability to understand your regular routine and what is upcoming. For work it includes the core work that I am getting paid to do and the non-core work that for various reasons are still meetings or work items that are enough of a priority to attend.

Rather than being designed to help the group track your work it is designed for the individual doing the work so it is next step driven. _On Purpose_ encourages you to break work down to next steps until you get to a next step that you can do. When the next step is to wait for someone to get back to you or wait for some program or process to do something then _On Purpose_ is meant to help save or remember what is required to easily resume the work later and it is designed to integrate with existing systems to automate knowing when you can return to something, if this automation does not exist then you set a timer for when to check back.

_On Purpose_ is meant to be a program that integrates with existing systems because that provides the most value and is economical as a hobby, side project. My goal with _On Purpose_ is to integrate with the Microsoft Platform because as a long time Microsoft employee I use Microsoft products. This means that I plan on having On Purpose integrate with the following products:
* Azure Dev Ops
* Outlook Email & Calendaring
* Microsoft Teams
* Microsoft To Do
* OneDrive (information sync's between machines through OneDrive rather than a dedicated service)

Currently _On Purpose_ is a console, text based Windows program written in Rust that uses an embedded SurrealDB database for data storage. I hope to eventually create an Android app for the phone. 

## Principals of _On Purpose: Time Management_:
* Everything we do is done because of a motivational reason, tieing everything we do back to the motivational reason helps us do the right thing in the right way.
* To pick between available tasks the following needs to be taken into account: each tasks urgency, the relative importance of tasks that have the same motivation, the ephemeral priorities of the moment.
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
