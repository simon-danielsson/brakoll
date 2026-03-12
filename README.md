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
    
> [!IMPORTANT]  
> **WeatherAPI**  
> Brakoll queries [WeatherAPI](https://www.weatherapi.com/) to fetch its weather data. To use this application, you must supply your own API key. Details on how to generate a key can be found on [WeatherAPIs developer page](https://www.weatherapi.com/docs/). Add your key to a new file in your home ($HOME) directory named ".brakoll": `~/.brakoll`  

  
``` terminal
Subcommands
help : print help

Flags
-l <str> : choose city location (default: Stockholm. Cities with spaces must be enclosed with double quotes; refer to the example down below!)
-t : view result directly in stdout instead of a TUI
-f <int> : set number of days to forecast (max: 10. default: 5. If a number is missing the default is used, if a number is larger than max the max value will be used.)

Example usage:
brakoll -l "rio de janeiro" -f 8

Controls
[Esc] : quit
[Ctrl-C] : quit
```
   
---
<div id="license"></div>

## 📜 License
This project is licensed under the [MIT License](https://github.com/simon-danielsson/brakoll/blob/main/LICENSE).  
  
---
<div id="dependencies"></div>

## 🛠 Dependencies
  
- [crossterm](https://github.com/crossterm-rs/crossterm)  
- [home](https://crates.io/crates/home/0.5.12)  
- [rand](https://github.com/rust-random/rand)  
- [serde](https://github.com/serde-rs/serde)  
- [reqwest](https://github.com/seanmonstar/reqwest)  
- [tokio](https://github.com/tokio-rs/tokio)  
