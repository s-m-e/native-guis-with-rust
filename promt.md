Build a minimal image viewer in Rust.

Use the crate "slint" for the UI.

The application is supposed to as small and minimal as possible, yet also as fast and "snappy" as possible. Latencies on user input are to be minimized.

The application must have two different views: "Overview" and "Image".

In the "overview" view, the window should display thumbnails for all images within the current working directory. Each thumbnail should be annotated with file meta data, i.e. file name and modification date. One thumbnail is always visually selected. Selection can be changed by clicking on an image or using the cursor keys on the keyboard to navigate through the thumbnails horizontally and vertically. If the number of thumbnails exceeds the size of the window, the user should be able to scroll.

Hitting the enter key or double clicking on an image makes the application activate the "image" view. In the "image" view, the currently selected image fills the entire window. No meta data is being displayed. Hitting the escape key makes the image disappear and the window is returned to its initial "overview" view. While in "image" view, also allow to go back and forth between images, but only support the left and right arrow keys. Go to the same images that those keys would select if in "overview" view.

The entire application should be able to enter and leave full screen mode by using the key "f". While thumbnails should be cached in memory for as long as the application is running, images shown at full size should only remain in memory as long as they are on display to conserve resources.

The application must have a CLI argument to specify a path. It is the images at this path which are supposed to be listed. Defaults to the current working directory.

The application is supposed to deal with folders containing more than ten thousand images. Each image file can be anywhere from a few kilobytes in size to over 100 megabytes. Supported formats: JPG, GIF, PNG, TIFF. Make sure to load thumbnails and metadata in the background while the application remains fully responsive to user input and events. Background loading should be handled by at least one separate thread, though if beneficial given the used crates and architecture, multiple threads can be used. If images contain their own thumbnails, use them instead of generating new ones. In general, load as little data from an image file as possible. Once the thumbnail of an image is present and its meta has been read, add it to the user interface. If it keeps the user interface more responsive, this may happen in batches of images instead of one at a time.
