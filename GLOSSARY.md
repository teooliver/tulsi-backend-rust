# Tulsi

Tulsi is a task management tool: people organise work into Tasks, group them under Projects, and track their progress by placing them in the Columns of a Kanban Board.

## Language

### Work

**Task**:
A single unit of work with a title and description. The central thing everything else in Tulsi organises, tracks or filters.
_Avoid_: Ticket, issue, card, item

**Deleted Task**:
A Task that someone has deleted. It no longer appears anywhere and can't be changed, but its History is kept.
_Avoid_: Archived task, removed task, trashed task

**Project**:
A named grouping of related Tasks. A Task belongs to at most one Project; deleting a Project leaves its Tasks in place, just without a Project. A Project is not tied to any Board: its Tasks may sit on different Boards.
_Avoid_: Epic, initiative

**Label**:
A named, optionally coloured tag that can be attached to any number of Tasks. Labels are shared across the whole system, so each Label name exists only once.
_Avoid_: Tag, category

### Kanban

**Board**:
A Kanban workspace made up of an ordered set of Columns. A Task is on a Board when it is in one of that Board's Columns.
_Avoid_: Workspace (reserved for a possible future top-level container), kanban

**Column**:
One ordered stage of a Board (e.g. "To do", "In progress", "Done"). A Column belongs to exactly one Board and is gone if its Board is deleted.
_Avoid_: Status, lane, stage, list

**Position**:
A Column's place in the left-to-right order of its Board.
_Avoid_: Order, index, rank

**Move**:
Placing a Task into a different Column. A Move is recorded in the Task's History.
_Avoid_: Transition, drag

### People

**User**:
A person with an account who can sign in to Tulsi.
_Avoid_: Account, member, person

**Author**:
The User who created a Task. It is set once, when the Task is created, and never changes.
_Avoid_: Creator, reporter, owner

**Assignee**:
The User currently responsible for doing a Task. A Task has at most one Assignee.
_Avoid_: Owner, responsible, assigned user

### History

**Task History**:
The chronological record of a Task's life: its creation, its deletion, and the meaningful changes made to it in between.
_Avoid_: Audit log, activity, changelog

**History Event**:
One entry in a Task's History: which kind of change happened, who made it, when, and the value before and after.
_Avoid_: Log entry, change, activity

**Event Type**:
The kind of change a History Event records: the Task being created or deleted, or a change of its title, description, Column, Assignee or Project.
_Avoid_: Action, change type

### Saved Views

**Saved View**:
A named set of Filters saved by one User, which can be run again later to see the Tasks that currently match. A Saved View belongs to its Owner and only the Owner can see it.
_Avoid_: Plan, saved search, view, query

**Filter**:
One condition in a Saved View, such as "Assignee is one of these Users" or "created after this date". A Task must meet every Filter in a Saved View to match, and meets a Filter if it matches any of the values listed in it.
_Avoid_: Criterion, rule, condition

**Owner**:
The User a Saved View belongs to.
_Avoid_: Author, creator

**Execute**:
Running a Saved View's Filters against the current Tasks to get the matching list.
_Avoid_: Run, apply, evaluate
