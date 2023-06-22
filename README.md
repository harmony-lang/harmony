<div align="center">

# 🎶 Harmony

*Functional programming made easier*

</div>

## 🌟 Introduction

Harmony is a functional programming language that compiles to JavaScript.
The entire goal of the design of Harmony is to make it easier for programmers that are new to functional programming to learn the concepts of functional programming. Harmony is designed to be a simple, easy to learn language that is still powerful enough to be used in real world applications.

## 📚 Table of Contents

- [🎶 Harmony](#🎶-harmony)
  - [🌟 Introduction](#🌟-introduction)
  - [📚 Table of Contents](#📚-table-of-contents)
  - [⚙️ Installation](#⚙️-installation)
  - [🚀 Usage](#🚀-usage)
    - [⌨️ Command Line Options](#⌨️-command-line-options)
  - [💡 Examples](#💡-examples)
    - [👋 Hello World](#👋-hello-world)
    - [➗ Factorial](#➗-factorial)
  - [📖 Documentation](#📖-documentation)
  - [🤝 Contributing](#🤝-contributing)
  - [📄 License](#📄-license)

## ⚙️ Installation

Harmony is available on [github](https://github.com/harmony-lang/harmony) and can be installed using [cargo](https://doc.rust-lang.org/cargo/).

```bash
cargo install harmony-lang
```

## 🚀 Usage

Harmony can be used as a command line tool to compile harmony files to JavaScript.

```console
$ harmony <file> [options]
```

### ⌨️ Command Line Options

| Option | Description |
| --- | --- |
| `-h`, `--help` | Prints help information |
| `-V`, `--version` | Prints version information |
| `-k`, `--keep` | Keep the generated JavaScript file |
| `-o <file>`, `--output <file>` | Output the generated JavaScript to a file |
| `-v`, `--verbose` | Prints verbose output |

## 💡 Examples

### 👋 Hello World

```harm
module HelloWorld

import Console.IO as IO

fun main =
    IO.println "Hello World!"
```

### ➗ Factorial

```harm
module Factorial

import Console.IO as IO

fun factorial(n: int) -> int =
    if n == 0 then 1
    else n * factorial (n - 1)

fun main =
    IO.println (factorial 5)
```

## 📖 Documentation

Documentation for Harmony can be found [here](https://harmony-lang.github.io/harmony/).

## 🤝 Contributing

Contributions are very welcome! Please read [CONTRIBUTING.md](https://github.com/harmony-lang/harmony/blob/master/CONTRIBUTING.md) for more information.

## 📄 License

Harmony is licensed under the [GNU General Public License v3.0](https://github.com/harmony-lang/harmony/blob/master/LICENSE).