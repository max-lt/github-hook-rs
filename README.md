# Github hook server

## Install

### Download executable

```bash
curl -L https://github.com/max-lt/github-hook-rs/releases/download/v0.1.4/github-hook \
  -o github-hook
```

### Make it executable

```bash
chmod +x github-hook
```

### Check if it works

```bash
./github-hook --version
```

### Move it to a directory in your PATH

```bash
sudo mv github-hook /usr/local/bin/
```

### Create config file

```bash
touch ~/github-hook-config.yml
```

#### Example of config file

```yml
# ~/github-hook-config.yml

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


### Create a systemd service (optional)

#### Create '/etc/systemd/system/github-hook.service'

```ini
# /etc/systemd/system/github-hook.service

[Unit]
Description=Github Webhook Server
After=network.target

[Service]
ExecStart=/usr/local/bin/github-hook
Restart=always
# Restart service after 10 seconds if node service crashes
RestartSec=10
# Output to syslog
StandardOutput=syslog
StandardError=syslog
SyslogIdentifier=github-hook
#User=<alternate user>
#Group=<alternate group>
Environment=PORT=8080 CONFIG=/path/to/github-hook-config.yml

[Install]
WantedBy=multi-user.target
```

#### Start the service
```
systemctl start github-hook
```

### Configure your github project: 

#### GitHub Webhook

> Note that this is an example for the "project1" repository.

Go into your project's settings > webhooks

Set `Payload URL` to `http://example.com:8080/github-hook/project1`

Set `Content type` to `application/json`

Set `Secret` to `project1secret`

#### Github Actions

You can also use this in your github actions. Just add the following step to your workflow:

```yaml
jobs:
  # Your other jobs
  # ...
  deploy_hook:
    name: Deploy hook
    runs-on: ubuntu-latest
    if: github.event_name == 'push' && (github.ref == 'refs/heads/main')
    steps:
      - name: Invoke deployment hook
        uses: distributhor/workflow-webhook@v3
        with:
          webhook_url: ${{ secrets.WEBHOOK_URL }} # http://example.com:8080/github-hook/project1
          webhook_secret: ${{ secrets.WEBHOOK_SECRET }} # project1 secret
```
