##Project Description##
This is a GUI based program to setup a python development environment for all 3 platforms: windows, linux and macos.

The target user is assume beginnner, even not very familiar with all kinds of popular well-known softwares or tools.

The language of the UI should support Chinese and English, defaut to Chinese but user can change to English, and potentially can be other popular languages too.

##Components to be installed##
1. The package manager 'uv'
2. The base python version, choose the current most stable release as the default, but user changable to use other availab versions. Moreover, the program can be re-launched to only install one of the components. For example, change to install another version of python.
3. The latest stable release of VSCode and its all needed plugins for python development

##Other Requirements##
Note that this shall include all 'PATH' that shall be updated. For example, windows system env, shell profile (default to ~/.bashrc, but user selective for popular shells).

Preferrably user just need one click and the program then install all the needed components with reasonable default configs.

The program shall provide a "network test" action for user to test its current network condition, with a default empty proxy setting which user can setup in case it is behind a firewall.

The program shall provide also a CLI interface with a default config file or config example so that a user who likes CLI can run the CLI to achieve the same. The VSCode part could then become optional for CLI since user may only has terminal environment.

It shall provide not only executable, but also installer package build.

Suggest the most fit program language that produces the minimum executable or install package.
