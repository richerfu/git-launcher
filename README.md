# Git launcher

A simple launcher for git project. Powered by [gpui](https://www.gpui.rs/) and [gpui-component](https://github.com/longbridge/gpui-component).

## Platform

- [x] macOS
- [ ] Windows
- [ ] Linux

## Configuration

Our configuration file path is `$HOME/.git-launcher/config.toml`. You can set it before you start our application.

```toml
[repo_config]
# read folder
base_dir = ["/Volumes/PSSD"]
# some folders should be ignored
ignore_dirs = ["node_modules", "target", ".git", "build", "dist"]
max_depth = 10
max_concurrent_tasks = 20

# you can ignore these config by default
[ui_config]
width = 600.0
height = 60.0

# setup application
[editor_config]
editor = "/Applications/Cursor.app"
```

## How to build

Download the repo and cargo build.

## How to pack

We use [cargo-packager](https://github.com/crabnebula-dev/cargo-packager) to pack it. You can pack it with the following bash:

```bash
cargo packager
```

## License

[MIT](./LICENSE)
