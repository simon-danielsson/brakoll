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
  
Having a good method of tracking what's finished and what needs to be done is crucial once a project grows large enough. Of course, for team-driven development having a full-featured issue tracker such as GitHub or similar is preferable, but for solo-developers GitHub can feel like an over-engineered solution to a simple problem.
  
My goal with Brakoll was to create an issue tracker that is portable between machines and requires no setup, configuration-file, folder or specialty development-environment to use.
  
(The name "Brakoll" is derived from the swedish saying "bra koll" - if someone is living their life with structure and purpose, a swedish person may say they have "bra koll".)
  
The idea for this project was inspired directly by [this video](https://www.youtube.com/watch?v=8NdRGmp70Go).
  
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
  
When you want to add something to your issue list you simply type it out in your project (tip: create a snippet for this; for example "issue").
   
``` rust
// *brakoll - d: fix typo in debug print, p: 10, t: debug, s: open
fn debug() {
    println!("debugG")
}
```
  
- d: description of the issue (obligatory)
- p: priority from 0 to infinity where the highest number takes priority (optional - fallback: 0)
- t: tag (optional - fallback: n/a)
- s: status [ (op)en | (pr)ogress | (cl)osed ] (optional - fallback: open)
  

> [!IMPORTANT]  
> * Issues are currently only single-line! If you want a long description, write it all on a single line or alternatively refer to a bigger document somewhere else inside your description.  
> * The prefix "*brakoll" is required but whatever is before it on the line is of no importance to the parser and will be ignored, e.g "#", "//", "--" or any other comment syntax.
  
Here's a way to integrate Brakoll into your neovim config using luasnip:
  
``` lua
local ls = require("luasnip")
local s = ls.snippet
local t = ls.text_node
local i = ls.insert_node

return {
    s("issue", {
        t("*brakoll - d: "), i(1),
        t(", p: "), i(2),
        t(", t: "), i(3),
        t(", s: open"),
    }),
}
```
  
### Subcommands and flags
  
All the issues listed, sorted by priority and status:
  
``` terminal
brakoll
```
  
An optional target path can be added (works alongside other flags):
  
``` terminal
brakoll <relative path>
```
  
Filter issues by tag:
  
``` terminal
brakoll -t <tag>
```
  
Filter issues by status:
  
``` terminal
brakoll -s <status>
```
  
Filter issues by description:
  
``` terminal
brakoll -d <keyword>
```
  
Limit search to zero depth (i.e. no recursion):
  
``` terminal
brakoll -r
```
  
Summary of all issues:
  
``` terminal
brakoll summary
```
  
Display help and version information:
  
``` terminal
brakoll help
```
  
---
<div id="license"></div>

## 📜 License
This project is licensed under the [MIT License](https://github.com/simon-danielsson/brakoll/blob/main/LICENSE).  
  
---
<div id="dependencies"></div>

## 🛠 Dependencies
  
- [crossterm](https://github.com/crossterm-rs/crossterm)  
- [walkdir](https://github.com/BurntSushi/walkdir)  
- [dirs](https://codeberg.org/dirs/dirs-rs)  
