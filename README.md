# terminator
Terminal-based learning tool for people studying for CompTIA certification exams.  This tool is intended to increase
familiarization using the terminal.  Users interact with the program to answer the questions, and even enter commands
that would normally be executed in a terminal using commands, switches, options, and flags.

# Notes:

1. `libsqlite3-dev` must be installed on machine if repository is cloned; this is not included with SQLite3.
   - On Debian-based Linux machines, this can be performed by entering the following commands:<br>
     1. `apt search libsqlite3-dev`: Searches the `apt` package manager for the tool. If the result of the search says 
        `[installed]`, then you already have this installed on your machine.<br>
     2. `sudo apt install libsqlite3-dev`: Installs the SQLite3 development files.
2. `build-essential` and `g++-multilib` will also be required; however, these might already be installed on your system.