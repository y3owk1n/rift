> [!NOTE]
> This is a fork of [rift](https://github.com/acsandmann/rift) that fixes some fundamental issues that otherwise is not usable at all for me.
> Scroll down to see more about the changes and why this fork exists

<div align="center">

# Rift

  <p>Rift is a tiling window manager for macOS that focuses on performance and usability. </p>
  <img src="assets/demo.gif" alt="Rift demo" />

  <p>
    <a href="https://github.com/acsandmann/rift/actions/workflows/rust.yml">
      <img src="https://img.shields.io/github/actions/workflow/status/acsandmann/rift/rust.yml?style=flat-square" alt="Rust CI Status" />
    </a>
    <a href="https://github.com/acsandmann/rift/commits/main">
      <img src="https://img.shields.io/github/last-commit/acsandmann/rift?style=flat-square" alt="Last Commit" />
    </a>
    <a href="https://github.com/acsandmann/rift/issues">
      <img src="https://img.shields.io/github/issues/acsandmann/rift?style=flat-square" alt="Open Issues" />
    </a>
    <a href="https://github.com/acsandmann/rift/stargazers">
      <img src="https://img.shields.io/github/stars/acsandmann/rift?style=flat-square" alt="GitHub stars" />
    </a>
  </p>
</div>

## Fork related information

### Fixes of this fork

- no longer have any ghost windows with cmd+w
- if cmd+w happens on the last window, it will try to focus any window that is nearest to it
- auto switch to the workspace when open an app that has rule that moves to another workspace
- auto switch to the workspace when moving window to another workspace
- when first launching, it will try to stay on the same workspace of the current focused window
- when quiting rift, it will try to move all the windows (including hidden) to its position, so that we can use it right away
- when wake from sleep, everything should still be the same as before sleep (layouts, arrangments, windows, etc)
- performance improvements on race conditions
- disabled animation by default
- added more tests
- fixed `--validate` flag to actually validate the config file instead of the restore file

### How to install this fork?

- There's no binary release for this yet, to not confuse anyone
- You can clone this repo and build it yourself with rust
- I normally just use this in my nix-darwin with home manager, just search `rift` in my nix-darwin repo [here](https://github.com/y3owk1n/nix-system-config-v2)
- No configuration changes, exact same `rift` configuration will still work

### Contribute back to the original repo?

- I don't mind to contribute back to the original repo, as these are the fundamental fixes that makes it usable or replace aerospace for me.
- However, I am not inclined towards splitting out the codebase, cherry-picking the fixes, go through pull request, do changes and all the hassle to get them merged.
- If upstream accepts full changes as it is, I can just create a PR directly to the original repo.
- Anyone are welcome to take these code and pick them and contribute back to upstream too, I don't mind.
- The only reason this fork exists is to make it work for me if i want to use rift as my twm.

### Will this fork be constantly pulling changes from upstream?

- Probably not, especially there's lots of code changes since then, I am not sure if I have the time to do rebasing for it.

### Will this fork be gone anytime soon?

- For now probably not, but if upstream has all of the fixes that I can live with it, I will just use the upstream instead.

### Will this fork eventually becomes something else with new branding?

- I don't know, as most part are from the original repo and they owns the credit.
- There's no new features being added as for now, but just fixes.
- One day if it drifted away from the original repo direction, probably thats the time.

## Features

- Multiple layout styles
  - Tiling (i3/sway-like)
  - Binary Space Partitioning (bspwm-like)
- Menubar icon that shows all of the workspaces and the layouts within <details> <summary><sup>click to see the menu bar icon</sup></summary><img src="assets/menubar.png" alt="Rift menu bar icon" /></details>
- MacOS-style mission control that allows you to visually navigate between workspaces <details><summary><sup>click to see mission control</sup></summary><img src="assets/mission_control.png" alt="Rift Mission Control view" /></details>
- Focus follows the mouse with auto raise
- Drag windows over one another to swap positions
- Performant animations <sup>(as seen in the [demo](#rift))</sup>
- Switch to next/previous workspace with trackpad gestures <sup>(just like native macOS)</sup>
- Hot reloadable configuration
- Interop with third-party programs (ie Sketchybar)
  - Requests can be made to rift via the cli or the mach port exposed [(lua client here)](https://github.com/acsandmann/rift.lua)
  - Signals can be sent on startup, workspace switches, and when the windows within a workspace change. These signals can be sent via a command(cli) or through a mach connection
- Does **not** require disabling SIP
- Works with “Displays have separate Spaces” enabled (unlike all other major WMs)

## Quick Start

Get up and running via the wiki:
<br>

[<kbd><br>config<br></kbd>][config_link]

[<kbd><br>quick start<br></kbd>][quick_start]
<br>

## Status

Rift is in active development but is still generally stable. There is no official release yet; expect ongoing changes.

> Issues and PRs are very welcome.

## Motivation

Aerospace worked well for me, but I missed animations and the ability to use fullscreen on one display while working on the other. I also prefer leveraging private/undocumented APIs as they tend to be more reliable (due to the OS being built on them and all the public APIs) and performant.
<sup><sup>for more on why rift exists and what rift strives to do, see the [manifesto](manifesto.md)</sup></sup>

## Credits

Rift began as a fork (and is licensed as such) of <a href="https://github.com/glide-wm/glide">glide-wm</a> but has since diverged significantly. It uses private APIs reverse engineered by yabai and other projects. It is not affiliated with glide-wm or yabai.

<!---------------------------------------------------------------------------->

[config_link]: https://github.com/acsandmann/rift/wiki/Config
[quick_start]: https://github.com/acsandmann/rift/wiki/Quick-Start
