please work out this porject and then keep completing things.

at some point we want to be self hosted and have a good STD (witten in zen lang)

use .agent/zen_language_reference.md for guidance

remove all the old loop syntax:
// OLD - DO NOT USE:
// loop i in 0..10 { }           // Range iteration
// loop item in items { }         // For-each

we should only have simple loops like 
```
range(1,10).loop(i -> {})
or 
loop(true -> {})

review our changes and merge with main

clear up the random files around the pleace.




notes from Lyndon
- read the .agent folder to help you
- use .agent directory to store important meta infomation as files (global_memory.md, todos.md, plan.md, scratchpad.md)
- order your todos as an estimate
- use gh-cli (to manage github, issues, commits, merges, branches)
- cleanup after yourself (clean up files after you are done, you can self terminate if you think you are done done)
- use testing
- A good heuristic is to spend 80% of your time on the actual porting, and 20% on the testing.
- simplicity, elegance, praticality and intelegence
- you work better at around 40% context window (100K-140k) we can either prime or cull the ctx window
- use frequent git commits and pushes 
- code principles DRY & KISS
- merge to main when you think it is smart to 
- git commit frequently 
