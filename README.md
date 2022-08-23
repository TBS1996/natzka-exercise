
#task-master


## About

A daemon that spawns instances chosen by a configuration file. This file also decides the conditions on which it will restart processes that have stopped.


## Installation  

Simply clone the repository to your PC. If you want to use the example, make sure to have firefox, thunderbird, and nautilus installed.


## How to use

The program accepts exactly one argument, which is the path of the configuration file. If you wanna use the default one (provided you have the above-mentioned programs installed), simply write "cargo run config.toml" from the cloned directory. 

You can make your own configuration file, but it has to be a .toml file in the same style.

To add your own entries, simply copy the pattern and edit the name and the path of executable. 
The restart condition decides what should happen when a process stops. Either it'll never restart, or it'll always restart, or it will restart if it was closed normally and not by an error.

To check if the "OnError" choice works, you can start a process with this daemon, when you receive the PID, you can forcibly close the program by writing "kill -9 $PID". If the program's reset condition is set to "OnError" then this should trigger a restart. 


