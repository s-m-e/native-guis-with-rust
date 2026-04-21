#import "@preview/polylux:0.4.0": *

#set page(
  paper: "presentation-16-9",
  footer: align(right, text(size: .8em, toolbox.slide-number)),
  margin: (bottom: 2em, rest: 1em),
)
#set text(
  font: "Lato",
  size: 23pt,
)
#show math.equation: set text(font: "Lete Sans Math")
#show heading: set block(below: 2em)

#let title="Native GUIs with Rust"

#slide[
  #set page(footer: none)
  #set align(horizon)

  #text(1.5em)[#title]

  Sebastian M. Ernst \<ernst\@pleiszenburg.de\> \
  2026-04-21
]

#slide[
  = (Partially) hypothetical use case
  #show: later

  - Browsing, discovering and tagging of *image libraries*
    - Microscope images
    - Astronomical images (including scanned libraries)
    - Remote sensing data
    - Calibration / research stuff
  - More 100s of gigabytes at least, possibly *100s of terabytes* in size
  - Access via *plain file system* (e.g. NFS, GlusterFS, ZFS, etc.)
  - Indexing of folder structures and files in the background
  - Loading or generating thumbnails / previews in the background
]

#slide[
  = Spiritual background

  #show: later
  - Lots and lots of Qt in the past
  - Even Tcl/Tk has its merits ...
  - I miss VB6 ... like others?
]

#slide[
  = GUI library requirements (a.k.a. the wishlist)

  #show: later

  - Target platform: *desktop* (not mobile/web)
    - Linux, FreeBSD & Windows (not WSL), maybe MacOS
  - Command line & `ncurses` is cool but ...
  - Performant, *low latency* if any, clean *threading*
  - *Event-driven* instead of immediate-mode rendering (game engines etc)
  - Written in Rust? (very wishful thinking, Rust-only project)
    - No second / domain-specific language or run time environment?
  - Static linking down to libc? (again wishful thinking)
  - "Reasonably sized binaries" (browser rendering engines ...)
]

#slide[
  = The classic: Qt (C++) for Rust

  #show: later
  - invent.kde.org/sdk/rust-qt-binding-generator | archived
  - github.com/White-Oak/qml-rust | archived
  - github.com/cyndis/qmlrs | QtQuick, 10 yrs unmaintained
  - github.com/rust-qt | `ritual` (wrapper for C++), 5 yrs unmaintained
  - github.com/woboq/qmetaobject-rs | Qml-based, maintained, WIP
    - Example: github.com/gyroflow/gyroflow
    - Author eventually wrote Slint ...
  - github.com/KDAB/cxx-qt | Qml-based, maintained, WIP

]

#slide[
  = Some notes on Qt

  #show: later
  - Static linking is known "problematic"
  - Bindings from C++ to other languages are known "problematic"
  - QML is a hot mess, also slow-ish
  - If fully statically linked: large
  - Complicated, unsteady licence situation

]

#slide[
  = Other bindings to C++

  #show: later
  - Tcl/Tk | (dynamic) linking, fragmented, not well established for Rust
  - GTK | github.com/gtk-rs/gtk4-rs | (dynamic) linking, large
  - WX | mostly relies on GTK on Linux/FreeBSD
    - github.com/AllenDang/wxDragon | WIP
    - github.com/kenz-gelsoft/wxRust | unmaintained
  - FLTK | github.com/fltk-rs/fltk-rs | lightweight
]

#slide[
  = The "pure-Rust" landscape

  #show: later
  - github.com/iced-rs/iced | Elm-like, large, `wgpu` ...
  - github.com/lapce/floem | large, `wgpu` ...
  - github.com/emilk/egui | immediate-mode, large, `wgpu` ...
  - github.com/slint-ui/slint | Rust+DSL, non-trivial licencing
]

#slide[
  = Further reading

  #show: later
  https://github.com/rust-unofficial/awesome-rust#gui
]
