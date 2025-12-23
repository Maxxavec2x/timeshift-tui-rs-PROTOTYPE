
# Timeshift-tui-rs

A TUI for [Timeshift](https://github.com/linuxmint/timeshift), that allow user to manage snapshots in a easier way than with the CLI. For now, I have only seen GUI for this, and as I use headless environments I needed a TUI. I'm convinced this can be an helpful tool for sysadmins.

## About the project
This project is written in glorious Rust, using the [Ratatui](https://ratatui.rs/) lib. I am currently learning Rust, and this is my first real project in this language, so please do not expect too much from it.

## Dependencies

I develop this project using ```cargo 1.92.0 (344c4567c 2025-10-21) (Arch Linux rust 1:1.92.0-1)```
I also use crates other than Ratatui, and I will list them all here once the project reaches a stable state.

## How to use

To use it, just clone the repo and run ```cargo run```. One day, I will provide a binary release, but for now I am focusing on adding more features and polishing the project. I am also planning to create an AUR package once the project is mature enough.

## Contributions

All contributions are welcome, but since I am using this project mainly for learning purposes, please do not change everything in a single PR without justification. I **know** there are awful things in the codebase, but please understand that I am new to Rust and systems programming.

I want to understand every change in the codebase, so if you submit a PR that is hard for me to understand, do not expect it to be merged in 5 minutes. (I also have a job, so I obviously do not have time for FOSS during work hours.)
