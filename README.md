# What is OnPurpose?

Over the past few years I have been on a journey to rethink how a computer can help with the problem of time and task management. This _OnPurpose_ project exists as a place for me to implement my ideas. My initial goal is to implement a CLI and text based menu driven program, then expand to the phone with a Graphical UI, and then make a desktop GUI that can be always visible and pinned opposite the OS taskbar. I expect this to be a multi-year journey, at the very least.

I should also mention that part of my goal is to influence and encourage these ideas to be integrated into other programs and into the OS itself. The reason is because my experience is that it is **not** the core idea but rather how convenient it is to implement that is the real barrier. While this is a practical problem for me I also see this problem from professional therapists, to professionals helping care for the elderly, to institutions researching how to practically improve the lives of neurodiverse adults and children. The problem that I see is that the guidance is not followed because it is not convenient or natural enough to follow with today's software.

The reason why I am personally involved is because of my neurodiversity. I need to do software that is compatible with my brain. I believe this starts by having the list I pick from be a list that puts the next step front and center. The normal view of a list of to do that only shows the smaller next steps after an item is selected is backwards to my brain and feels overwhelming. A list of literal and actionable next steps on the other hand is motivating and encourages hope. It also trains good habit of breaking down an item down to the next step and writing that down so you don't need to keep doing it every time you see that to do on your list.

I do believe that knowing what is upcoming is an essential and critical feature of to do software, but I mean this in two ways. The to do centric view of what is upcoming to accomplish that one to do remains necessary, but my emotional need is to know what **I, myself,** have upcoming. Scheduled activities are part of this, but so is my daily routine, the things I am waiting on, and things I will want to make time to respond to if they come up. I believe that awareness of what is upcoming for the user and the to do item, both, is required for me to make the proper decision, throughout the day, of what to do right then.

Furthermore I believe that to do software can feel rewarding when it is helping one decide what to do in the moment. I believe that this is the positive feedback loop. Making this work well should be at the heart of how to do software is designed and what supporting features it should have. I believe that some of these supporting features must center around the ideas of tree based item prioritization, integration with other software, reducing the hassle of data entry, and the user's daily routine and stated mood.

I believe that the long term value of to do software should measured by if it helped me make better decisions and feel better about what I actually did. Features in support of this goal should be centered around the theme of ensuring the user agrees that they prioritized the right thing after the fact, that there actions were on purpose. To do software should help the user record both effort and accomplishments. I want to do software to help me feel good about what I did and I want it to give me appropriate reminders throughout the day and at the right time so I can improve.

My aspiration with _OnPurpose_ is to redefine expectations of what to do software is. I feel like the core ideas in today's to do software can still be generally implemented in a day planner. I believe the fact that the to do app is so commonly used as a sample program is a signal that to do apps map much more closely to how computers operate than what is intuitive to the human mind. In addition my experience with existing to do software is that it is almost exclusively focused on projects that I am driving and cannot properly represent all the kinds of work that we do, one example is an inability to enumerate supportive or responsive items that we expect to come up.

I believe that the general issue with to do software is that it is either too simple to be truly useful or features are added until the whole thing becomes overly complicated and even still is not exactly doing what is desired. I have seen one of these two mistakes repeated countless times and I believe the solution to this problem rests in the word intuition. And speaking personally I have been stuck at this point for many years, even though I was spending a significant amount of time each week trying to figure out a path forward. My search centered around the idea that software needs to demonstrate understanding. The software needs to "get it."

What I have worked out is that to do items should be tied to the outcome hoped for and backed by the motivation for why we are doing this thing. My hypothesis is that being explicit about this will result in better decisions for how we spend our time. To do software should help with all three phases of preparing, acting, and reflecting on the things we do. The interface should help with the goal of doing the right thing right now from the big picture down to the smallest literal next step. When you are waiting for something to happen it should have awareness or integrations to help you know when to return to something. It should help you remember different to dos related to your current location, head space, or situation. And it should help with the process of stopping and resuming later so you can better remember what your doing. And finally it should help you recall and celebrate your effort independent of what actually came to pass.

This system just can't be done on a piece of paper to do list or planner because it is an active thing that is meant to be reactive and shifting according to the situation. It requires a workable UI that really makes it convenient to implement and follow and in order to do that requires really nailing the fundamental concepts in the most natural way.

## Basic Structure & Concepts

When something to do comes up or when something to do comes into your head you capture these items by pressing _Esc_ from the bullet list menu and selecting _Capture_. _On-Purpose_ helps you prioritize and recall these items, it helps you stay on track by staying true to the reason or reasons for why you are doing this, and it helps you break apart the work into smaller pieces while maintaining a focus on the literal next step or action that you should take.

### Focus On the Next Step with Covering

When you launch _On Purpose_ you are presented with a bullet list of to do items to select from. This is meant to be an actionable list of next step items. In order to make this happen a technique I call covering is used. The way this works is an item is selected from the bullet list and then cover is selected from the menu presented. Now a new item is entered or an existing item is selected that is a smaller next step oriented step towards completing this larger item. This process is repeated until you get down to the literal next step or next action to take. Once that item is finished then the next larger item is uncovered and returns to the bullet list. This process of covering until you get to a next step and then doing the next step is the core inner loop of _On Purpose_. While there is a lot more to it in a nutshell this is how _On Purpose_ works. You subdivide to the next step and then do that thing, rinse, repeat.

#### Covering When Waiting For Something

However sometimes the next step is to wait for something to happen. When this happens you select _waiting for_ in the cover menu. One of the keys behind the waiting for system is integration that allows this system to be as natural and automatic as possible. For example something that I intend to implement is integration with the Microsoft Graph APIs. What this will enable is the ability to send an email or over Teams send a message or make a post and then from the _waiting for_ menu you can select to wait for a response and set a maximum amount of time to wait.

The motivation behind this feature is a to do bullet list that only contains actionable items and to have items put themselves back on the list again once they are actionable. In order to make this work I also intended to make it possible to wait for a computer's CPU or network activity to return to being mostly idle so you can wait on a long running computer task. And when all of those things fail then you can wait on a periodic question that appears in the bullet list to remind you to manually check on something. I should also mention that a to do item can wait for another to do item to complete.

When you cover with a _waiting for_ something then during the covering process you are asked if there is an actionable next step on any of the items up the covering chain that should be added to the bullet list. In order to make this process more convenient and support additional features that have not been discussed yet it is also possible to break apart a bigger item into smaller steps so those can be selected from during this process. It should be mentioned however that the purpose here is to log things so they are not forgotten or as a way to realize what work is required to complete an item the process of exactly specifying the action to take should be saved for the next step process.

### Prioritizing Work Inside Routines

_On Purpose_ is designed to support prioritizing work inside a routine day with systems to break out of your routine in order to respond to a critical need or meet a deadline. A routine is a set of rules that inform which items you are willing to do and how to prioritize them. Routines are contained inside thing called a Life Area. In summary you decide how much time to devote to each Life Area and then inside each Life Area you decide how much time to devote to each routine. Also routines can allow work from other routine to appear as a guest in certain scenarios but still count time to the primary routine.

Supports the mixing of two ways of dealing with priorities. Date driven work with a hard deadline is supported. Also supported is doing work in priority order. Work is prioritized inside of routine, which is a time of day or signal of what your priorities are they are different for each routine that you select. This is nested. For example work that prioritizes any meetings I have accepted and then has various sub routines that I can select from.

#### Routines

Beyond logging literal things to do in order for _On Purpose_ to be complete it must also allow you to log items that you are in support of and tracking items that you want to have awareness of what is going on but there is not currently any literal action items. Also the waiting for items that have already been discussed. This is required so the full day can be captured and planned.

#### Routines & Taking a break & stating willingness and mood

#### Knowing what is upcoming

#### Alternate between focus work, unfocused/awareness work (Also account for what is refreshing work)

### Maintaining Focus, Logging, Keeping your eye on the prize, and feeling satisfaction

### Types of To Dos

- ProactiveActionToTake
- ReactiveBeAvailableToAct
- WaitingFor
- TrackingToBeAwareOf

### Types of Hopes

- MentallyResident
- OnDeck
- Intension
- Released

### Types of Motivations

### In the area work, multitasking and multi-purposing

## Integration with other systems (i.e. The Personal View)

Picking what to do

## Saving, Resuming, and Staying On Track

### Capturing

Take a screenshot, including scroll and simple interaction. Save accessibility data with the copy-paste.

## Reflection & Rewards

## Trying OnPurpose

_OnPurpose_ is currently at the very early stages. I am currently developing it by using it myself and quickly running into issues and then working to add features to address these issues. If you want to try it now you will need to compile it. As of right now On Purpose should work on both Windows and Linux, however Windows is my lead platform (because I work for Microsoft and therefor use Windows).

### Compiling OnPurpose

Compiling On Purpose requires the Rust toolchain and it requires installing various things as well so the Surreal DB dependencies can compile. These other things are things like LLVM and some GNU tools. And this is required because I use Surreal DB as an embedded database (aka "SQLLite") to save data to disk.

- [Install Rust from here](https://rustup.rs)
- [Instructions for how to install the SurrealDB dependencies are here](https://github.com/surrealdb/surrealdb/blob/main/doc/BUILDING.md)

### Using OnPurpose with Windows Terminal

In order for the Emoji and Unicode char to display properly you need to enable the new "Atlas" rendering engine. Go to Settings -> Rendering -> Engine and turn on `Use the new Text Render ("AtlasEngine")`
