# Taggytime

Time management and planning in one click

## About
**Taggytime** is a project dedicated to assist individuals in time management. Instead of letting users log their time spent in several categories of tasks, which apps like **Blocky Time** did a great job in, this project computes a "load factors" for each of the user's task to complete, based on factors including but not limited to the user's current schedule and expected / maximum workload associated with each task. 

## Status
This project is still in its early stage of development. Much of the core features are to be implemented. 

## Installation
This project is not completed, and cannot be installed yet. 

## Todo
Here is a list of non-core functionalities that shall be implemented in the future: 

* Time machine
* GUI
* Cache datetime property computations [important]
* Implement interval and setpos handling
* Optimize miv::advance_until() via elim. back-forth conversion.
* Use DFA and / or their genfun for optimization?
* Implement Recurrence parse from non-weekly events. 
* Handle week starting day.
* Use refinement typ for `ZoneOffset`
* Calendar event interval overlap chk
* `MinInterval` bounds chk
* Error-typ proper conversion.
* Implement VTimeZone / `ics` timezone.
* Organize util fns.
* Get rid of `MinInstant` offset to elim bugs
* Proof-read all raw arithmetics.
* Audit.

## Contact
If there is any further questions, please contact the author at `tianwei2@andrew.cmu.edu`.