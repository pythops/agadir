<div align="center">
  <h1> 🚧 Work In Progress 🚧 </h1>
  <br>
  <img src="assets/logo.png"/>
  <br>
  <h1> Agadir (ⴰⴳⴰⴷⵉⵔ) </h1>
  <h2> Blogging over the terminal </h2>
</div>

## 🔌 Installation

### 📥 Binary release

You can download the pre-built binaries from the release page [release page](https://github.com/pythops/agadir/releases)

### 📦 crates.io

You can install `agadir` from [crates.io](https://crates.io/crates/agadir)

```shell
cargo install agadir
```

### ⚒️ Build from source

Run the following command:

```shell
git clone https://github.com/pythops/agadir
cd agadir
cargo build --release
```

This will produce an executable file at `target/release/agadir` that you can copy to a directory in your `$PATH`.

## 🛞 Naviguation

`j` or `Down` : Scroll down.

`k` or `Up`: Scroll up.

`G`: Go to the end.

`gg`: Go to the top.

`Enter`: Show the content of the post.

`Esc`: Go to the table of content.

## 📰 Post format

the posts **should** have the following format:

```
---
title: Your post title here
created_at: DD/MM/YYYY
modified_at: DD/MM/YYYY
---

Your post content goes here in Markdown format.
```

## ⚙️ Configuration

The main directory is `$HOME/.agadir`, and it can be overriden with `AGADIR` env variable.

Its structure is as follows:

```
.agadir/
├── key
└── posts/
   ├── assets/
   │  └── fig.png
   ├── post_1.md
   └── post_2.md
```

- `key`: This is the server signing key. It is generated once at the startup and used everytime the server restarts.
- `posts`: This is where the posts should be located.
- `assets`: This directory serves as a place to store images/figures for the posts.

## 🚀 Deploy

The default listening port is `2222` and can be customized with `--port` or `-p` cli option.

## 📋Todo

- [ ] Adjust the terminal size based on the client.
- [ ] Display images.
- [ ] Load posts from remote git repositories.

## 📸 Demo

```
ssh blog.pythops.com
```

## ⚖️ License

GPLv3
