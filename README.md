
#  grgry

  

**grgry** (pronounced "**gregory**") is a CLI tool designed to perform git operations en masse, allowing you to bulk execute git commands across multiple repositories. It is heavily inspired by [ghorg](https://github.com/gabrie30/ghorg) and written in Rust as an introduction to the language.

##  Why grgry?

grgry is particularly helpful if you need to:
1. Make changes across multiple repositories simultaneously.
2. Onboard new team members by setting up their repositories.
3. Search and filter information across many repositories.
4. Manage multiple git providers and accounts simultaneously.

##  Supported Providers
1. Github
2. Gitlab

##  Setup

###  Installation
You can install grgry by [downloading](https://github.com/yingrjimsch/grgry/releases) the binaries directly.

After that you can move it into your `/usr/local/bin` directory to make it executable on linux or add it to your environment variables on windows.


##  Create Config File
The configuration file should be created the first time you execute the `grgry` command.


If you want to manually create a configuration file, add a file named ***grgry.toml*** inside the ***.config*** folder in your **$HOME** directory
```bash
touch /home/user/.config/grgry.toml
```
```bash
touch ~/.config/grgry.toml
```
 This config file stores your [profiles](#Profile), which you can switch between to access the correct provider with the appropriate user.
  
## Available Commands

### Clone
The `grgry clone` command can clone a group, user, or organization. It accepts the following parameters:
- `directory`: (Required) Name of the group/org/user to clone .
- `-u, --user`: (Optional) Specify if the directory is a user directory or not (default false).
- `-b, --branch`: (Optional) Clone specific branch (if not specified the deefault branch is cloned).
- `--regex`: (Optional) Filter repositories to clone using a regex pattern.
- `--rev-regex`: (Optional) Filter repositories to clone using a regex pattern exclusion.

If a repository does not match the provided branch or regex pattern, it will be skipped.

### Quick
The `grgry quick` command performs `git add`, `git commit`, and `git push` on one or many repositories together. It accepts the following parameters:
- `message`: (Required) Commit message same as `git commit -m message`.
- `--regex`: (Optional) Filter repositories to clone using a regex pattern.
- `--rev-regex`: (Optional) Filter repositories to clone using a regex pattern exclusion.
- `-s, --skip-interactive`: Don't ask for permission to execute command per repository (default is false).

If a repository does not match the regex pattern or has no changes, it will be skipped. 
If a local branch is not on the origin it will be pushed using `--set-upstream`.
If you want to know more about what changed you can type `m` on the interactive mode which returns the `git diff` of the repository.

### Mass
The `grgry mass` command can be used for executing single line git commands for multiple repositories at once (e.g. `git status --porcelain` --> `grgry "status --porcelain`) It accepts the following parameters:
- `command`: (Required) This is the command to execute as if it was a git command without git prefix.
- `--regex`: (Optional) Filter repositories to clone using a regex pattern.
- `--rev-regex`: (Optional) Filter repositories to clone using a regex pattern exclusion.
- `-s, --skip-interactive`: Don't ask for permission to execute command per repository (default is false).

### Profile

The `grgry profile` command manages different profiles stored in the `grgry.toml` configuration file. A profile might look like this:

<pre><code> 
["Gitlab Profile"] 
active = false 
pulloption = "ssh" 
username = "Your Name" 
email = "your@email.com" 
baseaddress = "https://gitlab.com" 
provider = "gitlab" 
token = "glp-1234..." 
targetbasepath = "/your/base/repo/path" 

["Github Profile"] 
active = true 
pulloption = "https" 
username = "Your Name" 
email = "your@email.com" 
baseaddress = "https://api.github.com" 
provider = "github" 
token = "token github_pat_1233..." 
targetbasepath = "/your/base/repo/path" 
</code></pre>

#### Profile commands:
-  `activate`:  Activate a specific profile to be used for cloning.
-  `add`:  Add a new profile interactively. Note: You'll be prompted to provide a personal access token for the provider. Without the token, you'll only be able to clone public repositories.
-  `delete`:  Delete a profile that is not in use or is incorrectly configured.

## Contributions

Contributions, improvements, and feature requests are welcome! Keep in mind that this project was created for personal use and as a learning project for Rust. The goal was to make it adaptable and fast.

## Tasks in progress
- [ ] Release pipeline for cargo
- [ ] Change the command about and long about to something clearer
- [ ] Create submodules to structure the code / refactoring
- [ ] `grgry show` with parameter `--all` showing the current activated profile or all profiles
