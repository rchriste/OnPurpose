# What is On Purpose?

Over the past few years I have been on a journey to rethink and re-imagine how a computer can better help with the problem of time and task management. I am neurodiverse and I have a belief that better software can greatly improve my mental health. **I overthink things, I am very out of sight, out of mind, and when something comes up I struggle to properly return to things.** These mental health issues make it difficult for me to consistently reason out the right thing to do right now, in the moment.

I believe software that helps one manage To Do's should be able to help with these problems but today's software seems to be concerned with helping the user create lists of things To Do and prioritize them but without consideration for the mental health challenges and the related practical problems that are also fundamental and need to be solved in order to reliably get things done without having a crises going on.

The almost universal approach of having To Dos organized inside a collection of lists is confusing to over thinkers like me because it is just a lot to consider, doing this frequently is overwhelming because it is just so mentally exhausting. But the mental health problems don't stop there. The issue is that some can feel very pressured each time they are reminded of the many things they have not done. When this is shown frequently to pick the next thing to do this negativity can be completely and utterly overwhelming. I expect that when all we had was a pen and paper or day planner then this was just a reality of how the world had to be. But if you use software on a computer or cell phone to track your work I believe there is probably an alternative way of presenting this information that is better for our mental health.

**Success for improving mental health should be measured by:**

1. If someone has a feeling of relief that the software is helping them track and manage this mess they are in,
2. If they are excited and hopeful because they know what the next step is and,
3. Do they have a hopeful belief that taking the next step will actually help.

**Regarding practical challenges success should be measured by:**

1. Does this software help me have a balanced life between my varied commitments and needs,
2. How well does it help me prioritize things inside a single commitment,
3. How well does it help me order and group my work so I am working efficiently and,
4. How convenient and integrated is using this software versus being an extra or duplicated thing on the side.

**I expect software that does well on these metrics to be a life changing experience to use, something that once you experience you wonder how you ever lived without.** But my observation is that people are not really seeing the problem in this way and they are not yet seeing this as a solvable problem despite multiple amazing innovations that have happened in software these past many years.

**A few years ago I had one of those, "Well I guess I'll just have to do it myself then!" moments.** My goal was to figure out a user interface (UI) and a user experience (UX) for a To Do like program that actually addresses these mental health issues. I have incorporated my interpretation of ideas discussed by David Allen and Steven Covey in their seminole books. I have been inspired by the successes (and failures) of my neurodiverse friends. I have paid attention to the implementation challenges of following a routine given by child therapists, adult therapists, professional retirement home staff, and some research done by the academic community that is dedicated to helping the neurodiverse. One of my realizations during this journey is that I should define my own opinionated time management system and work out a UI/UX that implements this system.

This _On Purpose_ project was created as a place for me to implement, refine, and try these ideas. My primary motivation is to have a program that I myself can use and greatly benefit from. I am developing this in the open and as open source so others can also benefit. Having said that **what is in this repo right now is a partially implemented work-in-progress that does not yet properly demonstrate my core vision.**

What I am writing is a console/text program that you use from the terminal. This is being written with a longer term goal of eventually having a core library that can also be leveraged from a phone app (Android). I should also mention that as much as possible my program is an extension to the Microsoft platform; this means a program that integrates with Microsoft Email & Calendar, Microsoft OneNote, Microsoft To Do, and Microsoft Teams through the Microsoft Graph API and Azure DevOps. The current operating system I am targeting is Microsoft Windows but I do expect this to work on Linux.

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

Type in an item you want _On Purpose_ to track and press _Enter_. _On Purpose_ currently creates a directory at `c:\.on_purpose.db` to save the data. After you enter your first item _On Purpose_ will exit and whenever _On Purpose_ exits you are expected to type `on_purpose` and press enter to run the _On Purpose_ program again and continue.

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

If you want to try it now you will need to compile it and use the Rust tool `Cargo` to install it. As of today I expect _On Purpose_ to work on both Windows and Linux however I do have plans to integrate with various Windows API in time. I will mention that setting up the Surreal DB build dependency is more of a pain in Windows proper than the convenient steps you can follow inside Windows' Linux WSL layer. But Windows is the target platform.

### Compiling On Purpose

Compiling On Purpose requires the Rust toolchain and it requires installing various things as well so the Surreal DB dependencies can compile. These other things are things like LLVM and some GNU tools. This is required because I use [Surreal DB](https://github.com/surrealdb/surrealdb) as an embedded database that persists data to disk.

* [Install Rust from here](https://rustup.rs)
* [Instructions for how to install the SurrealDB dependencies are here](https://github.com/surrealdb/surrealdb/blob/main/doc/BUILDING.md)

If you want to be able to just type `on_purpose` from a console window then you can install _On Purpose_ by doing `cargo install --path console` then as further changes are checked in you can do a `git pull` and rerun the cargo install command to update to the latest version.

Because it takes a while to build I will generally use the older version of _On Purpose_ while the new one compiles and then after I get an error that the file is in use I will close the _On Purpose_ program and rerun the cargo install command a second time to install the updated binary.

### Using On Purpose with Windows Terminal

In order for the Emoji and Unicode char to display properly you need to enable the new "Atlas" rendering engine. Go to Settings -> Rendering -> Engine and turn on `Use the new Text Render ("AtlasEngine")`
