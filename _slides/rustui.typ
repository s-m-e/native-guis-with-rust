#import "@preview/polylux:0.4.0": *
#import "@preview/rustycure:0.2.0": qr-code

#let title = "Native GUIs with Rust"
#let release = "2026-04-21"
#let url = "github.com/s-m-e/native-guis-with-rust"
#let color_bg = rgb("#333132")
#let color_fg = rgb("#bfbfbf")

#set page(
  paper: "presentation-16-9",
  footer: align(
    bottom,
    toolbox.full-width-block(
      fill: color_bg,
      inset: 8mm,
    )[
      #text(size: 12pt)[#release | #title | #url]
      #h(1fr)
      #text(size: 16pt)[#toolbox.slide-number / #toolbox.last-slide-number]
    ]
  ),
  margin: (bottom: 2em, rest: 1em),
  fill: rgb(color_bg),
)
#set text(
  font: "Open Sans",
  size: 22pt,
  fill: color_fg,
)
#show heading: set block(below: 2em)

#slide[
  #set page(footer: none)
  #set align(horizon)

  #qr-code(
    "https://" + url,
    width: 80mm,
    quiet-zone: false,
    dark-color: color_fg,
    light-color: color_bg,
  )

  #text(1.5em)[#title] \
  #text(0.8em)[Rust User Group Leipzig, #release]

  Sebastian M. Ernst \<ernst\@pleiszenburg.de\>
]

#slide[
  = (Partially) hypothetical use case \#1
  #show: later

  - Browsing, discovering and tagging of "chaotic" *image libraries*
    - Remote sensing data
    - Astronomical images (including scanned libraries)
    - Other R&D stuff, e.g. microscope or calibration imagery
  - More 100s of gigabytes at least, possibly *100s of terabytes* in size
  - Anywhere from 100k files & folders to 10M+ files
  - Access via *plain file system* (e.g. NFS, GlusterFS, ZFS, etc.)
  - Indexing of folder structures and files in the background
  - Loading or generating thumbnails / previews in the background
  - Partial loading of image files (where format supports it)

]

#slide[
  = (Partially) hypothetical use case \#2
  #show: later

  - Small tool for moving files around and basic diagnostics
  - Check for attached USB device
  - Format, mount, unpack tar files or similar, verify, unmount
  - Provide status output
  - Very non-technical target audience, colorful, buttons
  - Trivial updates

  \ Python script with PyQt/PySide, managed entirely by `uv`, does the job ... ?
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
