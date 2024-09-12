
  

#  grgry

  

  

**grgry** (pronounced "**gregory**") is a CLI tool designed to perform git operations en masse, allowing you to bulk execute git commands across multiple repositories. It is heavily inspired by [ghorg](https://github.com/gabrie30/ghorg) and written in Rust as an introduction to the language.

  

##  Why grgry?

  

grgry is particularly helpful if you need to:

1. Make changes across multiple repositories simultaneously.

2. Onboard new team members by setting up their repositories.

3. Manage multiple git providers and accounts simultaneously.

  

##  Supported Providers

1. Github

2. Gitlab

  

##  Setup

  

###  Installation

You can install grgry by [downloading](https://github.com/yingrjimsch/grgry/releases) the binaries directly.

  

After that you can move it into your `/usr/local/bin` directory to make it executable on linux:
```bash
wget https://github.com/Yingrjimsch/grgry/releases/download/v1.0.3/grgry-v1.0.2-x86_64-unknown-linux-gnu.tar.gz
tar -xvzf grgry-v1.0.2-x86_64-unknown-linux-gnu.tar.gz
sudo mv grgry-v1.0.2-x86_64-unknown-linux-gnu/grgry /usr/local/bin/
rm -rf grgry-v1.0.2*
grgry --version
```
or add it to your environment variables on **windows** with powershell (as admin):
```powershell
$zipUrl = "https://github.com/Yingrjimsch/grgry/releases/download/v1.0.2/grgry-v1.0.2-x86_64-pc-windows-msvc.zip"
$zipFileName = "grgry-v1.0.2-x86_64-pc-windows-msvc"
$zipFile = "$zipFileName.zip"
$extractDir = "$zipFileName-extracted"
$destinationPath = "C:\Program Files\grgry"

Invoke-WebRequest -Uri $zipUrl -OutFile $zipFile
Expand-Archive -Path $zipFile -DestinationPath $extractDir

if (-Not (Test-Path -Path $destinationPath)) { New-Item -ItemType Directory -Path $destinationPath }
Move-Item -Path "$extractDir\$zipFileName\grgry.exe" -Destination $destinationPath -Force

Remove-Item -Path $zipFile 
Remove-Item -Recurse -Force -Path $extractDir

$env:Path += ";$destinationPath" 
[System.Environment]::SetEnvironmentVariable("Path", [System.Environment]::GetEnvironmentVariable("Path", [System.EnvironmentVariableTarget]::Machine) + ";$destinationPath", [System.EnvironmentVariableTarget]::Machine)

grgry --version
```
  
  ###  Update
  To update grgry you can either install it again following the example above or you can make use of the update command which automatically switches to the latest version.


  In unix systems this can be done by:
  
  ```bash
  sudo -E grgry update
 ```


###  Create Config File
The configuration file should be created the first time you execute the `grgry` command.

If you want to manually create a configuration file, add a file named ***grgry.toml*** inside the ***.config*** folder in your **$HOME** directory


Linux:

```bash
mkdir $HOME/.config

touch  $HOME/.config/grgry.toml
#or
touch  ~/.config/grgry.toml
```
or Windows:
```powershell
New-Item $HOME\.config\grgry.toml -Force
```

This config file stores your [profiles](#Profile), which you can switch between to access the correct provider with the appropriate user.


### Chef's recommendation
To simplify workflow aliases can be created that map the great grgry functionality into git commands.
For `grgry clone` and `grgry quick` it is straightfoward, for `grgry mass` a special command `alias` can be used providing a smoother usage, instead of writing `grgry mass "status --porcelain" --regex ".*"` the alias allows you to run `git mass status --porcelain --regex ".*"` without any apostrophes whatsoever.


Linux:
```bash
git config --global alias.gclone '!grgry clone'
git config --global alias.quick '!grgry quick'
git config --global alias.mass '!grgry alias'
```


Windows:
```powershell
git config --global alias.gclone "!grgry clone"
git config --global alias.quick "!grgry quick"
git config --global alias.mass "!grgry alias"
```


##  Available Commands  

###  Clone

The `grgry clone` command can clone a group, user, or organization. It accepts the following parameters:

-  `directory`: (Required) Name of the group/org/user to clone .
- `-f , --force` Specify if the base directory should be removed before cloning or only a pull is necessary (default false).
-  `-u, --user`: (Optional) Specify if the directory is a user directory or not (default false).
-  `-b, --branch`: (Optional) Clone specific branch (if not specified the deefault branch is cloned).
-  `--regex`: (Optional) Filter repositories to clone using a regex pattern.
-  `--rev-regex`: (Optional) Filter repositories to clone using a regex pattern exclusion.



If a repository does not match the provided branch or regex pattern, it will be skipped. If the repo already exists it will be pulled by default, to reclone from scratch the `-f, --force` parameter can be set.

  

###  Quick

The `grgry quick` command performs `git pull --rebase`, `git add`, `git commit`, and `git push` on one or many repositories together. It accepts the following parameters:

-  `message`: (Required) Commit message same as `git commit -m message`.

-  `--regex`: (Optional) Filter repositories to clone using a regex pattern.

-  `--rev-regex`: (Optional) Filter repositories to clone using a regex pattern exclusion.

-  `-s, --skip-interactive`: Don't ask for permission to execute command per repository (default is false).

  

If a repository does not match the regex pattern or has no changes, it will be skipped.

If a local branch is not on the origin it will be pushed using `--set-upstream`.

If you want to know more about what changed you can type `m` on the interactive mode which returns the `git diff` of the repository.

  

###  Mass

The `grgry mass` command can be used for executing single line git commands for multiple repositories at once (e.g. `git status --porcelain` --> `grgry "status --porcelain"`) It accepts the following parameters:

-  `command`: (Required) This is the command to execute as if it was a git command without git prefix.

-  `--regex`: (Optional) Filter repositories to clone using a regex pattern.

-  `--rev-regex`: (Optional) Filter repositories to clone using a regex pattern exclusion.

-  `-s, --skip-interactive`: Don't ask for permission to execute command per repository (default is false).

  

###  Profile

  

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

  

####  Profile commands:

-  `activate`: Activate a specific profile to be used for cloning.

-  `add`: Add a new profile interactively. Note: You'll be prompted to provide a personal access token for the provider. Without the token, you'll only be able to clone public repositories.

-  `delete`: Delete a profile that is not in use or is incorrectly configured.
- `show`: Show the current activated profile. All profiles can be listed by adding `-a, --all`

### Update
The `grgry update` command gets the latest release from GitHub and replaces the binary at the correct place. This feature is experimental for now and only works if executed with `sudo -E` rights.
  

##  Contributions

Contributions, improvements, and feature requests are welcome! Keep in mind that this project was created for personal use and as a learning project for Rust. The goal was to make it adaptable and fast.


##  Tasks in progress

- [ ] Release pipeline for cargo

- [ ] Change the command about and long about to something clearer

-  [x]  <del>Create submodules to structure the code / refactoring<del>

-  [x]  <del>`grgry show` with parameter `--all` showing the current activated profile or all profiles</del>

-  [x]  <del>add option to `pull --rebase` in `grgry quick`<del>

- [ ] add an option to `--ammend` commits

-  [x]  <del>open issue with walkdir to stop at .git folder instead of traversing into the folder (speedup)</del>

- [ ] try to work with https://github.blog/open-source/git/get-up-to-speed-with-partial-clone-and-shallow-clone/ to speeden up the git clone --> minimal speedup leads to lower prio

-  [x]  <del>quick default en mass instead of default singular<del>

-  [x]  <del>try https://statics.teams.cdn.office.net/evergreen-assets/safelinks/1/atp-safelinks.html for a `grgry update` functionallity<del>

-  [x]  <del>check if git is installed (if not assert break)<del>

-  [x]  <del>create alias for git to use instead of mass<del>

-  [x]  <del>force clone to remove `targetbasepath` and reclone from scratch<del>

- [ ] add configurable `grgry quick` for people that don't want to follow the same workflow

- [ ] quick, clone, mass in three different ways: interactive, multiselect, skip-interactive (regex still working in addition) (default interactive)