# RANDL


Random Downloader

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

And if you do not want to download it, You can use the `dry-run` flag.

The `pull` subcommand also have the `from` flag that will pull from a specific url without adding it, You can add download timeout by using the `timeout` flag and if you want to pull repeatedly you can use `repeat` flag followed by the number of how much you want to repeat instead of running randl repeatedly.

The `no-confirm` flag in `pull` subcommand is used to skip all confirmation dialogs during pulling.

### Migrating from RTD
This project was previously known as RTD. To migrate, update your binary name from `rtd` to `randl`. Your existing repos list at `~/.config/rtd/` will need to be moved to `~/.config/randl/`.

I did not know there were other CLI tools called RTD, To avoid conflict, I decided to rename it to randl.

## How it works

1. Add & sync
Add a repository URL and sync it locally. This downloads the repo index to your machine.

2. Pull
randl picks a random repository from your local index, then picks a random line from it.

3. Reward or nested?
- If the line is a URL → you get that file as your reward.
- If the line starts with `Nested`, randl fetches that repo and picks a random line from it, repeating until it hits a reward.

## Creating your own repository

Creating your own repository is really simple, You just need:

1. An internet connection.

2. A working http or https server.

3. A text editor.


What you need to do is to run the http/https server and create a text file that are accessible through the http/https server. Here is an example of the repository:

```
# This is a reward
https://pastebin.com/raw/sqg8Ay0d
# If you want to make a nested repository
# that links your repository with others,
# you can use the Nested tag, like this:
Nested https://gist.githubusercontent.com/Bimasakti1024/c05d38ef8b93b8fd7dfb861977dd48e7/raw/37589e33d1547038c4c69e4c9fd796644c73071b/randl-repo.txt
```

## License

This project is licensed under the MIT License.

