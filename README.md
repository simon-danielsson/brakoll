<h1 align="center">
    Brakoll
</h1>
  
<p align="center">
  <em>Simple issue tracker for coding projects</em>
</p>
  
<p align="center">
    <img src="https://img.shields.io/crates/v/brakoll?style=flat-square&color=blueviolet&link=https%3A%2F%2Fcrates.io%2Fcrates%brakoll" alt="Crates.io version" />
    <img src="https://img.shields.io/badge/license-MIT-green?style=flat-square" alt="MIT License" />
  <!-- <img src="https://img.shields.io/badge/Rust-stable-orange?style=flat-square" alt="Rust" /> -->
  <img src="https://img.shields.io/github/last-commit/simon-danielsson/brakoll/main?style=flat-square&color=blue" alt="Last commit" />
</p>
  
<p align="center">
  <a href="#info">Info</a> •
  <a href="#install">Install</a> •
  <a href="#usage">Usage</a> •
  <a href="#dependencies">Dependencies</a> •
  <a href="#license">License</a>
</p>  
   

---
<div id="info"></div>

## 📌 Information
  
Having a good method of tracking what's finished and what needs to be done is crucial once a project grows large enough. Of course, for team-driven development having a full-featured issue tracker such as GitHub or similar is preferable, but for solo-developers GitHub can feel like a quite over-engineered solution to a simple problem.
  
(The name "Brakoll" is derived from the swedish word "koll", which is the act of surveying or scanning something.)
  
---
<div id="install"></div>

## 📦 Install
    
``` bash
cargo install brakoll
```
   
---
<div id="usage"></div>

## 💻 Usage
    
### Adding a new issue
  
When you want to add something to your issue list you simply type it out in your project (I would advise you to create a snippet for this; for example "issue").
   
``` rust
// *brakoll - d: fix formatting issue in debug statement, p: 10, t: debug, s: todo
```
  
- d: description of the issue (obligatory)
- p: priority from 0 to infinity where the highest number is the most critical priority (optional - fallback: 0)
- t: tag (optional - fallback: n/a)
- s: status [done | todo] (optional - fallback: todo)
  

> [!IMPORTANT]  
> Issues are currently only single-line! If you want a long description, write it all on a single line or alternatively refer to a bigger document somewhere else inside your description. The prefix "*brakoll" is required but whatever is before it on the line is of no importance to the parser and will be ignored, e.g "#", "//", "--" or other comment syntax.

### Listing and reviewing issues
  
Subcommands and flags will be added in future versions, but right now all you have to do is type "brakoll" inside your current directory and all your issues within it and any children directories will be listed.
  
``` terminal
brakoll
```
  
**Typical terminal output**
  
``` terminal
2 issue(s) were found.

*** 50: done ***
file: /Users/user/dev/my_awesome_project/src/api.rs
line: 426, tag: refactor
desc: query parser for-loop

*** 10: todo ***
file: /Users/user/dev/my_awesome_project/src/main.rs
line: 108, tag: debug
desc: fix formatting issue in debug statement

2 issue(s) were found.
```


---
<div id="license"></div>

## 📜 License
This project is licensed under the [MIT License](https://github.com/simon-danielsson/brakoll/blob/main/LICENSE).  
  
---
<div id="dependencies"></div>

## 🛠 Dependencies
  
- [walkdir](https://github.com/BurntSushi/walkdir)  
