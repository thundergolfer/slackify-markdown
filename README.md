# slackify-markdown [![Build Status](https://travis-ci.com/thundergolfer/slackify-markdown.svg?token=yHGWQ42iK2BPk1FjaUMc&branch=master)](https://travis-ci.com/thundergolfer/slackify-markdown)

Convert markdown into Slack's bastardized markdown

> ⚠️ **Warning:** This is my first Rust and it's an awful, hacky thing (but functional, atleast).

----

## Usage 

#### From your clipboard ✅

`CTRL-V` Copy the markdown you want converted to your clipboard. Then...

* **Mac OSX** -> `pbpaste | slackify-markdown | pbcopy`

... and paste it into Slack!

#### From your terminal

If you don't pipe anything to `slackify-markdown`, it will read everything you type into the terminal
until you hit `CTRL` + `D`, and then it will convert. 


## Install

#### Homebrew

Easiest way to install on macOS is by using [Homebrew](https://brew.sh/).

```
$ brew tap thundergolfer/homebrew-formulae
$ brew install slackify-markdown
```

#### Manual Installation

You can get binaries for OSX and Linux on this project's [releases page](https://github.com/thundergolfer/slackify-markdown/releases).

After downloading, you unzip the `.tar.gz` and move the binary to a place that's on your path (`$PATH` on Linux/OSX).


## Development

#### Code Overview

* [`main.rs`](src/main.rs) contains the basics of reading inputs and calling the conversion function, and also contains unit tests.
* [`slackdown.rs`](src/slackdown.rs) implements the Markdown -> 'Slackdown' conversion logic. It is a copy-and-hack of the `pulldown-cmark` crate's [`html.rs`](https://github.com/raphlinus/pulldown-cmark/blob/master/src/html.rs) module.
* [`escape.rs`](src/lib.rs) is a direct lift from [`pulldown-cmark`](https://github.com/raphlinus/pulldown-cmark/) and...
* [`lib.rs`](src/lib.rs) is the crate's default library file, and (in my understanding) allows for modules within `src` to refer to each other through `use crate::<BLAH>` 

#### Releasing

TravisCI is used to release _Linux_ and _OSX_ binaries each time a **tagged** commit is pushed to `master`.
 
## License

This project uses the [MIT](https://choosealicense.com/licenses/mit/) license, preserving the original copyright of code used from [`pulldown-cmark`](https://github.com/raphlinus/pulldown-cmark/).  