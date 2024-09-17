# RsBar

## Overview

Simple status bar for `Hyprland`, written in Rust 

## Dependencies

RsBar use `GTK4` to draw its interface on the screen.

There the list of dependencies:
+ gcc
+ rust/cargo
+ pkg-config
+ cairo
+ glib
+ gtk4
+ gtk4-layer-shell
+ graphene

## Build

To build `RsBar` follow to RsBar directory and execute this command:

``` bash
cargo build --bin rsbar --profile release
```

After building `RsBar` binary will be in directory: `./target/release/`
 

