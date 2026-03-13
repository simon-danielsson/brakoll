## todo

**todo**
- [ ] add status filter flags for the list() function (i.e. "brakoll -s o/p/c")
- [ ] implement a loading animation during search for issues
- [ ] sort the issues list before the list() function 
    1. the issues with "done" status should be printed absolutely first in order of priority (those with lowest priority first)
    2. the issues with "todo" status should be printed last in order of priority
- [ ] add "help/-h/--help" subcommand as well as showing the help at failed flag parse
- [ ] add "-t" flag to allow the user to filter which tags they want to see
- [ ] add "summary" subcommand to get a list of how many issues of each tag exist
- [ ] one cool thing would be a "tree" subcommand so that the user can see more visually in which files the most issues exist

**done**
- [x] instead of todo/done, i could instead do open/prog/clos to replicate a more regular issue tracker (this will have to be decided now first before any other features are implemented)
- [x] perhaps add some concatenation to the filename in the list() function in some way, perhaps turn the home folder into just "~"
- [x] add logic so that if the target file is inside a "target" directory it is ignored
