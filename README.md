# nansi
`nansi` is a simple tool for task automation. Its primary functionality is to execute a sequence of commands in a defined order. It was inspired by what `Dockerfile` is and what `ansible` is not.

## Features
- running commands
- OS independent
- indicates whether a command was executed successfully
- allows for dependency creation; if command `B` is dependent on command `A`, and command `A` was not executed successfully, command `B` is skipped

# Usage
```
nansi --help
```

## Remarks
`nansi` is OS independent so features related to a specific OS can be used by calling a specific shell. For instance, a command definition should look like in the following examples:

- Linux (`bash`):
```
"exec": "/bin/bash",
"args": [
    "mkdir",
    "~/test"
]
```

- Windows (`cmd`):
```
"exec": "cmd",
"args": [
    "/C",
    "mkdir %homedrive%%homepath%/test"
]
```
- Windows (`powershell`):
```
"exec": "powershell",
"args": [
    "-c",
    "mkdir $HOME/test"
]
```

# Demo
![](https://andy.codes/assets/img/nansi/nansi_demo.gif)
