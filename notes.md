## general design

prefix
description
priority:
from 0 to 100 (where 100 is most critical)
tag
status:
todo, done
  
"*todo" : prefix that lets brakoll know that this line is an issue
Brakoll works by searching through the current directory and its children, finding all the lines with a "*todo" prefix and returning current issues
  
---
  
### Examples

This is line 246 in an example file "my_application/src/server.rs":
  
``` rust
// *todo - d: fix formatting issue in debug statement, p: 10, t: debug, s: todo
```
  
From within a directory (your project directory for instance) you can run the "get" subcommand without any extra arguments to get all the issues:
  
``` terminal
brakoll get
```
  
And recieve this output (the issues with most critical priority are listed last, those of poor priority or those with the "done" status are printed first):
  
``` terminal
2 issue(s) were found.

=== 80: done ===
file: /Users/user/dev/my_application/src/api.rs
line: 472, tags: n/a
Desc: refactor query logic

*** 10: todo ***
file: /Users/user/dev/my_application/src/server.rs
line: 246, tags: debug
Desc: fix formatting issue in debug statement

2 issue(s) were found.
```

  
---
  
## todo


