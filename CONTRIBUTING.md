# Contributing to Mr. Big

First of all, thanks in advance for contributing! We need your contributions! Check out the issues section.

## Pull Requests

When submitting a Pull Request please make sure the following holds:
* There is at least one issue related to the pull request.
* If there is an RFC related to the pull request, link it.
* If there is no related RFC, make sure you clearly state the importance and impact of the pull request.
* You are also contributing with [tests](how-we-test) to the features/changes suggested.

## New Releases

To make a new release do the following:
* Bump crates versions by setting `crateVersion` in `./.Cargo.data.yaml`
* Run the script `./scripts/generate_manifests`
* Tag the commit with `git tag -m "New version" 0.1.3`
* Update the [CHANGELOG.md](CHANGELOG.md). Change descriptions may be taken from the Git history, but should be edited to ensure a consistent format, based on [Keep A Changelog](https://github.com/olivierlacan/keep-a-changelog/blob/master/CHANGELOG.md).

## Required tools

Besides the rust toolchain, other tools are used for compiling and testing the repository:

* [cargo-make](https://www.google.com/url?sa=t&rct=j&q=&esrc=s&source=web&cd=1&cad=rja&uact=8&ved=2ahUKEwjnw7iVjOrnAhVPrxoKHR0SC-MQFjAAegQIAxAB&url=https%3A%2F%2Fgithub.com%2Fsagiegurari%2Fcargo-make&usg=AOvVaw1Qge8hMXQXhWjRBkuAEsD5) `0.25.0` or newer
* [grpc_cli](https://www.google.com/url?sa=t&rct=j&q=&esrc=s&source=web&cd=1&cad=rja&uact=8&ved=2ahUKEwibi_GjjOrnAhVBzIUKHQS_CHcQFjAAegQIARAB&url=https%3A%2F%2Fgithub.com%2Fgrpc%2Fgrpc%2Fblob%2Fmaster%2Fdoc%2Fcommand_line_tool.md&usg=AOvVaw1YSr5Zt5tMTec9lqS23MPD)
* bash
* [docker](https://www.google.com/url?sa=t&rct=j&q=&esrc=s&source=web&cd=1&cad=rja&uact=8&ved=2ahUKEwiIvJexjOrnAhWM3oUKHdlzD5cQFjAAegQIDxAC&url=https%3A%2F%2Fwww.docker.com%2F&usg=AOvVaw3p9e1qPvdfjCrUwPYAhUlS)

If you're willing to use VSCode, we recommend [developing in a container](developing-in-a-container) and you don't have to install any of the tools above, except for docker.

## Using VSCode

You are free to use any editor you like.
Nonetheless, we suggests some guidelines for using VSCode, which make it fairly easy to contribute to the project.

### Opening the project

To open the project in VSCode, choose `File -> Open Folder ...` and pick the root folder of this repo.

### Developing in a container

To develop with a consistent and easily reproducible toolchain, developing in a container is a great approach, we recommend it.

To do so, install the extension **Remote - Containers** (`ms-vscode-remote.remote-containers`). You'll find the specs related to devcontainer in the [devcontainer folder](.devcontainer).
 This setup works for OS X out of the box, but not for Linux (at the time of writing this doc).
 Feel free to contribute in order to support other OSes.

 ### Other extensions

 The following list of VSCode extensions is recommended:

 ```
 better-toml
 markdown-all-in-one
 rust
 syntax-highlighter
 vscode-kubernetes-tools
 vscode-docker
 ```

 At the time of writing this, syntax highlighting for Rust in VSCode is not so great. One suggestion is to use the extension `syntax-highlighter` mentioned above with the following settings added to your `settings.json`:

 ```js
    "syntax.highlightLanguages": [
        "rust",
    ],
    "workbench.colorCustomizations": {
        "[Default Dark+]": {
            "syntax.type": "#1db461",
            "syntax.scope": "#5aa8a0",
            "syntax.function": "#6ca8dd",
            "syntax.variable": "#e0d890",
            "syntax.number": "#da8e50",
            "syntax.string": "#c7704e",
            "syntax.comment": "#b6561e",
            "syntax.constant": "#A89F9B",
            "syntax.directive": "#fdfdfd",
            "syntax.control": "#7ac3da",
            "syntax.operator": "#ffffff",
            "syntax.modifier": "#8cfff5",
            "syntax.punctuation": "#A1887F",
        }
    },
```

## How we test

As much as possible we want to have [unit tests](https://doc.rust-lang.org/rust-by-example/testing/unit_testing.html).

Integration tests should go in the [tests](folder).

Please consider using the approach of [property based testing](https://fsharpforfunandprofit.com/posts/property-based-testing-2/).
