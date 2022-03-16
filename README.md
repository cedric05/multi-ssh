## Multi-Ssh

Running a command across multiple servers is usuall devops job. Running same command across multiple instances is very rare, but i got myself into that. 

Multi-ssh takes care of running same command across multiple instances and prints output accordingly, with readline (takes care of keeping history)


### Install

`cargo install --path .`

### Usage

`multi-ssh  --tag cluster_dev --config node_config.json`

Run commands like a ssh-shell.

To copy files from local to all remote nodes, follow below format

`.copy  <list of files in `,` seperated>` (inspired from sqlite shell)

### Config file


```json
[
  {
    "public_address": "171.31.0.22",
    "keyfile": "~/.ssh/id_rsa",
    "tag": "cluster-alpha",
  },
  {
    "public_address": "171.31.0.3",
    "keyfile": "~/.ssh/id_rsa",
    "tag": "cluster-alpha",
  }
]
```