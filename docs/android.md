## Building for Android

There is very preliminary support for building azurescens as an Android app.
You will need Android 5.0 or later, which corresponds to OpenGL ES 3.1 or
later.

First, modify `Cargo.toml` like so:

    [features]
    default = ["android"]

This is necessary because [`cargo apk`][cargo-apk] does not yet support
`--features`.

Then, run

    ./android/build-and-install

Make sure `docker` and `adb` are in your `$PATH`. You will likely need to be
root.

Once the build and install is complete, you will see a live dump of the Android
system log. At this point you can launch the app on your device in the normal
way.

If all is well you will see some delicious fractals. Touch the screen anywhere
to change the control parameter (equivalent of moving the mouse on the desktop
version). FPS and any errors will be written to the Android system log.

[cargo-apk]: https://github.com/tomaka/android-rs-glue/tree/master/cargo-apk
