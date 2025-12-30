## Plan process

Create a new plan directory at `lightplayer/plans/<name>`. This will be the base directory for all the plan files.

Analyze the current scope of work, and create a plan file 00-questions.md with any questions we need to answer to build this.

Once created, ask me the questions ONE AT A TIME with:

- a summary of the current state
- your suggested course forward

I will answer or ask more questions. We will then move on.

We must ensure that we have a clear acceptance criteria for the plan. There should be a phase
at the end for verifying the acceptance criteria and debugging.

Once you're done with the questions, you will present me with a suggestion of plan phases,
like this example:

```
1. Extend ElfLoadInfo
2. Create Object Submodule Structure
3. Implement Object Layout Calculation
4. Implement Object Section Loading
5. Implement Object Symbol Map Building
6. Implement Symbol Map Merging
```

I will then make suggestions to change the phases, or add more phases.

Once I tell you that we're ready to start, you will save these phases to the
plan directory as 00-phases.md.

You will then create a new file called 00-overview.md with the overview of the plan.

Then you will create a new file for each phase, named like 01-phase-title.md.

Then commit the plan to git with the message "lpc: create plan <name>"

Each phase should end with a success criteria section, and all code should compile. Any
tests relevant to the phase should be run and pass.

Any warnings that aren't unused code (that will be used later) should be fixed.

The final phase should be a cleanup phase:

- remove any temporary code or TODOs, debug prints, etc.
- fix all warnings
- ensure all tests pass
- ensure all code is clean and readable
- remove the plan directory
- run 'cargo +nightly fmt' to format the code on lightplayer/ directory

Then commit the changes with a message like "lpc: complete plan <name>". Include details of the
effect of the plan in the commit message (but not the implementation details).
