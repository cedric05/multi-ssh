## Multi-Ssh

Running a command across multiple servers is usuall devops job. Running same command across multiple instances is very rare, but i got myself into that. 

Multi-ssh takes care of running same command across multiple instances and prints output accordingly, with readline (takes care of keeping history)


### Install

`cargo install --path .`

### Usage

#### Command runner
`multi_ssh  --tag cluster_dev --config node_config.json shell`

(shell here is optional)

#### Copy files

`multi_ssh  --tag cluster_alph --config node_config.json copy`

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