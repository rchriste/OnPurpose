# What is On Purpose?

Over the past few years I have been on a journey to rethink and re-imagine how a computer can better help with the problem of time and task management. I am neurodiverse and I have a belief that better software can greatly improve my mental health. **I overthink things, I am very out of sight, out of mind, and when something comes up I struggle to properly return to things.** These mental health issues make it difficult for me to consistently reason out the right thing to do right now, in the moment.

I believe software that helps one manage To Do's should be able to help with these problems but today's software seems to be concerned with helping the user create lists of things To Do and prioritize them but without consideration for the mental health challenges and the related practical problems that are also fundamental and need to be solved in order to reliably get things done without having a crises going on.

The almost universal approach of having To Dos organized inside a collection of lists is confusing to over thinkers like me because it is just a lot to consider, doing this frequently is overwhelming because it is just so mentally exhausting. But the mental health problems don't stop there. The issue is that some can feel very pressured each time they are reminded of the many things they have not done. When this is shown frequently to pick the next thing to do this negativity can be completely and utterly overwhelming. I expect that when all we had was a pen and paper or day planner then this was just a reality of how the world had to be. But if you use software on a computer or cell phone to track your work I believe there is probably an alternative way of presenting this information that is better for our mental health.

**Success for improving mental health should be measured by:**

1. If someone has a feeling of relief that the software is helping them track this mess they are in,
2. If they are excited and hopeful because they know what the next step is and,
3. Do they have a hopeful belief that taking the next step will actually help.

**Regarding practical challenges success should be measured by:**

1. Does this software help me have a balanced life between my varied commitments and needs,
2. How well does it help me prioritize things inside a single commitment,
3. How well does it help me order and group my work so I am working efficiently and,
4. How convenient and integrated is using this software versus being an extra or duplicated thing on the side.

**I expect software that does well on these metrics to be a life changing experience to use, something that once you experience you wonder how you ever lived without.** But my observation is that people are not really seeing the problem in this way and they are not yet seeing this as a solvable problem despite multiple amazing innovations that have happened in software these past many years.

**A few years ago I had one of those, "Well I guess I'll just have to do it myself then!" moments.** My goal was to figure out a user interface (UI) and a user experience (UX) for a To Do like program that actually addresses these mental health issues. I have incorporated my interpretation of ideas discussed by David Allen and Steven Covey in their seminole books. I have been inspired by the successes (and failures) of my neurodiverse friends. I have paid attention to the implementation challenges of following a routine given by child therapists, adult therapists, professional retirement home staff, and some research done by the academic community that is dedicated to helping the neurodiverse. One of my realizations during this journey is that I should define my own opinionated time management system and work out a UI/UX that implements this system.

This _On Purpose_ project was created as a place for me to implement, refine, and try these ideas. If you use the program currently checked in, it does kinda work but I recommend you wait for the time being. I mean I am using this program now, but as of today what I have written is a partially implemented work-in-progress that does not yet demonstrate the core of my vision. **I am targeting the summer of 2024 as an upcoming milestone for when I hope to have a usable program that shows my vision.** This is a console/text program that you use from the terminal. I should also mention that as much as possible my program is an extension to the Microsoft platform; this means a program that runs on Microsoft Windows and integrates with Microsoft Email & Calendar, Microsoft OneNote, Microsoft To Do, and Microsoft Teams through the Microsoft Graph API and Azure DevOps.

## Getting Started

After _On Purpose_ is installed you launch _On Purpose_ by opening the Windows Program "Terminal" and then at the prompt typing `on_purpose` and pressing enter. On first launch you will see a menu item to Capture a new item.

```text
PS C:\Users\russ-> on_purpose
Welcome to On-Purpose: Time Management Rethought
This is the console prototype using the inquire package
Version 0.0.78
?
> 🗬    Capture                    🗭
  ↝ ↝  Change Routine             ↜
        Reflection
  👁 🗒️ View Bullet List (To Dos) 👁
  🗬 🙏 Capture Hope              🗭
  👁 🙏 View Hopes                👁
v 🗬 🎯 Capture Motivation        🗭
[↑↓ to move, enter to select, type to filter]
```

Press Enter

```Text
>  🗬   Capture                  🗭
? Enter New Item ⍠
```

Type in an item you want _On Purpose_ to track and press _Enter_. _On Purpose_ currently creates a directory at `c:\.on_purpose.db` to save the data. After you enter your first item _On Purpose_ will exit and whenever _On Purpose_ exits you are expected to type `on_purpose` and press enter to run the _On Purpose_ program again and continue.

Now when you launch _On Purpose_ you are shown a _Dynamic Bullet List_ and as you capture more items that list will grow. To capture further items press _Esc_ from this menu. To search this list, just start typing but **in general _On Purpose_ is designed with the goal that you should mostly just pick the top item and then make a selection from the list for that item. If this item should not be the top item or you cannot take action at this time then you state such in the context list.** However that full list is not initially shown, once you select an item newly captured you are given the following contextual list of choices.

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
>  ❓ Respond to Robert's text message
>  Declare Item Type
?
> New Action 🪜
  New Supportive or Tracking
  New Proactive Multi-Step Goal 🪧
  New Responsive Multi-Step Goal 🪧
  New Proactive Motivational Reason 🎯
  New Responsive Motivational Reason 🎯
[↑↓ to move, enter to select, type to filter]
```

**New Action** Means the item captured is an action you should take. _Emoji is a ladder 🪜 with next steps._

**New Supportive or Tracking** Means this is something expected to take some of your time, but in response to something that will probably or maybe happen. As of today this is an underdeveloped feature. Integration with the Microsoft Graph APIs & Azure DevOps is not hooked up. Also detection for if you are maybe over committed is not implemented. If you select this option now it will just hide the Item from showing up in the dynamic bullet list.

**New Proactive Multi-Step Goal** Is for a milestone or hopeful outcome that will need to be broken down to smaller next steps. _Emoji is a Milestone sign 🪧 or goal post._

**New Responsive Multi-Step Goal** The word responsive means do **not** prompt for a next step but this goal should be findable when parenting an action to a goal.

**New Proactive Motivational Reason** This is for stating that the item captured is a reason for doing something. Because there is almost always a diverse number of benefits to doing something the word motivational is also used. The test to know if a reason is motivational is to ask the question if this was not true would that significantly change the priority or cancel the work. _Emoji is a target 🎯 that provides something to aim for._

**New Responsive Motivational Reason** Once again the word responsive means that when something comes up this is a motivation for acting or responding but _On Purpose_ should **not** prompt to define goals or actions.

After you declare an _item type_ and before you are given the full contextual list you also need to set what I am currently calling the "Staging" for that item. This is required because the information is used to order the list. The following is what that menu looks like.

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

## Installing On Purpose

If you want to try it now you will need to compile it and use the Rust tool `Cargo` to install it. As of today I expect _On Purpose_ to work on both Windows and Linux however I do have plans to integrate with various Windows API in time. I will mention that setting up the Surreal DB build dependency is more of a pain in Windows proper than the convenient steps you can follow inside Windows' Linux WSL layer. But Windows is the target platform.

### Compiling On Purpose

Compiling On Purpose requires the Rust toolchain and it requires installing various things as well so the Surreal DB dependencies can compile. These other things are things like LLVM and some GNU tools. This is required because I use [Surreal DB](https://github.com/surrealdb/surrealdb) as an embedded database that persists data to disk.

- [Install Rust from here](https://rustup.rs)
- [Instructions for how to install the SurrealDB dependencies are here](https://github.com/surrealdb/surrealdb/blob/main/doc/BUILDING.md)

If you want to be able to just type `on_purpose` from a console window then you can install _On Purpose_ by doing `cargo install --path console` then as further changes are checked in you can do a `git pull` and rerun the cargo install command to update to the latest version.

Because it takes a while to build I will generally use the older version of _On Purpose_ while the new one compiles and then after I get an error that the file is in use I will close the _On Purpose_ program and rerun the cargo install command a second time to install the updated binary.

### Using On Purpose with Windows Terminal

In order for the Emoji and Unicode char to display properly you need to enable the new "Atlas" rendering engine. Go to Settings -> Rendering -> Engine and turn on `Use the new Text Render ("AtlasEngine")`
