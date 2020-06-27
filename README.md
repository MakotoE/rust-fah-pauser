Pauses Folding@home when a specified program is running. This allows FAH to run "While I'm working" but also makes FAH stop for resource-intensive apps.

Create `~/.config/fah-pauser.yml` (or `%userprofile%\.config\fah-pauser.yml`) and list programs that should pause FAH when any of them are running.

```
PauseOn:
- devenv.exe
- rFactor2.exe
```

fah-pauser checks the process list every minute.