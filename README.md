# What is OnPurpose?
Over the past few years I have been on a journey to rethink how a computer can help with the problem of time and task management. This OnPurpose project exists as a place for me to implement my ideas. My initial goal is to implement a CLI and text based menu driven program and then expand to the phone with a Graphical UI and then make a desktop GUI that can be always visible and pinned opposite the OS taskbar. I expect this to be a multi-year journey, at the very least.

I should also mention that part of my goal is to influence and encourage these ideas to be integrated into other programs and into the OS itself. The reason is because my experience is that it is **not** the core idea but rather how convenient it is to implement that is the real barrier. I see this problem from professional therapists, to professionals helping care for the elderly, to institutions researching how to practically improve the lives of neurodiverse adults and children. The problem that I see is that the guidance is not followed because it is not convenient or natural enough to follow with today's software.

My aspiration with *OnPurpose* is to redefine expectations of what to do software is. I feel like the core ideas in today's to do software can still be generally implemented in a day planner. I believe the fact that the to do app is so commonly used as a sample program is a signal that to do apps map much more closely to how computers operate than what is intuitive to the human mind. In addition my experience with existing to do software is that it cannot properly represent all the kinds of work that we do, one example is an inability to enumerate supportive or responsive items that we expect to come up.

I believe that the general issue with to do software is that it is either too simple to be truly useful or features are added until the whole things becomes overly complicated and even still is not exactly doing what is desired. I have seen one of these two mistakes repeated countless times and I believe the solution to this problem rests in the word intuition. And speaking personally I have been stuck at this point for many years, even though I was spending a significant amount of time each week trying to figure out a path forward. My search centered around the idea that software needs to demonstrate understanding. The software needs to "get it" and I believe that is the key to working out what is intutive.

What I have worked out is that to do items should be tied to the outcome hoped for and backed by the reason, motivation, or value for why we are doing this thing. My hypothsis is that being explicit about this will result in better decisions for how we spend our time. To do software should help with all three phases of preparing, acting, and reflecting on the things we do. The interface should help with the goal of doing the right thing right now from the big picture down to the smallest literally next step. When you are waiting for something to happen it should have awareness or integrations to help you know when to return to something. It should help you remember different to dos related to your current location, headspace, or situation. And it should help with the process of stopping and resuming later so you can better remember what your doing. And finally it should help you recall and celebrate your effort independent of what actually came to pass.

## Drive to Next Steps, (i.e. Covering, To Dos, Hopes, and Reasons)
Have a list of things to do and cover that to the next step. To do's tie back to hopes and are done for reasons.

### Other uses of Covering
Can cover item by another item and can cover an item by a watcher that watches 

### Picking what to do, acting with awareness, and in context
Pay attention to what is mentally resident
See Upcoming
Pick mood and state willingness

### In the area work, multitasking and multi-purposing

## Integration with other systems (i.e. The Personal View)
Picking what to do

## Saving, Resuming, and Staying On Track

### Capturing
Take a screenshot, including scroll and simple interaction. Save accessibility data with the copy-paste 

## Reflection & Rewards

# Compiling OnPurpose
To try out **OnPurpose** you need the Rust toolchain installed (https://rustup.rs). I am currently doing development inside the WSL (Ubuntu) on Windows. I leverage Surreal DB as an integrated Database which requires installing some packages from apt-get because of the features I have enabled. Please note that at some point I expect to reach a need to switch over to being a native Windows app.

### Install Dependencies for SurrealDB
For Ubuntu
```
apt-get -y update
apt-get -y install \
	curl \
	llvm \
	cmake \
	binutils \
	clang-11 \
	qemu-user \
	musl-tools \
	libssl-dev \
	pkg-config \
	build-essential \
	protobuf-compiler
```
Otherwise see the following for instructions - https://github.com/surrealdb/surrealdb/blob/main/doc/BUILDING.md

# Platforms
As of this very moment I am writing a text based Linux CLI app that I am running inside the Windows Subsystem for Linux (WSL). At some point I expect that I will need tighter integration into the Windows platform and when that time comes I plan on switching over to being a Windows CLI app. When I get around to adding Phone support my current intensions are to focus on the Android platform. When the time comes I do also plan on investigating the idea of being a locally installed Web App compiled to Web Assembly.