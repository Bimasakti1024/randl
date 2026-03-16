<img src="assets/logo.svg" align="right" alt="randl logo" width=100>

# RANDL

*The internet is the reward pool*

A simple CLI to download random things from a repository.

randl is powered by a federated network of static-hosted repos. Anyone can host one, anyone can link to others.


## Quickstart

Here is how to add a repository:
```bash
randl repository add <NAME> <URL>
```

The first repository in this project is: [https://gist.githubusercontent.com/Bimasakti1024/c05d38ef8b93b8fd7dfb861977dd48e7/raw/randl-repo.txt](https://gist.githubusercontent.com/Bimasakti1024/c05d38ef8b93b8fd7dfb861977dd48e7/raw/randl-repo.txt)

And to remove a repository:
```bash
randl repository remove <NAME>
```
Here is how to list all available repositories in your configuration:
```bash
randl repository list
```


Before you can pull, You need to synchronize the repository using:
```bash
randl repository sync
```
You could also synchronize a selected repository by:
```bash
randl repository sync <NAME>
```

And also run `randl repo` instead of `randl repository` as a shortcut.

Here is how to pull from a repository:

```bash
randl pull
```
The pull subcommand have a flag called `max-depth` which will set the maximum depth for a nested repository.

If you want to set on what directory the reward should be downloaded, You can use the `output-directory` flag.

So for example if you want to save it to  `~/Downloads`:
```bash
randl pull --output-directory ~/Downloads
```

And if you do not want to download it, You can use the `dry-run` flag.

The `pull` subcommand also have the `from` flag that will pull from a specific url without adding it, You can add download timeout by using the `timeout` flag and if you want to pull repeatedly you can use `repeat` flag followed by the number of how much you want to repeat instead of running randl repeatedly.

For example if you want to pull 3 times without downloading you can run:

```bash
randl pull --repeat 3 --dry-run
```

And if you want to do it from a repository that you did not want to add:

```bash
randl pull --repeat 3 --dry-run --from <URL>
```

The `no-confirm` flag in `pull` subcommand is used to skip all confirmation dialogs during pulling.

### Migrating from RTD
This project was previously known as RTD. To migrate, update your binary name from `rtd` to `randl`. Your existing repos list at `~/.config/rtd/` will need to be moved to `~/.config/randl/`.

I did not know there were other CLI tools called RTD, To avoid conflict, I decided to rename it to randl.

## Configuration

randl stores its configuration at `~/.config/randl/config.toml`, which is automatically created on first run.

All keys in the `config.toml` file serve as default configuration which can be overridden by using flags.

For example if the `max_depth` key value is 3, you can temporarily modify it without touching the `config.toml` file by using the `max-depth` flag.

| Key | Default | Description |
|-----|---------|-------------|
| `max_depth` | `3` | Maximum depth for nested repository following |
| `output_directory` | `.` | Directory where rewards are saved |
| `repeat` | `1` | How many times to pull |
| `timeout` | `30` | HTTP timeout in seconds |
| `no_confirm` | `false` | Skip all confirmation dialogs |
| `dry_run` | `false` | Preview reward without downloading |
| `keep_cache` | `false` | Keep sync cache when removing a repository |

## How it works

1. Add & sync
Add a repository URL and sync it locally. This downloads the repo index to your machine.

2. Pull
randl picks a random repository from your local index, then picks a random line from it.

3. Reward or nested?
- If the line is a URL, you get that file as your reward.
- If the line starts with `Nested`, randl fetches that repo and picks a random line from it, repeating until it hits a reward.

## Creating your own repository

Creating your own repository is really simple. You just need:

1. An internet connection.
2. Somewhere to host a raw text file: GitHub Gist, Pastebin, GitHub Pages, or any HTTP server.
3. A text editor.

Create a text file with one URL per line. Here is an example:
```
# This is a reward
https://pastebin.com/raw/sqg8Ay0d
# If you want to link to another repository, use the Nested tag:
Nested https://gist.githubusercontent.com/...
```

Some free options to host your repository:

- [GitHub Gist](https://gist.github.com): easy and version controlled
- [Pastebin](https://pastebin.com): simple, no account needed (you need an account if you want to edit)
- [GitHub Pages](https://pages.github.com): best for larger repos, free with a GitHub account
- [0x0.st](https://0x0.st): temporary, can save up to a year

## License

This project is licensed under the MIT License.
