# TODO items

Before this can be considered an MVP, the following items still need to be reworked.

## Proto
* Can MAC address be made to NOT be a structure (string?)?

## Server
* Should not need to be run by privileged user. No IP assignment is being done.  Is the act DORA privileged?
* Should it validate the provided interface? (it does today)
* Add more debug
  * what port are we running on 
* Add the following CLI command options:
  * --port non-default location of port
  * --dir where should the backup file be stored
* Switch to UDS by default
* Add systemd socket activation files (UDS)

## Client
* Add exit codes