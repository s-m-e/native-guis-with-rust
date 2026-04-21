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
  = Requirements (a.k.a. the wishlist)

  Looking for a GUI library ...
  #show: later

  - Performant, *low latency* if any, clean threading
  - Written in Rust? (wishful thinking, Rust-only project)
  - Static linking down to libc? (again wishful thinking)
  - "Reasonably sized binaries"
  - Target platform: *desktop* (not mobile/web)
    - Linux & Windows, maybe MacOS
  - No second / domain-specific language?
  - No browser rendering engine or similar battleship
  - Command line & `ncurses` is cool but ...
]

#slide[
  = The classics: Qt

  #show: later
  - github.com/cyndis/qmlrs | QtQuick, 10 yrs unmaintained
  - github.com/rust-qt | `ritual` (wrapper for C++), 5 yrs unmaintained
  - github.com/woboq/qmetaobject-rs | Qml-based, maintained, WIP
    - Example: github.com/gyroflow/gyroflow
  - invent.kde.org/sdk/rust-qt-binding-generator | archived
  - github.com/KDAB/cxx-qt | maintained, WIP
  - github.com/White-Oak/qml-rust | archived

]

#slide[
  = The landscape


  #show: later
  https://github.com/rust-unofficial/awesome-rust#gui
]
