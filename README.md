# Mach-O RPATH Modifier

## Introduction

This project demonstrates how to add and modify the `rpath` (runtime search path) in Mach-O files, the native executable format for macOS binaries. The name "Stealing the moon?" is a playful reference to the character El Macho (aka Mach-o) from the Despicable Me franchise.

Inspired by the talk: [Reverse Engineering macOS](https://www.youtube.com/watch?v=S9FFzsF0aIA)

## Project Goal

My aim is to replicate the functionality of `install_name_tool` for adding `rpath` to Mach-O files. This tool is crucial for managing dynamic library dependencies in macOS applications.

## How it Works

### Using `install_name_tool` (for reference)

Here's how you would typically use `install_name_tool` to add an `rpath`:

```bash
# Compile a sample program
xcrun clang helloworld.c -o helloworld

# Add an rpath
install_name_tool -add_rpath "hey/how/are/you" helloworld 

# Verify the change
otool -l helloworld
```

After running these commands, you should see a new load command in the output:

```
Load command 17
          cmd LC_RPATH
      cmdsize 32
         path hey/how/are/you (offset 12)
```

### Current Implementation

This project aims to reproduce the same behavior without relying on `install_name_tool`. I'm working on manipulating the Mach-O file structure directly to add the `LC_RPATH` load command.

## Current Status

Current implementation is currently encountering corruption issues when modifying the file ( what a surprise! :D ).


## Disclaimer

This tool is for educational purposes only. Always ensure you have the right to modify any binaries you're working with.