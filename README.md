# Why On Purpose?

_On Purpose_ is a an open source program that I, Russ Christensen, wrote to help me track to do items and dynamically schedule my day. It is meant to be the go to program when choosing what to do next. It is meant to be in that habit loop.

I believe _On Purpose_ is innovative and unique and I say this even though there is already an almost endless number of existing to do apps. _On Purpose_ is meant to be a reimagined and rethought to do experience that is tailored to the needs to the neurodiverse and specifically for those who like me have high functioning autism & ADHD.

If you are thinking about using _On Purpose_ for yourself then first that is awesome and second I want to make you aware that this is a hobby project that I work on as I am able. What this means is that making progress takes time. Over the course of years I expect to make significant progress but otherwise things are slow going. I am going to share some of the long term vision for On Purpose and then cover what is currently implemented.

## Long Term Vision

_On Purpose_ should help me find balance between my many different commitments, it should help me remember to return to things, and it should help me recall and reflect on the things I have done. _On Purpose_ should be designed to be low friction. It should be an easy reference to decide what to do now, log what is being done, and it should be convenient to quickly record things to do later. To make this happen _On Purpose_ should be available on the Desktop, the phone, and the watch. Furthermore it should be possible to record things through speaking and pictures in addition to keyboard and touchscreen input. AI should be leveraged to reduce the fatigue of repeated data entry and provide a conversational user experience. _On Purpose_ should integrate with other programs that are a potential source of things to do or something to wait on.

The core purpose of _On Purpose_ is to be that of a scheduler that helps you decide what to do and helps you make decisions on purpose. _On Purpose_ should take into account what has already been done and how much time has been spent on something as a way to schedule what to do next as a way to help find balance. This should not just be tracking time but more about only inputting data when you are logging that you did something and then really using this to make that decision and to figure out how much work something is. It should help you cycle through your various commitments. It also accounts for what is mentally resident along with the normal stuff of needing certain things to be true in order to work on something. It should account for priority along with mental health reasons for being able to focus on something or not. It should help you find balance between your many different commitments and needs. Not a perfect decision but something that you can live with.

On Purpose should help you track your to do items, your relaxation items, and your perishable items. Rather than going to YouTube or a video game console to pick what to do you should be able to use _On Purpose_ which helps you remember and recall all of the various things that you might want to do to take a break and relax but do so in a way that helps you still feel productive. It should also help you remember to use up food items before they perish when you buy things.

-Reflection & Journalling
-What is upcoming, minimize surprises

## What is implemented now

## Previous Write-up

to do application meant to greatly improve my mental health. It is a research project and a hobby. The objective is to let go of a user experience that can largely be implemented in a day planner, a spreadsheet, or simple queries to a database and really rethink or re-imagine the user experience to be closer to how our minds operate and closer to how we in reality actually do work today to pick what to work on and successfully get things done.

I am creating _On Purpose_ because I am neurodiverse and operating in this "natural" and "expected" way is a huge problem for me. When there is a crises going on I can rise to the challenge, and crises from time to time is unavoidable, but the idea that I need a crises to truly perform at my peak is an idea that is bad for my mental health and it does not promote a balanced life. I call this _On Purpose_ because the goal is to do software that helps me make better decisions so I can get things done while still doing the non-hyper focus things of being responsive, aware, and generally maintaining the things I am expected to do while not being overwhelmed from my ability to service and make progress on the things that matter most.

**My goal is to create a prototype that does well on the following criteria:**

1. Does someone have a feeling of relief that the software is helping them track and manage this mess they are in or is the to do list itself a source of stress,
2. If they are excited and hopeful because they know what the next step is and a faith that taking the next step will help, and
3. Does the software help them manage the expectations of others and that they have for themselves.
4. Does this software help me have a balanced life between my varied commitments and needs,
5. How well does it help me make progress and prioritize things inside a single commitment,
6. How well does it help me work efficiently in how it both orders and groups my work and,
7. How convenient and integrated is using this software versus being an extra or duplicated thing on the side.

My goal is to write a program that in my judgement and my experience does well on these criteria. However I am developing this as an open source project because I do have a secondary goal of helping others. When it comes to this project my eventual focus is on having high impact for those who also need it most over high usage or high adoption. To make an analogy I want to say that software and websites that support accessibility for the blind and physically disabled is something that I find so encouraging and it gives me hope about this world. I view accessibility as something that is only high impact for a few but at the same time these same features are also generally useful to many from time to time. In my mind I am trying to do a similar thing but for mental health disabilities or at least for my mental health disability that I have a personal connection with. At the same time, I do believe that my ideas might also be generally useful and it is an eventual goal of mine to advocate for the adoption of my best ideas into commercial software. But my immediate focus is a program that is transformational for me in my life.

**What I am writing is a console/text program that you use from the terminal.** What is in this repo right now is a partially implemented work-in-progress. Lot of menu items are not implemented and much functionality is missing. I believe this along with usability issues would make my program hard to use by others for the time being. I am roughly targeting the summer of 2024 for when I hope to have something that is worth trying out.

I am writing _On Purpose_ in Rust and I do intend to eventually have more than just a desktop console application and I consider a phone application to be a core feature, but also something that comes who knows when and after I reach version 1.0. This is being written with a longer term goal of eventually having a core library that can also be leveraged from a phone app (Android). I am also waiting for UI development in Rust, especially on a phone to get easier before I start working on a phone app or a desktop graphical user interface (GUI).

**I should also mention that as much as possible my program is an extension to the Microsoft platform; this means a program that integrates with Microsoft Email & Calendar, Microsoft OneNote, Microsoft To Do, and Microsoft Teams through the Microsoft Graph API and Azure DevOps. The current operating system I am targeting is Microsoft Windows but I do expect this to work on Linux.**

## What is On Purpose?

_On Purpose_ is meant to help someone cycle through their various to dos efficiently. The vision is to help one return to things before they forget what they are doing, while also mixing in other things that are ongoing and working in new things that come up. _On Purpose_ is a bullet list like to do application for prioritizing your work. As a general rule you are meant to pick the top item from the list and work on that with the rest of the items being upcoming work.

To Do's are organized into 2 priorities or 4 priority levels, Mentally Resident items and On Deck items. A new item is placed at the bottom of the On Deck list, priority 4, and given an amount of time to move from the bottom to the top of the list. Likewise after an item is worked on it is placed at the bottom of the mentally resident list, priority 3, and given a time frame to move through the mentally resident list. These lists are sorted by the percentage of time elapsed. After an On Deck item has more than 100% of time elapsed it jumps in priority to priority 2, above the mentally resident items and likewise mentally resident items that are more than 100% of time elapsed jump to the top, priority 1 list.

In _On Purpose_ after working on an item select the option `I worked on this`. You will then be given the prompt, `How long until you need to work on this again?` and you answer a time value for example `1h` for 1 hour or `5m` for 5 minutes or `1d` for 1 day. The item will then be set at the bottom of the mentally resident (🧠) priority level, which is priority level 3. The item will then move from the bottom to the top of that priority level based on the percentage of time that has passed. For example a one hour item will be near the middle and ordered based on the number at 50% after 30 minutes and then after 55 minutes it will be ordered based on the number 92% and near the top. So this is the first principal that a time frame is given and items are ordered according to the percentage of time that has passed.

### Reverse List, Next Step First

[TODO]

### Reasons to do things or not & Integration

Internal, External, Scheduling

[TODO]

## Getting Started

After _On Purpose_ is installed you launch _On Purpose_ by opening the Windows Program "Terminal" and then at the prompt typing `on_purpose` and pressing enter. On first launch you will see a menu item to Capture a new item.

```Text
PS C:\Users\russ-> on_purpose
Welcome to On-Purpose: Time Management Rethought
This is the console prototype using the inquire package
Version 0.0.90
? Select from the below list
> 🗬   Capture New Item          🗭
[↑↓ to move, enter to select, type to filter]
```

What is being shown is a list with only one item so just press enter.

```Text
>  🗬   Capture                  🗭
? Enter New Item ⍠
```

Type in an item you want _On Purpose_ to track and press _Enter_. _On Purpose_ currently creates a directory at `c:\.on_purpose.db` to save the data. If you are using Linux the save data is kept in your home directory. After you enter your first item _On Purpose_ will exit and whenever _On Purpose_ exits you are expected to type `on_purpose` and press enter to run the _On Purpose_ program again and continue.

Now when you launch _On Purpose_ you are shown a _Dynamic Bullet List_ and as you capture more items that list will grow. To search this list, start typing but **in general _On Purpose_ is designed with the goal that you should mostly just pick the top item and then make a selection from the list for that item. If this item should not be the top item or you cannot take action at this time then you state such in the context list after selecting that item.** However that full list is not initially shown, once you select an item newly captured you are given the following contextual list of choices.

```Text
Welcome to On-Purpose: Time Management Rethought
This is the console prototype using the inquire package
Version 0.0.85
>  ❓ Respond to Robert's text message
?
> Declare Item Type
  I finished
  ⭱ Parent to a new or existing Item
[↑↓ to move, enter to select, type to filter]
```

If this is a quick item that you just did or it should no longer be on the list then select `I finished`. Otherwise you should select `Declare Item Type`.

```Text
> Select from the below list ❓ Respond to Robert's text message
> Select from the below list Declare Item Type
? Select from the below list  
> Action 🪜
  Multi-Step Goal 🪧
  Motivational Reason 🎯
  Help
[↑↓ to move, enter to select, type to filter]
```

The goal here is to state if this is an action to take or something that will need to be broken down further to get to the next step. There is also an option to state if rather than an item to do this is a reason for doing things.

```Text
Welcome to On-Purpose: Time Management Rethought
This is the console prototype using the inquire package
Version 0.0.86
>  [SET STAGING] 🪜 Respond to Robert's text message
?
> On Deck
  Mentally Resident
  Intension
  Released
  Not Set
  Make Item Reactive
[↑↓ to move, enter to select, type to filter]
```

**On Deck** In general items that you should do soon should start with _On Deck_. On Deck signals that this item should be done after the currently mentally resident items or after an on deck deadline is reached. After that deadline is reached then this item will become more important than normal mentally resident items that themselves have not hit a deadline.

**Mentally Resident** This is meant to signal that you are in the middle of doing something that was too big to do all at once and if you don't return to this item shortly you will probably forget something important that will make completing this item more difficult.

**Intension** This signals that this item is something that you intend to do at some point. In part because the prioritization system is not yet implemented the functionality to manage these items is also mostly not implemented although pressing _Esc_ from the bullet list menu and then going into the Hope/Goals menu does ofter some very limited functionality.

**Released** Released items are things that you do not want to be burdened with managing or planning but it is valid for _On Purpose_ to suggest these items and they should be findable in search. Once again this is currently an underdeveloped feature.

**Not Set** This marks the item as not set so it will need to be set in the future.

**Make Item Reactive** This item should not be prompted as a proactive action to take in the dynamic bullet list but it should be findable when search for a parent item.

## Major Features Currently Missing

**Routines along with the Prioritization and Scheduling of Work** is currently missing. This is about being able to define a different set of priorities during different parts of the day, which I call a routine. Then it is about prioritizing work against other related work. And finally it is about the mostly rough scheduling of this work to do.

**Along with work** is something that we desire to do along with something else even though it is not required and maybe not even related to the primary item. Doing something in the background during a meeting is an example of along with work. Another example is if you are programming and you make a not strictly or absolutely required change because it is a good idea and you are already editing that part of the code then this is also "along with work." The idea is that this is work that is done along with something else because that just makes and is more efficient.

**Mood** is not really a perfect name but that is what I am calling it for now. This is about being able to define work based on certain qualities that account for our mental or physical state. For example:

* Being able to say that I need to work on something right now that involves new information like being made aware of things versus being able to work or focus on something very expected and without surprises. Or,
* Work that requires my full undivided attention versus things I can do with partial attention. Or,
* Things that I can do even though I don't feel too well right now versus work that needs to wait until I feel physically fine.
* Things that involve sitting still versus things when I need to be moving around.

**Integration with the Microsoft Platform** is about making _On Purpose_ more reactive, smart, and useful to what is happening rather than needing to be told everything. For example:

* When you flag and email it will automatically show up on the list. Or,
* When you send someone or a group an email message or message over Microsoft Teams you can mark that you are waiting for a response. Or,
* You can start a Windows Focus Session. Or,
* When you create an Azure DevOps PR, Build or Deployment you can track this and be told when it return because there is something that you can do.
* Watch your email or Azure DevOps PRs, and Work Items for new things coming your way that you should look at.
* The ability to see your upcoming meetings in your Dynamic Bullet list, state if you are going to attend and be able to better schedule work around it.

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
