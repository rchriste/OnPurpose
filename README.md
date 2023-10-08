# What is OnPurpose?

Over the past few years I have been on a journey to rethink how a computer can help with the problem of time and task management. This OnPurpose project exists as a place for me to implement my ideas. My initial goal is to implement a CLI and text based menu driven program, then expand to the phone with a Graphical UI, and then make a desktop GUI that can be always visible and pinned opposite the OS taskbar. I expect this to be a multi-year journey, at the very least.

I should also mention that part of my goal is to influence and encourage these ideas to be integrated into other programs and into the OS itself. The reason is because my experience is that it is **not** the core idea but rather how convenient it is to implement that is the real barrier. While this is a practical problem for me I also see this problem from professional therapists, to professionals helping care for the elderly, to institutions researching how to practically improve the lives of neurodiverse adults and children. The problem that I see is that the guidance is not followed because it is not convenient or natural enough to follow with today's software.

My aspiration with *OnPurpose* is to redefine expectations of what to do software is. I feel like the core ideas in today's to do software can still be generally implemented in a day planner. I believe the fact that the to do app is so commonly used as a sample program is a signal that to do apps map much more closely to how computers operate than what is intuitive to the human mind. In addition my experience with existing to do software is that it cannot properly represent all the kinds of work that we do, one example is an inability to enumerate supportive or responsive items that we expect to come up.

I believe that the general issue with to do software is that it is either too simple to be truly useful or features are added until the whole thing becomes overly complicated and even still is not exactly doing what is desired. I have seen one of these two mistakes repeated countless times and I believe the solution to this problem rests in the word intuition. And speaking personally I have been stuck at this point for many years, even though I was spending a significant amount of time each week trying to figure out a path forward. My search centered around the idea that software needs to demonstrate understanding. The software needs to "get it."

What I have worked out is that to do items should be tied to the outcome hoped for and backed by the motivation for why we are doing this thing. My hypothesis is that being explicit about this will result in better decisions for how we spend our time. To do software should help with all three phases of preparing, acting, and reflecting on the things we do. The interface should help with the goal of doing the right thing right now from the big picture down to the smallest literal next step. When you are waiting for something to happen it should have awareness or integrations to help you know when to return to something. It should help you remember different to dos related to your current location, head space, or situation. And it should help with the process of stopping and resuming later so you can better remember what your doing. And finally it should help you recall and celebrate your effort independent of what actually came to pass.

## Why hasn't this already been done and tried? (MAYBE THIS REPEATS AND SHOULD BE REMOVED OR MOVED TO THE PRESENTATION ONLY)

As weird as this sounds I am just not really aware of to do software that tries to be a scheduler in a wholistic way. I think part of the reason is the idea that if a computer is a scheduler that it is then taking away from the agency of the person rather than a presenter of choices but the user remains in control. Regardless my experience is that to do software goes help you prioritize things but the idea is that you do things off the to do list but the goal is always to also do other things outside of that.

This system just can't be done on a piece of paper system because it is an active to do system and is meant to be reactive and shifting according to the situation. Which brings up the second issue that it requires a workable UI that really makes it convenient to implement and follow and in order to do that requires really nailing the fundamental concepts in the most natural way.

## Drive to Next Steps, (i.e. Covering, To Dos, Hopes, and Motivations)

Have a list of things to do and cover that to the next step. To do's tie back to hopes and are done for motivations.
There are next steps and do not forget items.

### Alternate between focus work, unfocused/awareness work (Also account for what is refreshing work)

### Other uses of Covering

Can cover item by another item and can cover an item by a watcher that watches. With a rethink date if the covered item doesn't complete by then.

### Picking what to do, acting with awareness, and in context

Pay attention to what is mentally resident
See Upcoming
Pick mood and state willingness

### Types of To Dos

* ProactiveActionToTake
* ReactiveBeAvailableToAct
* WaitingFor
* TrackingToBeAwareOf

### Types of Hopes

* MentallyResident
* OnDeck
* Intension
* Released

### Types of Motivations

### In the area work, multitasking and multi-purposing

## Integration with other systems (i.e. The Personal View)

Picking what to do

## Saving, Resuming, and Staying On Track

### Capturing

Take a screenshot, including scroll and simple interaction. Save accessibility data with the copy-paste.  

## Reflection & Rewards

## Trying OnPurpose

OnPurpose is currently at the very early stages. I am currently developing it by using it myself and quickly running into issues and then working to add features to address these issues. If you want to try it now you will need to compile it. As of right now On Purpose should work on both Windows and Linux, however Windows is my lead platform (because I work for Microsoft and therefor use Windows).

### Compiling OnPurpose

Compiling On Purpose requires the Rust toolchain and it requires installing various things as well so the Surreal DB dependencies can compile. These other things are things like LLVM and some GNU tools. And this is required because I use Surreal DB as an embedded database (aka "SQLLite") to save data to disk.

* [Install Rust from here](https://rustup.rs)
* [Instructions for how to install the SurrealDB dependencies are here](https://github.com/surrealdb/surrealdb/blob/main/doc/BUILDING.md)

### Using OnPurpose with Windows Terminal

In order for the Emoji and Unicode char to display properly you need to enable the new "Atlas" rendering engine. Go to Settings -> Rendering -> Engine and turn on `Use the new Text Render ("AtlasEngine")`
