# dosoxide
Yet another attempt to bring DOS back to the stage. Now in Rust, for some reason.

### What?
It's basically a DOS extender (go into 32 bit mode, access full memory, etc) but in Rust. I tried to make it somewhat modern. I tried.

### Why?
Because I know someone who develops a game for DOS and I hated to see how they struggle with the legacy garbage tools.

### How?
You need rust-src and LLVM stuff.

```
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly
```

And then LLVM, on Windows:
`winget install -e --id LLVM.LLVM`

You hopefully know how to get it on Linux.

Then compile by running build.bat/build.sh, I primarily tested this on Arch Linux with DosBox X through Flatpak.
That's basically it.

### Examples?

Got you. https://github.com/Davilarek/dosoxide_app_example
