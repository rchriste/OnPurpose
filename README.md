# On Purpose: Intuitive Time Management
## Why does this project exist?

Hi my name is Russ Christensen and I have been diagnosed with Autism, which is a mental health disability. Before I relate this to time management I want to discuss the disability of being blind. This disability is obviously impactful but when software is designed to work well for the blind then it changes from being a barrier to an indispensable help, and depending on how well the software is designed for someone who is blind, as there are obviously many levels in between. I bring this up because by making changes across the software ecosystem we can reduce the impact of the blindness disability and I believe the same is true for the mental health disability. When computers help me remember what to do and what I did and when they help me focus on getting that done and doing it without randomization, unless that randomization is maybe a higher priority than me staying on task, when computers do that they can change from being a barrier to an indispensable help. **Now my understanding is that in a clinical setting, those who are neurodiverse discuss the challenges of time management quite a bit so I believe there is room for improvement, to put it lightly.**

This project exists because I want to be part of the solution. I am a software programmer and a few years ago I was looking for a project that could help me learn the Rust programming language and I settled on the idea of writing my own time management software as a hobby. I challenged myself to do more than just rewrite my own version of an app that already exists. What if to do apps didn't exist and we took someone, like me, who has this mental health disability and challenged him to leverage the best ideas of others and invent a user experience and a software solution to this problem.

_On Purpose: Intuitive Time Management_ is the name of the program that I eventually came up with. This is a research project. My goal with this project is to experiment with, adjust, and prove out these ideas with the goal of reducing the risk of integrating these ideas into existing software. The risk exists because I am suggesting impactful changes to the software ecosystem. My initial goal was to write a piece of software that changed my life because it reduces the practical impact of my disability and I am proud to say that I have been benefiting from _On Purpose_ constantly and every day since January of 2024. Improving the program for myself remains my top priority but I am also working towards the goal of being able to support others because if these ideas are ever going to be integrated into software more widely then we need to have a group of users who use this program and greatly benefit from it. I am on a multi-year journey.

## What we should expect a To Do app to do:

* Help efficiently pick which item to do by taking into account: urgency, relative importance, and in-the-moment priorities.
* Help be aware of and prepared for what is upcoming.
* Help transition from one item to the next & save and resume work.
* Help the user avoid distractions, stay on task, & remember the purpose behind the work.
* Help recall what was done and feel rewarded for effort.

## Existing solutions

The problem that I had with existing self-help time management solutions is almost no one seems to stick with these systems over the course of many years. Especially when someone faces a crises these other system are more commonly abandoned rather than leaned into and relied on. Another signal that the ideal solution doesn't yet exist is the fact that these existing time management self-help systems might be more convenient to carry out with software, but software is not strictly required. Imagine using a modern word processor for a couple years and then returning to a typewriter; and yet people return to a simple bullet list all the time after trying out the time management software of today.

## What is _On Purpose_?

All to often I have heard or read wonderful guidance on how to manage my time, but the guidance is very principal based with implementation details left to the reader. _On Purpose_ is meant to fill this void.

I envision _On Purpose_ as the personal or individualized view. When it comes to groups or companies there are many existing systems and programs that help plan, implement, and track work. _On Purpose_ on the other hand is for the individual. It aspires to integrate with these existing programs and help that person track and balance all that they need to keep track of. This includes inside and outside of work, meaning the ability to understand your regular routine and what is upcoming. For work it includes the core work that I am getting paid to do and the non-core work that for various reasons are still meetings or work items that are enough of a priority to do or attend.

Rather than being designed to help the group track your work it is designed for the individual doing the work so it is next step driven. _On Purpose_ encourages you to break work down to next steps until you get to a next step that you can do. When the next step is to wait for someone to get back to you or wait for some program or process to do something then _On Purpose_ is meant to help save or remember what is required to easily resume the work later and it is designed to integrate with existing systems to automate knowing when you can return to something. _On Purpose_ is intended to do a lot of this automatically but if it can't then you set a timer for when to check back.

_On Purpose_ is meant to be a program that integrates with existing systems rather than replaces them. My goal with _On Purpose_ is to integrate with the Microsoft Platform because I am a long time Microsoft employee and I use Microsoft products in my day job and at home. This means that I plan on having _On Purpose_ integrate with the following products:
* Outlook Email, Calendaring, & To Do
* Microsoft Teams
* Microsoft OneNote
* Azure Dev Ops
* OneDrive

## Interested?

If so I want to warn you that _On Purpose_ is currently an unpaid hobby project for me and while this remains true I am the primary customer. At this stage the application is rough with bugs and usability problems. It is more as a proof of concept and research project than it is a serious commercial or open source project. 

I am currently working towards implementing my vision to something that I consider showable. If you want to build it and try it out then be my guest but otherwise I recommend waiting until I am far enough long to feel good about providing binaries to download or install.

## Core Rust Crates of _On Purpose_

Currently _On Purpose_ is a text based Windows program written in Rust. It brings up a selection of items using the [inquire](https://github.com/mikaelmello/inquire) crate. This is the current UI because it is the easiest to experiment with as I work out the core feature set. In time I intend to adopt [Ratui](https://ratatui.rs/) for a more fully featured but still text based UI. 

I am also paying attention to GUI app development in Rust. I am doing this for two reasons, on the desktop I would like to eventually be an always viewable docked application similar to the windows start bar. I also hope to eventually create an Android app for the phone and investigate the idea of integrating in some fashion with the Android operating system. I am paying attention to the following projects and I intend to eventually try to prototype _On Purpose_ in each of them: [Makepad](https://github.com/makepad/makepad), [Dioxus](https://dioxuslabs.com/), and [Xilem](https://github.com/linebender/xilem). Also to better share code between platforms I am paying attention to [Robius](https://robius.rs/).

The data storage layer is implemented as an embedded [SurrealDB](https://github.com/surrealdb/surrealdb) database. You can think of this like [SQLLite](https://www.sqlite.org/index.html) except I am using [SurrealDB](https://github.com/surrealdb/surrealdb). Currently I only save data locally. Sync'ing the data between machines is planned, however I want to avoid having a service for multiple reasons. Ideally I would sync the data between machines using the [Microsoft Graph To Do REST APIs](https://lib.rs/crates/graph-rs-sdk), but I doubt I can make my to do items compatible with the Microsoft To Do schema. I plan on trying to extend the To Do API with a json blob in the To Do Notes section, but I'm not sure how much I should be doing this. Also there are things to sync beyond to do items, like time spent logs so beyond the To Do REST API I plan on also syncing data between machines by placing files in [OneDrive](https://lib.rs/crates/onedrive).

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
