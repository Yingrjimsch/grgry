
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
You can install grgry via 
```bash
cargo  install  grgry
```
or by [downloading](#) the binaries directly.

##  Create Config File

Create a configuration file named ***grgry.toml*** inside the ***.config*** folder in your **$HOME** directory
```bash
touch /home/user/.config/grgry.toml
```
```bash
touch ~/.config/grgry.toml
```
 This config file stores your [profiles](#Profile) , which you can switch between to access the correct provider with the appropriate user.
  
## Available Commands

### Clone
The `grgry clone` command can clone a group, user, or organization. It accepts the following parameters:
- `directory`: Name of the group/org/user.
- `--regex`: (Optional) Filter repositories to clone using a regex pattern.
- `-u, --user`: (Optional) Specify thet the repositories of a user should be cloned (default is false).
- `-b, --branch`: (Optional) Specify which branch to pull (default HEAD)

If a repository does not match the provided branch or regex pattern, it will be skipped.

### Quick
The `quick` command performs `git add`, `git commit`, and `git push` on one or many repositories together. It accepts the following parameters:
-   `message`: Commit message (required).
-   `--mass`: (Optional) Use regex to specify repositories to include.
-   `--no-interactive`: (Optional) When enabled, processes all repositories without prompting for confirmation (default is false).
-   `-b, --branch`: (Optional) Specify which branch to apply the operation on (default is current branch).

If a repository does not match the provided branch or regex pattern, it will be skipped.

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

### Mass (Experimental)

The `grgry mass` command lets you translate any git command into a mass-executable command. You can specify the git command (without the `git` prefix) and use a regex to determine which repositories the command should be executed on.

## Contributions

Contributions, improvements, and feature requests are welcome! Keep in mind that this project was created for personal use and as a learning project for Rust. The goal was to make it adaptable and fast.