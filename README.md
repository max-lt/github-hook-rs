# Github hook server

## Install

Copy executable somewhere (for example in `/usr/local/bin/github-hook-rs`)

Create '/etc/systemd/system/github-hook.service'
```
[Unit]
Description=Github Webhook Server
After=network.target

[Service]
ExecStart=/usr/local/bin/github-hook-rs
Restart=always
# Restart service after 10 seconds if node service crashes
RestartSec=10
# Output to syslog
StandardOutput=syslog
StandardError=syslog
SyslogIdentifier=github-hook
#User=<alternate user>
#Group=<alternate group>
Environment=PORT=8080 CONFIG=/home/me/my-config.yml

[Install]
WantedBy=multi-user.target
```

## Start
```
systemctl start github-hook
```

## Example of config file

```yml
repositories:
  project1:
    secret: project1secret
    script: /some/path/script.sh
  project2:
    secret: project2secret
    script: /another/path/script.sh --some-argument
  project3:
    secret: project1secret
    script: /some/path/script.sh
    branch: master # optional
```

# Configure your github project: 

> Note that this is an example for the "project1" repository.

Go into your project's settings > webhooks

Set `Payload URL` to `http://example.com:8080/github-hook/project1`

Set `Content type` to `application/json`

Set `Secret` to `project1secret`
