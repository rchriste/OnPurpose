# Scheduling

_On Purpose_ is meant to help you decide what to do right then. This means that _On Purpose_ is intended to be what you go to in order to decide what to do and it means that the goal of the bullet list view is for the top item to be something that the user agrees should be worked on next. These guiding principals have a profound impact on the design of _On Purpose_. Most especially the goal is to gather and track information that is required to solve this problem.

How an item is classified into stages plays a large roll in deciding what to do next.

```Text
   To Do Item Stages
═══════════════════════
┌─────────────────────┐     Requires information that is currently in your brain 
│🧠 Mentally Resident │     that will fade over time. It is important to return 
└─────────────────────┘     to these items to prevent that from happening.
           ⇧
     ┌───────────┐          Something you hope to get to as soon as 
     │🪜 On Deck │          the mentally resident workload allows it 
     └───────────┘          or after an amount of time has passed.
           ⇧
     ┌───────────┐          Items that should be reviewed to be put on 
     │📌 Planned │          On Deck once related or earlier items that 
     └───────────┘          are part of the same effort are concluded.
           ⇧
 ┌───────────────────┐      For items that you hope to get around 
 │ 🤔 Thinking about │      to or that are not yet well formed 
 └───────────────────┘      enough to take on the work.


      ╒══════════╕          Things that you would love to do but they should not be directly 
      │ Released │          tracked as a to do item, rather they are suggested as something 
      ╘══════════╛          that could also be done when doing other work. 
```

## Mentally Resident & On Deck

When suggesting what to do _On Purpose_ prefers mentally resident items. The goal is to finish the mentally resident items and then select the next on deck item and work on that. However on deck items can be suggested regardless after a certain amount of time has passed. This is to ensure that everything that is either the mentally resident or on deck will eventually make it to the top of the list and become the suggested item. In order to work this out the user is asked to enter what is called a lap time. This is entered when an item is first set as mentally resident or on deck and again whenever the item is worked on. Then the number of laps is calculated based on how much time has elapsed.

```Text
                How items on the Bullet List are prioritized
═════════════════════════════════════════════════════════════════════════════════════
│     ⏰     │ 🧠 Mentally Resident │ Items with a laps number above one are shown 
│ Laps > 𝟏.𝟎 │            &          │ first. Mentally resident items have the laps 
│     ⏰     │       🪜 On Deck     │ number squared so they are higher on the list.
─────────────────────────────────────────────────────────────────────────────────────
│ Laps < 𝟏.𝟎 │ 🧠 Mentally Resident │ Then non-lapped mentally resident items.
─────────────────────────────────────────────────────────────────────────────────────
│ Laps < 𝟏.𝟎 │       🪜 On Deck     │ Then non-lapped On Deck items.
─────────────────────────────────────────────────────────────────────────────────────
```
